//! fix-issues-70-73 / #70：整表与关系数组的无损分片。
//! UT 使用可计数的剧本式 HTTP mock；ST-MCP-18 使用真实隔离 backend + stdio。

use std::any::Any;
use std::collections::HashMap;
use std::io::{BufRead, BufReader, Read, Write};
use std::net::TcpListener as StdTcpListener;
use std::panic::{catch_unwind, resume_unwind, AssertUnwindSafe};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use coldrawdb_mcp::api::ApiClient;
use coldrawdb_mcp::collab::{diff::diff_diagrams, drive_ops};
use coldrawdb_mcp::reporter;
use coldrawdb_mcp::{Config, McpService};
use futures_util::FutureExt;
use serde_json::{json, Value};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

fn finish_case(id: &str, started: Instant, outcome: Result<(), Box<dyn Any + Send>>) {
    let failed = outcome.is_err();
    reporter::report(
        id,
        if failed {
            Err("测试断言失败".into())
        } else {
            Ok(())
        },
        started.elapsed().as_millis(),
    );
    if let Err(payload) = outcome {
        resume_unwind(payload);
    }
}

fn service(base_url: String) -> McpService {
    let config = Config::from_values(HashMap::from([
        ("COLDRAWDB_BASE_URL".into(), base_url),
        ("COLDRAWDB_REQUEST_TIMEOUT_SECS".into(), "10".into()),
    ]))
    .unwrap();
    McpService::new(ApiClient::new(config).unwrap())
}

async fn read_request(stream: &mut TcpStream) -> (String, String, Value) {
    let mut bytes = Vec::new();
    let (header_end, content_length) = loop {
        let mut chunk = [0_u8; 8192];
        let count = stream.read(&mut chunk).await.unwrap();
        assert!(count > 0, "请求提前结束");
        bytes.extend_from_slice(&chunk[..count]);
        if let Some(index) = bytes.windows(4).position(|window| window == b"\r\n\r\n") {
            let header_end = index + 4;
            let headers = String::from_utf8_lossy(&bytes[..index]);
            let content_length = headers
                .lines()
                .find_map(|line| {
                    line.to_ascii_lowercase()
                        .strip_prefix("content-length:")
                        .map(str::trim)
                        .map(str::parse::<usize>)
                })
                .transpose()
                .unwrap()
                .unwrap_or(0);
            if bytes.len() >= header_end + content_length {
                break (header_end, content_length);
            }
        }
    };
    let headers = String::from_utf8_lossy(&bytes[..header_end]);
    let mut parts = headers.lines().next().unwrap().split_whitespace();
    let method = parts.next().unwrap().to_string();
    let path = parts.next().unwrap().to_string();
    let body = if content_length == 0 {
        json!({})
    } else {
        serde_json::from_slice(&bytes[header_end..header_end + content_length]).unwrap()
    };
    (method, path, body)
}

async fn write_response(stream: &mut TcpStream, body: Value) {
    let payload = body.to_string();
    let response = format!(
        "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{payload}",
        payload.len()
    );
    stream.write_all(response.as_bytes()).await.unwrap();
}

async fn scripted_backend(
    responses: Vec<Value>,
) -> (String, Arc<Mutex<Vec<(String, String, Value)>>>) {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let captured = Arc::new(Mutex::new(Vec::new()));
    let requests = captured.clone();
    tokio::spawn(async move {
        for response in responses {
            let (mut stream, _) = listener.accept().await.unwrap();
            let (method, path, body) = read_request(&mut stream).await;
            requests.lock().unwrap().push((method, path, body));
            write_response(&mut stream, response).await;
        }
    });
    (format!("http://{address}"), captured)
}

async fn get_put_backend(
    diagram: Value,
    revision: i64,
) -> (String, Arc<Mutex<Vec<(String, String, Value)>>>) {
    scripted_backend(vec![
        json!({"code":0,"data":diagram}),
        json!({"code":0,"data":{"id":"d1","revision":revision}}),
    ])
    .await
}

fn base_diagram() -> Value {
    json!({
        "id":"d1","name":"分片","revision":4,
        "tables":[
            {"id":"t-users","name":"users","comment":"原表","x":10,"y":20,"color":"#111",
             "fields":[{"id":"f-user-id","name":"id","type":"INT"},
                       {"id":"f-user-name","name":"name","type":"TEXT"}]},
            {"id":"t-orders","name":"orders","comment":"","x":300,"y":20,"color":"#222",
             "fields":[{"id":"f-order-id","name":"id","type":"INT"},
                       {"id":"f-order-user","name":"user_id","type":"INT"}]}
        ],
        "references":[{"id":"r-existing","start_table_id":"t-users","start_field_id":"f-user-id",
                       "end_table_id":"t-orders","end_field_id":"f-order-user",
                       "type_":"one_to_many","on_delete":"RESTRICT","on_update":"NO_ACTION"}],
        "areas":[],"notes":[]
    })
}

fn full_new_fields() -> Vec<Value> {
    vec![
        json!({"id":"src-price","name":"price","type":"DECIMAL","comment":"含税金额",
               "increment":true,"unique":true,"check":"price >= 0","tag":"金额",
               "dict_code":"money","default":"0","primary":true,"not_null":true}),
        json!({"id":"src-note","name":"note","type":"TEXT","comment":"备注",
               "increment":false,"unique":false,"check":"","tag":"说明",
               "dict_code":"","default":"''","primary":false,"not_null":false}),
    ]
}

fn create_table_arguments(fields: Value) -> Value {
    json!({"id":"d1","action":"create","table_id":"src-table","name":"invoice",
           "comment":"发票表","x":123.5,"y":456.25,"color":"#abcdef","locked":true,
           "fields":fields})
}

#[tokio::test]
async fn ut_mcp_41_table_create_preserves_fields_and_returns_actual_ids() {
    let started = Instant::now();
    let outcome = AssertUnwindSafe(async {
        let original = base_diagram();
        let (base, captured) = get_put_backend(original.clone(), 5).await;
        let response = service(base)
            .call(
                "update_table",
                create_table_arguments(json!(full_new_fields())),
            )
            .await
            .unwrap();
        assert_eq!(response["id"], "d1");
        assert_eq!(response["revision"], 5);
        let actual_table_id = response["table_id"].as_str().unwrap();
        assert!(actual_table_id.starts_with("auto-"));
        assert_ne!(actual_table_id, "src-table");
        let field_map = response["field_id_map"].as_object().unwrap();
        assert_eq!(field_map.len(), 2);
        assert!(field_map["src-price"]
            .as_str()
            .unwrap()
            .starts_with("auto-"));
        assert!(field_map["src-note"].as_str().unwrap().starts_with("auto-"));

        let requests = captured.lock().unwrap();
        assert_eq!(requests.len(), 2, "合法 create 必须恰好一组 GET/PUT");
        assert_eq!(requests[0].0, "GET");
        assert_eq!(requests[1].0, "PUT");
        let sent = &requests[1].2["diagram"];
        assert_eq!(sent["tables"][0], original["tables"][0]);
        assert_eq!(sent["tables"][1], original["tables"][1]);
        assert_eq!(sent["references"], original["references"]);
        let created = &sent["tables"][2];
        assert_eq!(created["id"], response["table_id"]);
        for key in ["name", "comment", "x", "y", "color", "locked"] {
            assert_eq!(created[key], create_table_arguments(json!([]))[key]);
        }
        for (index, source) in full_new_fields().iter().enumerate() {
            let actual = &created["fields"][index];
            let mut expected = source.clone();
            expected["id"] = field_map[source["id"].as_str().unwrap()].clone();
            assert_eq!(actual, &expected, "字段全属性与顺序必须保持");
        }
        drop(requests);

        let invalid = vec![
            create_table_arguments(json!([
                {"id":"dup","name":"a"},{"id":"dup","name":"b"}
            ])),
            json!({"id":"d1","action":"create","table_id":"t","fields":[]}),
            json!({"id":"d1","action":"create","table_id":"t","name":"t"}),
            json!({"id":"d1","action":"update","table_id":"t-users","fields":[]}),
        ];
        for arguments in invalid {
            let (base, captured) = scripted_backend(vec![]).await;
            let error = service(base)
                .call("update_table", arguments)
                .await
                .unwrap_err();
            assert_eq!(error.code, "VALIDATION_ERROR");
            assert!(captured.lock().unwrap().is_empty(), "非法输入不得发送 HTTP");
        }

        let (base, captured) = get_put_backend(original, 5).await;
        let response = service(base)
            .call(
                "update_table",
                json!({"id":"d1","table_id":"t-users","comment":"缺省 action"}),
            )
            .await
            .unwrap();
        assert_eq!(response, json!({"id":"d1","revision":5}));
        assert_eq!(
            captured.lock().unwrap()[1].2["diagram"]["tables"][0]["comment"],
            "缺省 action"
        );
    })
    .catch_unwind()
    .await;
    finish_case("UT-MCP-41", started, outcome);
}

#[tokio::test]
async fn ut_mcp_42_batch_reference_create_uses_one_snapshot() {
    let started = Instant::now();
    let outcome =
        AssertUnwindSafe(async {
            let (base, captured) = get_put_backend(base_diagram(), 5).await;
            let response = service(base)
            .call("update_reference", json!({"id":"d1","action":"create","references":[
                {"ref_id":"r-explicit","start_table_id":"t-users","start_field_id":"f-user-id",
                 "end_table_id":"t-orders","end_field_id":"f-order-id"},
                {"start_table_id":"t-orders","start_field_id":"f-order-user",
                 "end_table_id":"t-users","end_field_id":"f-user-id"}
            ]}))
            .await
            .unwrap();
            assert_eq!(response, json!({"id":"d1","revision":5}));
            let requests = captured.lock().unwrap();
            assert_eq!(requests.len(), 2, "批量 create 必须一次 GET + 一次 PUT");
            let references = requests[1].2["diagram"]["references"].as_array().unwrap();
            assert_eq!(references.len(), 3);
            assert_eq!(references[0]["id"], "r-existing");
            assert_eq!(references[1]["id"], "r-explicit");
            let generated = references[2]["id"].as_str().unwrap();
            assert!(!generated.is_empty());
            assert_ne!(generated, "r-explicit");
            drop(requests);

            let duplicate_batch = json!({"id":"d1","action":"create","references":[
                {"ref_id":"dup","start_table_id":"t-users","start_field_id":"f-user-id",
                 "end_table_id":"t-orders","end_field_id":"f-order-id"},
                {"ref_id":"dup","start_table_id":"t-users","start_field_id":"f-user-id",
                 "end_table_id":"t-orders","end_field_id":"f-order-id"}
            ]});
            let existing_conflict = json!({"id":"d1","action":"create","references":[
                {"ref_id":"r-existing","start_table_id":"t-users","start_field_id":"f-user-id",
                 "end_table_id":"t-orders","end_field_id":"f-order-id"}
            ]});
            for arguments in [duplicate_batch, existing_conflict] {
                let (base, captured) =
                    scripted_backend(vec![json!({"code":0,"data":base_diagram()})]).await;
                let error = service(base)
                    .call("update_reference", arguments)
                    .await
                    .unwrap_err();
                assert_eq!(error.code, "VALIDATION_ERROR");
                let requests = captured.lock().unwrap();
                assert_eq!(requests.len(), 1, "冲突批次最多读取，不得 PUT");
                assert_eq!(requests[0].0, "GET");
            }

            for arguments in [
                json!({"id":"d1","action":"create","references":[],"start_table_id":"t-users"}),
                json!({"id":"d1","action":"create","references":[]}),
                json!({"id":"d1","action":"update","ref_id":"r-existing","references":[{}]}),
                json!({"id":"d1","action":"create","references":[{
                    "start_table_id":"t-users","start_field_id":"f-user-id",
                    "end_table_id":"t-orders","end_field_id":"f-order-id","cardinality":"bad"
                }]}),
            ] {
                let (base, captured) = scripted_backend(vec![]).await;
                let error = service(base)
                    .call("update_reference", arguments)
                    .await
                    .unwrap_err();
                assert_eq!(error.code, "VALIDATION_ERROR");
                assert!(captured.lock().unwrap().is_empty());
            }

            let (base, captured) = get_put_backend(base_diagram(), 5).await;
            service(base)
                .call(
                    "update_reference",
                    json!({"id":"d1","action":"delete","ref_id":"r-existing"}),
                )
                .await
                .unwrap();
            assert!(captured.lock().unwrap()[1].2["diagram"]["references"]
                .as_array()
                .unwrap()
                .is_empty());
        })
        .catch_unwind()
        .await;
    finish_case("UT-MCP-42", started, outcome);
}

#[tokio::test]
async fn ut_mcp_43_reference_type_and_cascades_are_normalized() {
    let started = Instant::now();
    let outcome = AssertUnwindSafe(async {
        let (base, captured) = get_put_backend(base_diagram(), 5).await;
        service(base)
            .call(
                "update_reference",
                json!({"id":"d1","action":"create","ref_id":"r-single",
                "start_table_id":"t-users","start_field_id":"f-user-id",
                "end_table_id":"t-orders","end_field_id":"f-order-id"}),
            )
            .await
            .unwrap();
        let requests = captured.lock().unwrap();
        let created = &requests[1].2["diagram"]["references"][1];
        assert_eq!(created["type_"], "many_to_one");
        assert_eq!(created["on_delete"], "NO_ACTION");
        assert_eq!(created["on_update"], "NO_ACTION");
        assert!(created.get("cardinality").is_none());
        drop(requests);

        let cascades = [
            "cascade",
            "set_null",
            "set_default",
            "restrict",
            "no_action",
        ];
        let expected = [
            "CASCADE",
            "SET_NULL",
            "SET_DEFAULT",
            "RESTRICT",
            "NO_ACTION",
        ];
        let batch: Vec<Value> = cascades
            .iter()
            .enumerate()
            .map(|(index, cascade)| {
                json!({"ref_id":format!("r-cascade-{index}"),
                "start_table_id":"t-users","start_field_id":"f-user-id",
                "end_table_id":"t-orders","end_field_id":"f-order-id",
                "cardinality":"one_to_one","on_delete":cascade,"on_update":cascade})
            })
            .collect();
        let (base, captured) = get_put_backend(base_diagram(), 5).await;
        service(base)
            .call(
                "update_reference",
                json!({"id":"d1","action":"create","references":batch}),
            )
            .await
            .unwrap();
        let requests = captured.lock().unwrap();
        let references = requests[1].2["diagram"]["references"].as_array().unwrap();
        for (index, reference) in references[1..].iter().enumerate() {
            assert_eq!(reference["type_"], "one_to_one");
            assert_eq!(reference["on_delete"], expected[index]);
            assert_eq!(reference["on_update"], expected[index]);
            assert!(reference.get("cardinality").is_none());
        }
        drop(requests);

        let mut original = base_diagram();
        original["references"][0]["cardinality"] = json!("many_to_many");
        original["references"][0]
            .as_object_mut()
            .unwrap()
            .remove("type_");
        let (base, captured) = get_put_backend(original, 5).await;
        service(base)
            .call(
                "update_reference",
                json!({"id":"d1","action":"update","ref_id":"r-existing",
                "cardinality":"many_to_many","on_delete":"set_default","on_update":"restrict"}),
            )
            .await
            .unwrap();
        let requests = captured.lock().unwrap();
        let updated = &requests[1].2["diagram"]["references"][0];
        assert_eq!(updated["type_"], "many_to_many");
        assert_eq!(updated["on_delete"], "SET_DEFAULT");
        assert_eq!(updated["on_update"], "RESTRICT");
        assert!(
            updated.get("cardinality").is_none(),
            "mock GET 原字段不得当成写入证据"
        );
    })
    .catch_unwind()
    .await;
    finish_case("UT-MCP-43", started, outcome);
}

#[tokio::test]
async fn ut_mcp_49_reference_endpoint_existence_and_field_ownership() {
    let started = Instant::now();
    let outcome = AssertUnwindSafe(async {
        let actual = json!({
            "id":"d1","revision":4,
            "tables":[
                {"id":"auto-users","name":"users","fields":[
                    {"id":"auto-user-id","name":"id","type":"INT"},
                    {"id":"auto-user-name","name":"name","type":"TEXT"}
                ]},
                {"id":"auto-orders","name":"orders","fields":[
                    {"id":"auto-order-id","name":"id","type":"INT"},
                    {"id":"auto-order-user","name":"user_id","type":"INT"}
                ]}
            ],
            "references":[],"areas":[],"notes":[]
        });
        let valid = json!({"start_table_id":"auto-users","start_field_id":"auto-user-id",
            "end_table_id":"auto-orders","end_field_id":"auto-order-user"});
        let invalid_cases = vec![
            json!({"id":"d1","action":"create","start_table_id":"t-users","start_field_id":"f-user-id",
                "end_table_id":"t-orders","end_field_id":"f-order-user"}),
            json!({"id":"d1","action":"create","start_table_id":"auto-missing","start_field_id":"auto-user-id",
                "end_table_id":"auto-orders","end_field_id":"auto-order-user"}),
            json!({"id":"d1","action":"create","start_table_id":"auto-users","start_field_id":"auto-order-id",
                "end_table_id":"auto-orders","end_field_id":"auto-order-user"}),
            json!({"id":"d1","action":"create","references":[valid.clone(),
                {"start_table_id":"auto-users","start_field_id":"auto-user-id",
                 "end_table_id":"auto-orders","end_field_id":"auto-user-name"}]}),
        ];
        for arguments in invalid_cases {
            let (base, captured) =
                scripted_backend(vec![json!({"code":0,"data":actual.clone()})]).await;
            let error = service(base)
                .call("update_reference", arguments)
                .await
                .unwrap_err();
            assert_eq!(error.code, "VALIDATION_ERROR");
            assert!(error.details.as_ref().unwrap().get("endpoint").is_some());
            let requests = captured.lock().unwrap();
            assert_eq!(requests.len(), 1, "无效端点只能 GET，不得 PUT/WS");
            assert_eq!(requests[0].0, "GET");
        }

        let (base, captured) = get_put_backend(actual, 5).await;
        service(base)
            .call(
                "update_reference",
                json!({"id":"d1","action":"create","references":[valid]}),
            )
            .await
            .unwrap();
        let requests = captured.lock().unwrap();
        assert_eq!(requests.len(), 2);
        assert_eq!(requests[1].0, "PUT");
        assert_eq!(requests[1].2["diagram"]["references"][0]["start_field_id"], "auto-user-id");
        assert_eq!(requests[1].2["diagram"]["references"][0]["end_field_id"], "auto-order-user");
    })
    .catch_unwind()
    .await;
    finish_case("UT-MCP-49", started, outcome);
}

#[test]
fn ut_mcp_44_room_diff_emits_whole_table_and_reference_ops() {
    let started = Instant::now();
    let outcome = catch_unwind(AssertUnwindSafe(|| {
        let base = base_diagram();
        let mut desired = base.clone();
        desired["tables"].as_array_mut().unwrap().push(json!({
            "id":"auto-new-table","name":"new_table","fields":full_new_fields()
        }));
        desired["references"].as_array_mut().unwrap().extend([
            json!({"id":"r-new-1","start_table_id":"t-users","start_field_id":"f-user-id",
                   "end_table_id":"t-orders","end_field_id":"f-order-id","type_":"one_to_many"}),
            json!({"id":"r-new-2","start_table_id":"t-orders","start_field_id":"f-order-user",
                   "end_table_id":"t-users","end_field_id":"f-user-id","type_":"many_to_one"}),
        ]);
        let ops = diff_diagrams(&base, &desired);
        assert_eq!(ops.len(), 3);
        assert_eq!(ops[0]["type"], "table.create");
        assert_eq!(ops[0]["changes"]["fields"].as_array().unwrap().len(), 2);
        assert!(!ops.iter().any(|op| op["type"] == "field.create"));
        let reference_ops: Vec<_> = ops
            .iter()
            .filter(|op| op["type"] == "reference.create")
            .collect();
        assert_eq!(reference_ops.len(), 2);
        let inbound = vec![
            json!({"type":"connected","serverRev":4}),
            json!({"type":"ack","serverRev":5}),
            json!({"type":"ack","serverRev":6}),
            json!({"type":"ack","serverRev":7}),
        ];
        let revision = drive_ops(&ops, &inbound, 4).unwrap();
        assert_eq!(revision, 7, "最终 revision 必须来自最后一条 ack");
    }));
    finish_case("UT-MCP-44", started, outcome);
}

// ── ST-MCP-18：真实隔离 backend + coldrawdb-mcp stdio ─────────────────────

struct ChildGuard(Child);

impl Drop for ChildGuard {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

fn backend_binary() -> PathBuf {
    if let Some(path) = std::env::var_os("COLDRAWDB_BACKEND_BIN") {
        return PathBuf::from(path);
    }
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
    let debug = root.join("backend/target/debug/backend");
    if debug.is_file() {
        debug
    } else {
        root.join("backend/target/release/backend")
    }
}

fn reserve_port() -> u16 {
    StdTcpListener::bind("127.0.0.1:0")
        .unwrap()
        .local_addr()
        .unwrap()
        .port()
}

async fn wait_for_backend(base_url: &str, child: &mut Child) {
    let client = reqwest::Client::builder().no_proxy().build().unwrap();
    let deadline = Instant::now() + Duration::from_secs(30);
    loop {
        if let Some(status) = child.try_wait().unwrap() {
            panic!("真实 backend 提前退出: {status}");
        }
        if let Ok(response) = client
            .get(format!("{base_url}/api/v1/diagrams/health"))
            .send()
            .await
        {
            if response.status().is_success() {
                return;
            }
        }
        assert!(Instant::now() < deadline, "等待真实 backend 健康检查超时");
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
}

struct StdioSession {
    next_id: i64,
    stdin: std::process::ChildStdin,
    reader: BufReader<std::process::ChildStdout>,
}

impl StdioSession {
    fn call(&mut self, method: &str, params: Value) -> Value {
        let id = self.next_id;
        self.next_id += 1;
        writeln!(
            self.stdin,
            "{}",
            json!({"jsonrpc":"2.0","id":id,"method":method,"params":params})
        )
        .unwrap();
        self.stdin.flush().unwrap();
        let mut line = String::new();
        assert!(
            self.reader.read_line(&mut line).unwrap() > 0,
            "stdio 提前关闭"
        );
        let response: Value = serde_json::from_str(&line).unwrap();
        assert_eq!(response["id"], id);
        assert!(response.get("error").is_none(), "JSON-RPC 错误: {response}");
        response["result"].clone()
    }

    fn tool(&mut self, name: &str, arguments: Value) -> Value {
        let result = self.call("tools/call", json!({"name":name,"arguments":arguments}));
        assert_eq!(result["isError"], false, "工具 {name} 失败: {result}");
        let structured = result["structuredContent"].clone();
        assert!(structured.is_object());
        let text: Value =
            serde_json::from_str(result["content"][0]["text"].as_str().unwrap()).unwrap();
        assert_eq!(text, structured, "text 与 structuredContent 必须一致");
        structured
    }
}

fn large_fixture() -> (Vec<Value>, Vec<Value>) {
    let mut tables = Vec::new();
    for table_index in 0..54 {
        let field_count = if table_index < 53 { 16 } else { 15 };
        let fields: Vec<Value> = (0..field_count)
            .map(|field_index| {
                json!({"id":format!("src-field-{table_index}-{field_index}"),
                    "name":format!("column_{field_index}"),
                    "type":if field_index % 2 == 0 {"INT"} else {"VARCHAR(80)"},
                    "comment":format!("字段 {table_index}/{field_index}：完整注释、约束与默认值必须保留"),
                    "increment":field_index == 0,"unique":field_index % 3 == 0,
                    "check":format!("column_{field_index} >= 0"),"tag":format!("tag-{}",field_index % 4),
                    "dict_code":format!("dict-{}",field_index % 2),"default":"0",
                    "primary":field_index == 0,"not_null":field_index % 2 == 0})
            })
            .collect();
        tables.push(json!({"id":format!("src-table-{table_index}"),
            "name":format!("table_{table_index}"),"comment":format!("第 {table_index} 张全保真表"),
            "x":table_index * 240,"y":(table_index % 6) * 320,
            "color":format!("#{:06x}",0x100000 + table_index * 173),"locked":false,"fields":fields}));
    }
    let references: Vec<Value> = (0..76)
        .map(|index| {
            let start = index % 54;
            let end = (index * 7 + 11) % 54;
            json!({"ref_id":format!("src-reference-{index}"),
                "start_table_id":format!("src-table-{start}"),"start_field_id":format!("src-field-{start}-0"),
                "end_table_id":format!("src-table-{end}"),"end_field_id":format!("src-field-{end}-1"),
                "cardinality":if index % 2 == 0 {"one_to_many"} else {"many_to_one"},
                "on_delete":if index % 2 == 0 {"cascade"} else {"set_null"},
                "on_update":if index % 3 == 0 {"no_action"} else {"restrict"},
                "name":format!("relationship_{index}_with_a_readable_persistent_name"),
                "color":format!("#{:06x}",0x300000 + index * 211)})
        })
        .collect();
    (tables, references)
}

fn assert_argument_budget(arguments: &Value) {
    let bytes = serde_json::to_vec(arguments).unwrap().len();
    assert!(bytes < 49_152, "单次工具参数超过 48 KiB: {bytes}");
}

fn field_value<'a>(field: &'a Value, key: &str) -> &'a Value {
    if key == "type" {
        field
            .get("type")
            .or_else(|| field.get("type_"))
            .unwrap_or(&Value::Null)
    } else {
        field.get(key).unwrap_or(&Value::Null)
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
#[ignore = "需要真实 backend 二进制；verify 预跑显式执行"]
async fn st_mcp_18_real_backend_and_stdio_54_tables() {
    let started = Instant::now();
    let outcome = AssertUnwindSafe(async {
        let temp = tempfile::tempdir().unwrap();
        let db_path = temp.path().join("chunked.sqlite");
        let port = reserve_port();
        let base_url = format!("http://127.0.0.1:{port}");
        let backend_bin = backend_binary();
        assert!(backend_bin.is_file(), "真实 backend 不存在: {}", backend_bin.display());
        let backend_root = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap().join("backend");
        let mut backend = ChildGuard(
            Command::new(&backend_bin)
                .current_dir(backend_root)
                .env("COLDRAWDB_BIND_ADDR", format!("127.0.0.1:{port}"))
                .env("COLDRAWDB_DB_URL", format!("sqlite://{}?mode=rwc", db_path.display()))
                .env("COLDRAWDB_DIAGRAMS_AUTH", "off")
                .env("COLDRAWDB_STATIC_DIR", temp.path().join("no-static"))
                .env("RUST_LOG", "warn")
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .spawn()
                .unwrap(),
        );
        wait_for_backend(&base_url, &mut backend.0).await;

        let (tables, references) = large_fixture();
        let original_bytes = serde_json::to_vec(&json!({"tables":tables,"references":references}))
            .unwrap()
            .len();
        assert!(original_bytes > 63_284, "原始 fixture 必须超过实测边界: {original_bytes}");
        assert_eq!(tables.iter().map(|table| table["fields"].as_array().unwrap().len()).sum::<usize>(), 863);
        assert_eq!(references.len(), 76);

        let base_for_stdio = base_url.clone();
        tokio::task::spawn_blocking(move || {
            let mut mcp = ChildGuard(
                Command::new(env!("CARGO_BIN_EXE_coldrawdb-mcp"))
                    .env("COLDRAWDB_BASE_URL", base_for_stdio)
                    .env("COLDRAWDB_AUTH_PROBE", "off")
                    .env_remove("COLDRAWDB_ACCESS_TOKEN")
                    .stdin(Stdio::piped())
                    .stdout(Stdio::piped())
                    .stderr(Stdio::piped())
                    .spawn()
                    .unwrap(),
            );
            let mut session = StdioSession {
                next_id: 1,
                stdin: mcp.0.stdin.take().unwrap(),
                reader: BufReader::new(mcp.0.stdout.take().unwrap()),
            };
            let mut stderr = mcp.0.stderr.take().unwrap();
            session.call("initialize", json!({"protocolVersion":"2025-06-18","capabilities":{},"clientInfo":{"name":"st-mcp-18","version":"1"}}));
            let tools = session.call("tools/list", json!({}));
            assert_eq!(tools["tools"].as_array().unwrap().len(), 12);

            let created = session.tool("create_diagram", json!({"name":"ST-MCP-18","database":"postgresql"}));
            assert_eq!(created.as_object().unwrap().keys().collect::<Vec<_>>(), ["id"]);
            let diagram_id = created["id"].as_str().unwrap().to_string();
            let mut table_id_map = HashMap::new();
            let mut field_id_map = HashMap::new();
            let mut expected_revision = 0_i64;
            for table in &tables {
                let arguments = json!({"id":diagram_id,"action":"create","table_id":table["id"],
                    "name":table["name"],"comment":table["comment"],"x":table["x"],"y":table["y"],
                    "color":table["color"],"locked":table["locked"],"fields":table["fields"]});
                assert_argument_budget(&arguments);
                let result = session.tool("update_table", arguments);
                expected_revision += 1;
                assert_eq!(result["revision"], expected_revision);
                let actual_table_id = result["table_id"].as_str().unwrap().to_string();
                assert!(actual_table_id.starts_with("auto-"));
                table_id_map.insert(table["id"].as_str().unwrap().to_string(), actual_table_id);
                for (old_id, new_id) in result["field_id_map"].as_object().unwrap() {
                    assert!(new_id.as_str().unwrap().starts_with("auto-"));
                    field_id_map.insert(old_id.clone(), new_id.as_str().unwrap().to_string());
                }
            }
            assert_eq!(field_id_map.len(), 863);

            let remapped: Vec<Value> = references
                .iter()
                .map(|reference| {
                    let mut remapped = reference.clone();
                    for key in ["start_table_id", "end_table_id"] {
                        let old = reference[key].as_str().unwrap();
                        remapped[key] = json!(table_id_map[old]);
                    }
                    for key in ["start_field_id", "end_field_id"] {
                        let old = reference[key].as_str().unwrap();
                        remapped[key] = json!(field_id_map[old]);
                    }
                    remapped
                })
                .collect();
            let mut reference_calls = 0;
            let mut pending: Vec<Value> = Vec::new();
            for reference in remapped {
                let mut candidate = pending.clone();
                candidate.push(reference.clone());
                let candidate_arguments = json!({"id":diagram_id,"action":"create","references":candidate});
                if !pending.is_empty() && serde_json::to_vec(&candidate_arguments).unwrap().len() >= 12 * 1024 {
                    let arguments = json!({"id":diagram_id,"action":"create","references":pending});
                    assert_argument_budget(&arguments);
                    let result = session.tool("update_reference", arguments);
                    expected_revision += 1;
                    assert_eq!(result["revision"], expected_revision);
                    reference_calls += 1;
                    pending = vec![reference];
                } else {
                    pending.push(reference);
                }
            }
            if !pending.is_empty() {
                let arguments = json!({"id":diagram_id,"action":"create","references":pending});
                assert_argument_budget(&arguments);
                let result = session.tool("update_reference", arguments);
                expected_revision += 1;
                assert_eq!(result["revision"], expected_revision);
                reference_calls += 1;
            }
            assert!(reference_calls >= 2, "测试必须实际执行多个关系分片");

            let result = session.tool("get_diagram", json!({"id":diagram_id}));
            let diagram = &result["diagram"];
            assert_eq!(diagram["tables"].as_array().unwrap().len(), 54);
            assert_eq!(diagram["references"].as_array().unwrap().len(), 76);
            assert_eq!(diagram["tables"].as_array().unwrap().iter()
                .map(|table| table["fields"].as_array().unwrap().len()).sum::<usize>(), 863);
            for source in &tables {
                let actual_id = &table_id_map[source["id"].as_str().unwrap()];
                let actual = diagram["tables"].as_array().unwrap().iter()
                    .find(|table| &table["id"] == actual_id).unwrap();
                for key in ["name", "comment", "color"] {
                    assert_eq!(actual[key], source[key], "表属性丢失: {key}");
                }
                for key in ["x", "y"] {
                    assert_eq!(actual[key].as_f64(), source[key].as_f64(), "表坐标丢失: {key}");
                }
                for source_field in source["fields"].as_array().unwrap() {
                    let actual_field_id = &field_id_map[source_field["id"].as_str().unwrap()];
                    let actual_field = actual["fields"].as_array().unwrap().iter()
                        .find(|field| &field["id"] == actual_field_id).unwrap();
                    for key in ["name","type","comment","increment","unique","check","tag","dict_code","default","primary","not_null"] {
                        assert_eq!(field_value(actual_field, key), &source_field[key],
                            "字段属性丢失: {}.{key}", source_field["name"].as_str().unwrap());
                    }
                }
            }
            for source in &references {
                let actual = diagram["references"].as_array().unwrap().iter()
                    .find(|reference| reference["id"] == source["ref_id"]).unwrap();
                assert_eq!(actual["type_"], source["cardinality"]);
                assert_eq!(actual["on_delete"], source["on_delete"].as_str().unwrap().to_uppercase());
                assert_eq!(actual["on_update"], source["on_update"].as_str().unwrap().to_uppercase());
                assert_eq!(actual["name"], source["name"]);
                assert_eq!(actual["color"], source["color"]);
                assert_eq!(actual["start_table_id"], json!(table_id_map[source["start_table_id"].as_str().unwrap()]));
                assert_eq!(actual["end_field_id"], json!(field_id_map[source["end_field_id"].as_str().unwrap()]));
            }
            let deleted = session.tool("delete_diagram", json!({"id":diagram_id,"confirm":true}));
            assert_eq!(deleted["deleted"], true);
            drop(session.stdin);
            assert!(mcp.0.wait().unwrap().success());
            let mut logs = String::new();
            stderr.read_to_string(&mut logs).unwrap();
            assert!(logs.is_empty(), "成功 stdio 链路不得写 stderr: {logs}");
        })
        .await
        .unwrap();
        assert!(started.elapsed().as_millis() > 0, "真实 ST 必须记录非零耗时");
    })
    .catch_unwind()
    .await;
    finish_case("ST-MCP-18", started, outcome);
}
