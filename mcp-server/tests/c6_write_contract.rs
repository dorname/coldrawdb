//! fix-remote-github-issues-7-18（2026-09-18，issue #18/#12）：MCP 写工具契约测试
//! 覆盖 UT-MCP-23 / UT-MCP-24 / ST-MCP-13：
//! - 写工具（update_diagram/update_table/update_field/update_reference/layout_diagram）
//!   返回前必须 normalize 为与各自 outputSchema 完全一致的瘦响应；
//! - update_reference 支持 color 参数（create 带入 / update 不传保持原值 / delete 忽略 / 超长拒绝）；
//! - stdio 端到端 layout_diagram 的 structuredContent 必须通过 outputSchema 校验。

use std::collections::HashMap;
use std::io::{BufRead, BufReader, Write};
use std::process::{Command, Stdio};
use std::sync::{Arc, Mutex};
use std::time::Instant;

use coldrawdb_mcp::api::ApiClient;
use coldrawdb_mcp::reporter;
use coldrawdb_mcp::{Config, McpService};
use serde_json::{json, Value};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

fn record(ids: &[&str], started: Instant) {
    for id in ids {
        reporter::report(id, Ok(()), started.elapsed().as_millis());
    }
}

fn service(base_url: String) -> McpService {
    let config = Config::from_values(HashMap::from([
        ("COLDRAWDB_BASE_URL".into(), base_url),
        ("COLDRAWDB_REQUEST_TIMEOUT_SECS".into(), "5".into()),
    ]))
    .unwrap();
    McpService::new(ApiClient::new(config).unwrap())
}

// ── mock 后端（沿用 c3/c5 的手工 TcpListener 模式）─────────────────────────

/// 读取一个完整 HTTP 请求，返回 (method, path, body)。
async fn read_request(stream: &mut TcpStream) -> (String, String, Value) {
    let mut bytes = Vec::new();
    let (header_end, content_length) = loop {
        let mut chunk = [0_u8; 4096];
        let count = stream.read(&mut chunk).await.unwrap();
        if count == 0 {
            panic!("请求提前结束");
        }
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

/// 剧本式 mock 后端：每个连接按顺序应答一个 200 响应体，捕获全部 (method, path, body)。
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
            requests
                .lock()
                .unwrap()
                .push((method, path, body));
            write_response(&mut stream, response).await;
        }
    });
    (format!("http://{address}"), captured)
}

/// 仅应答一次 PUT 的 mock（供 update_diagram 使用）。
async fn put_only_backend(put_data: Value) -> (String, Arc<Mutex<Vec<(String, String, Value)>>>) {
    scripted_backend(vec![json!({"code":0,"data":put_data})]).await
}

/// 先应答 GET 再应答 PUT 的 mock（供 update_table/field/reference/layout 使用）。
async fn get_put_backend(
    get_data: Value,
    put_data: Value,
) -> (String, Arc<Mutex<Vec<(String, String, Value)>>>) {
    scripted_backend(vec![
        json!({"code":0,"data":get_data}),
        json!({"code":0,"data":put_data}),
    ])
    .await
}

// ── 契约断言辅助（手写 outputSchema 校验：字段集 + 类型断言，不引 jsonschema crate）──

/// update_* 四工具 outputSchema：恰好 { id: string, revision: integer ≥ 1 }。
fn assert_update_output(value: &Value, expect_id: &str, expect_revision: i64) {
    let object = value.as_object().expect("写工具响应必须是 object");
    let mut keys: Vec<&str> = object.keys().map(String::as_str).collect();
    keys.sort();
    assert_eq!(keys, ["id", "revision"], "响应必须恰好只含 id/revision");
    assert_eq!(value["id"], json!(expect_id));
    assert_eq!(value["revision"], json!(expect_revision));
    assert!(value["id"].as_str().map(|s| !s.is_empty()).unwrap_or(false));
    assert!(value["revision"].as_i64().unwrap_or(0) >= 1);
}

/// layout_diagram outputSchema：恰好 { id, revision, tables_repositioned }，
/// 类型分别为 string / integer ≥1 / integer ≥0。
fn assert_layout_output(value: &Value, expect_id: &str, expect_count: i64) {
    let object = value.as_object().expect("layout 响应必须是 object");
    let mut keys: Vec<&str> = object.keys().map(String::as_str).collect();
    keys.sort();
    assert_eq!(
        keys,
        ["id", "revision", "tables_repositioned"],
        "layout 响应必须恰好只含 id/revision/tables_repositioned"
    );
    assert_eq!(value["id"], json!(expect_id));
    assert!(value["revision"].as_i64().unwrap_or(0) >= 1);
    assert_eq!(value["tables_repositioned"], json!(expect_count));
    assert!(value["tables_repositioned"].as_i64().unwrap_or(-1) >= 0);
}

/// 单表（含一个字段）+ 一条关系的 GET diagram fixture。
fn diagram_fixture() -> Value {
    json!({
        "id":"d1","name":"契约","revision":5,
        "tables":[{"id":"t1","name":"users","x":0.0,"y":0.0,
                   "fields":[{"id":"f1","name":"id","type":"INT"}]}],
        "references":[{"id":"r1","start_table_id":"t1","start_field_id":"f1",
                       "end_table_id":"t1","end_field_id":"f1",
                       "cardinality":"many_to_one","color":"#f00"}]
    })
}

/// 上游 PUT 返回的「含额外字段」data 形状（envelope 碎片 + 整图字段漂移）。
fn fat_put_data() -> Value {
    json!({"id":"d1","revision":6,"code":0,"request_id":"r-fat",
           "tables":[{"id":"t1","name":"整图泄漏"}],"notes":[]})
}

// ── UT-MCP-23: 写工具瘦响应 normalize（契约）───────────────────────────────

#[tokio::test]
async fn ut_mcp23_write_tools_slim_response_contract() {
    let started = Instant::now();
    // fix-mcp-tool-gaps-issue76：update_diagram 增加 UT-MCP-54 守护后写前先 GET base，
    // mock 必须先应答 GET；desired 需覆盖 base 全部表（t1）以通过缺表守护。
    let guard_safe_desired = json!({
        "id":"d1","revision":5,
        "tables":[{"id":"t1","name":"users","fields":[]}],
        "references":[]
    });

    // 1) update_diagram：上游 PUT 返回含额外字段的 data → 恰好 {id, revision}
    let (base, _) = get_put_backend(diagram_fixture(), fat_put_data()).await;
    let svc = service(base);
    let resp = svc
        .call(
            "update_diagram",
            json!({"id":"d1","expected_revision":5,"diagram":guard_safe_desired}),
        )
        .await
        .unwrap();
    assert_update_output(&resp, "d1", 6);

    // 2) update_table：同上
    let (base, _) = get_put_backend(diagram_fixture(), fat_put_data()).await;
    let resp = service(base)
        .call("update_table", json!({"id":"d1","table_id":"t1","name":" renamed"}))
        .await
        .unwrap();
    assert_update_output(&resp, "d1", 6);

    // 3) update_field：同上
    let (base, _) = get_put_backend(diagram_fixture(), fat_put_data()).await;
    let resp = service(base)
        .call(
            "update_field",
            json!({"id":"d1","table_id":"t1","field_id":"f1","name":"fid"}),
        )
        .await
        .unwrap();
    assert_update_output(&resp, "d1", 6);

    // 4) update_reference：同上
    let (base, _) = get_put_backend(diagram_fixture(), fat_put_data()).await;
    let resp = service(base)
        .call(
            "update_reference",
            json!({"id":"d1","action":"update","ref_id":"r1","cardinality":"one_to_many"}),
        )
        .await
        .unwrap();
    assert_update_output(&resp, "d1", 6);

    // 5) layout_diagram：恰好 {id, revision, tables_repositioned}
    let layout_get = json!({
        "id":"d1","revision":5,
        "tables":[{"id":"a","x":0.0,"y":0.0},{"id":"b","x":10.0,"y":10.0},{"id":"c","x":20.0,"y":20.0}],
        "references":[{"start_table_id":"a","end_table_id":"b"},
                      {"start_table_id":"b","end_table_id":"c"}]
    });
    let (base, _) = get_put_backend(layout_get, fat_put_data()).await;
    let resp = service(base)
        .call("layout_diagram", json!({"id":"d1","mode":"force"}))
        .await
        .unwrap();
    assert_layout_output(&resp, "d1", 3);

    // 6) 上游 data 缺 id → 以请求路径 id 补齐
    let (base, _) = get_put_backend(diagram_fixture(), json!({"revision":7,"request_id":"r-no-id"})).await;
    let resp = service(base)
        .call(
            "update_diagram",
            json!({"id":"d1","expected_revision":5,"diagram":guard_safe_desired}),
        )
        .await
        .unwrap();
    assert_update_output(&resp, "d1", 7);

    // 7) 上游 data 缺 revision → 错误映射，不得产出缺字段的成功响应
    let (base, _) = get_put_backend(diagram_fixture(), json!({"id":"d1","request_id":"r-no-rev"})).await;
    let error = service(base)
        .call(
            "update_diagram",
            json!({"id":"d1","expected_revision":5,"diagram":guard_safe_desired}),
        )
        .await
        .unwrap_err();
    assert_eq!(error.code, "UPSTREAM_ERROR");

    // 8) revision 非数 → 错误映射
    let (base, _) = get_put_backend(diagram_fixture(), json!({"id":"d1","revision":"6"})).await;
    let error = service(base)
        .call(
            "update_diagram",
            json!({"id":"d1","expected_revision":5,"diagram":guard_safe_desired}),
        )
        .await
        .unwrap_err();
    assert_eq!(error.code, "UPSTREAM_ERROR");

    // 9) revision = 0（不满足 outputSchema minimum:1）→ 错误映射
    let (base, _) = get_put_backend(diagram_fixture(), json!({"id":"d1","revision":0})).await;
    let error = service(base)
        .call(
            "update_diagram",
            json!({"id":"d1","expected_revision":5,"diagram":guard_safe_desired}),
        )
        .await
        .unwrap_err();
    assert_eq!(error.code, "UPSTREAM_ERROR");

    record(&["UT-MCP-23"], started);
}

// ── UT-MCP-24: update_reference 支持 color 参数 ────────────────────────────

#[tokio::test]
async fn ut_mcp24_update_reference_color() {
    let started = Instant::now();

    // 1) create + color → PUT body 的 diagram 中该 reference 含 color
    let mut get = diagram_fixture();
    get["references"] = json!([]);
    get["tables"] = json!([
        {"id":"t1","name":"users","x":0.0,"y":0.0,"fields":[{"id":"f1","name":"id"}]},
        {"id":"t2","name":"orders","x":100.0,"y":0.0,"fields":[{"id":"f2","name":"uid"}]}
    ]);
    let (base, captured) = get_put_backend(get, json!({"id":"d1","revision":6})).await;
    let resp = service(base)
        .call(
            "update_reference",
            json!({"id":"d1","action":"create","start_table_id":"t1","start_field_id":"f1",
                   "end_table_id":"t2","end_field_id":"f2","color":"#3af"}),
        )
        .await
        .unwrap();
    assert_update_output(&resp, "d1", 6);
    let requests = captured.lock().unwrap();
    let put_body = &requests[1].2;
    let refs = put_body["diagram"]["references"].as_array().unwrap();
    let created = refs
        .iter()
        .find(|r| r["start_table_id"] == "t1" && r["end_table_id"] == "t2")
        .expect("PUT body 应包含新建 reference");
    assert_eq!(created["color"], json!("#3af"));
    drop(requests);

    // 2) update 不传 color → PUT body 中原值 "#f00" 保持
    let (base, captured) = get_put_backend(diagram_fixture(), json!({"id":"d1","revision":6})).await;
    service(base)
        .call(
            "update_reference",
            json!({"id":"d1","action":"update","ref_id":"r1","cardinality":"one_to_one"}),
        )
        .await
        .unwrap();
    let requests = captured.lock().unwrap();
    let refs = requests[1].2["diagram"]["references"].as_array().unwrap();
    let r1 = refs.iter().find(|r| r["id"] == "r1").unwrap();
    assert_eq!(r1["color"], json!("#f00"), "不传 color 时必须保持原值");
    assert_eq!(r1["type_"], json!("one_to_one"));
    drop(requests);

    // 3) update 传 color → PUT body 中覆盖为新值
    let (base, captured) = get_put_backend(diagram_fixture(), json!({"id":"d1","revision":6})).await;
    service(base)
        .call(
            "update_reference",
            json!({"id":"d1","action":"update","ref_id":"r1","color":"#0f0"}),
        )
        .await
        .unwrap();
    let requests = captured.lock().unwrap();
    let refs = requests[1].2["diagram"]["references"].as_array().unwrap();
    assert_eq!(refs.iter().find(|r| r["id"] == "r1").unwrap()["color"], json!("#0f0"));
    drop(requests);

    // 4) color 超长（>64）→ VALIDATION_ERROR（本地拒绝，不发 HTTP）
    let svc = service("http://127.0.0.1:9".into());
    let long_color = "c".repeat(65);
    for arguments in [
        json!({"id":"d1","action":"create","start_table_id":"t1","start_field_id":"f1",
               "end_table_id":"t2","end_field_id":"f2","color":long_color}),
        json!({"id":"d1","action":"update","ref_id":"r1","color":long_color}),
    ] {
        assert_eq!(
            svc.call("update_reference", arguments)
                .await
                .unwrap_err()
                .code,
            "VALIDATION_ERROR"
        );
    }

    // 5) delete 忽略 color（即使超长也不校验，正常走 GET + PUT）
    let (base, _) = get_put_backend(diagram_fixture(), json!({"id":"d1","revision":6})).await;
    let resp = service(base)
        .call(
            "update_reference",
            json!({"id":"d1","action":"delete","ref_id":"r1","color":"c".repeat(65)}),
        )
        .await
        .unwrap();
    assert_update_output(&resp, "d1", 6);

    record(&["UT-MCP-24"], started);
}

// ── ST-MCP-13: stdio 端到端 layout_diagram structuredContent 过校验 ─────────

// 使用 multi_thread runtime：mock 后端任务在 worker 线程上运行，
// 主线程可阻塞式读写真实子进程的 stdio 管道而不死锁。
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn st_mcp13_stdio_layout_structured_content() {
    let started = Instant::now();

    // mock 后端：GET 返回三表两关系（初始位置轻微错位且过近，布局后必然分离移动）；
    // PUT 返回瘦响应。
    let (base_url, captured) = get_put_backend(
        json!({"id":"d1","name":"布局","revision":5,
               "tables":[{"id":"a","x":0.0,"y":0.0},{"id":"b","x":10.0,"y":10.0},{"id":"c","x":20.0,"y":20.0}],
               "references":[{"start_table_id":"a","end_table_id":"b"},
                             {"start_table_id":"b","end_table_id":"c"}]}),
        json!({"id":"d1","revision":6}),
    )
    .await;

    // 真实子进程 stdio 握手 + tools/call（沿用 ST-MCP-01 模式）。
    let mut child = Command::new(env!("CARGO_BIN_EXE_coldrawdb-mcp"))
        .env("COLDRAWDB_BASE_URL", &base_url)
        // diagram-api-auth：剧本式 mock 只按序应答工具调用，关闭启动期鉴权探测
        .env("COLDRAWDB_AUTH_PROBE", "off")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let mut stdin = child.stdin.take().unwrap();
    writeln!(stdin, "{}", json!({"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-06-18","capabilities":{},"clientInfo":{"name":"openlogos-test","version":"1"}}})).unwrap();
    writeln!(
        stdin,
        "{}",
        json!({"jsonrpc":"2.0","method":"notifications/initialized","params":{}})
    )
    .unwrap();
    writeln!(
        stdin,
        "{}",
        json!({"jsonrpc":"2.0","id":2,"method":"tools/call",
               "params":{"name":"layout_diagram","arguments":{"id":"d1","mode":"force"}}})
    )
    .unwrap();
    drop(stdin);

    let mut reader = BufReader::new(child.stdout.take().unwrap());
    let mut first = String::new();
    let mut second = String::new();
    reader.read_line(&mut first).unwrap();
    reader.read_line(&mut second).unwrap();
    let called: Value = serde_json::from_str(&second).unwrap();
    assert!(child.wait().unwrap().success());

    // structuredContent 必须通过 layout_diagram outputSchema（恰好三字段、类型正确、revision 递增）。
    let structured = called
        .pointer("/result/structuredContent")
        .expect("tools/call 结果缺少 structuredContent");
    assert_layout_output(structured, "d1", 3);
    assert_eq!(structured["revision"], json!(6), "revision 应由 5 递增为 6");
    assert_eq!(called.pointer("/result/isError"), Some(&json!(false)));

    // content[0].text 反序列化必须与 structuredContent 完全一致。
    let text = called
        .pointer("/result/content/0/text")
        .and_then(Value::as_str)
        .expect("tools/call 结果缺少 content[0].text");
    let text_value: Value = serde_json::from_str(text).unwrap();
    assert_eq!(&text_value, structured, "content[0].text 必须与 structuredContent 一致");

    // 坐标已更新：PUT body 中三表位置被布局改写（初始 0/10/20 错位过近，布局按 spacing≈180 分离）。
    let requests = captured.lock().unwrap();
    let put_tables = requests[1].2["diagram"]["tables"].as_array().unwrap();
    let initial = [("a", 0.0_f64, 0.0_f64), ("b", 10.0, 10.0), ("c", 20.0, 20.0)];
    let moved = put_tables.iter().any(|t| {
        let id = t["id"].as_str().unwrap_or("");
        initial
            .iter()
            .find(|(i, _, _)| *i == id)
            .map(|(_, x, y)| {
                (t["x"].as_f64().unwrap_or(*x) - x).abs() > 1e-6
                    || (t["y"].as_f64().unwrap_or(*y) - y).abs() > 1e-6
            })
            .unwrap_or(false)
    });
    assert!(moved, "layout 后至少一张表的坐标应被更新");

    record(&["ST-MCP-13"], started);
}

// ── UT-MCP-28: import_schema payload 前置规范化 ────────────────────────────

#[tokio::test]
async fn ut_mcp28_import_payload_normalized() {
    let started = Instant::now();
    // mock 后端应答一次 import 200，捕获 POST body
    let import_result = json!({
        "code":0,
        "data":{"diagram_id":"d9","imported_tables":1,"imported_fields":1,
                "warnings":[],"source":"mcp"},
        "request_id":"r1"
    });
    let (base, captured) = scripted_backend(vec![import_result.clone()]).await;
    let payload = json!({
        "tables":[{"name":"t_ai","fields":[{"name":"f_ai","type":"INT"}]}]
    });
    let resp = service(base)
        .call(
            "import_schema",
            json!({"format":"drawdb_json","payload":payload}),
        )
        .await
        .unwrap();
    assert_eq!(resp["diagram_id"], json!("d9"));

    let requests = captured.lock().unwrap();
    assert_eq!(requests.len(), 1);
    assert_eq!(requests[0].0, "POST");
    assert_eq!(requests[0].1, "/api/v1/diagrams/import");
    let sent = &requests[0].2["payload"];
    // 顶层 name 兜底
    assert_eq!(sent["name"], json!("imported_diagram"));
    // 缺 id 的表/字段被补全（auto- 前缀）
    let table_id = sent["tables"][0]["id"].as_str().unwrap_or("");
    assert!(table_id.starts_with("auto-"), "表 id 必须自动补全: {table_id}");
    let field_id = sent["tables"][0]["fields"][0]["id"].as_str().unwrap_or("");
    assert!(field_id.starts_with("auto-"), "字段 id 必须自动补全: {field_id}");

    // 已有 name/id 原样保留，不重复补全
    let (base2, captured2) = scripted_backend(vec![import_result]).await;
    let keep = json!({"name":"keep","tables":[{"id":"t1","name":"t","fields":[{"id":"f1","name":"f"}]}]});
    service(base2)
        .call("import_schema", json!({"format":"drawdb_json","payload":keep}))
        .await
        .unwrap();
    let requests2 = captured2.lock().unwrap();
    let sent2 = &requests2[0].2["payload"];
    assert_eq!(sent2["name"], json!("keep"));
    assert_eq!(sent2["tables"][0]["id"], json!("t1"));
    assert_eq!(sent2["tables"][0]["fields"][0]["id"], json!("f1"));

    record(&["UT-MCP-28"], started);
}

// ── UT-MCP-29: Diagram.name 可空契约（存量脏图防御）─────────────────────────

#[tokio::test]
async fn ut_mcp29_diagram_name_nullable_contract() {
    let started = Instant::now();
    let tools = McpService::tools().expect("tools/list 必须可解析");
    let get_diagram = tools
        .iter()
        .find(|t| t["name"] == json!("get_diagram"))
        .expect("必须存在 get_diagram 工具");
    let schema = &get_diagram["outputSchema"];
    // resolve_refs 后 outputSchema 即 DiagramResult → properties.diagram = Diagram
    let diagram_schema = &schema["properties"]["diagram"];
    let required = diagram_schema["required"]
        .as_array()
        .expect("Diagram 必须有 required 数组");
    assert!(
        required.iter().any(|r| r == &json!("name")),
        "name 必须保留在 required 中（键存在，值可空）"
    );
    let name_type = &diagram_schema["properties"]["name"]["type"];
    assert!(
        name_type.as_array().map(|a| a.iter().any(|t| t == &json!("null"))).unwrap_or(false),
        "Diagram.name 类型必须为 [string, null]（存量脏图防御），实际: {name_type}"
    );

    record(&["UT-MCP-29"], started);
}

// ── ST-MCP-14: stdio 端到端——缺 id payload 导入后结构校验 ──────────────────

#[tokio::test(flavor = "multi_thread")]
async fn st_mcp14_stdio_import_missing_ids_then_get() {
    let started = Instant::now();
    // mock 后端：依次应答 import 200、get 200（返回补全后的图）
    let imported_id = "d-import-14";
    let import_result = json!({
        "code":0,
        "data":{"diagram_id":imported_id,"imported_tables":1,"imported_fields":1,
                "warnings":[],"source":"mcp"},
        "request_id":"r-imp"
    });
    let imported_diagram = json!({
        "id":imported_id,"name":"imported_diagram","revision":1,
        "tables":[{"id":"auto-aaa","name":"t_ai","x":0.0,"y":0.0,
                   "fields":[{"id":"auto-bbb","name":"f_ai","type":"INT"}]}],
        "references":[]
    });
    let (base, captured) = scripted_backend(vec![
        import_result,
        json!({"code":0,"data":imported_diagram,"request_id":"r-get"}),
    ])
    .await;

    let mut child = Command::new(env!("CARGO_BIN_EXE_coldrawdb-mcp"))
        .env("COLDRAWDB_BASE_URL", &base)
        // diagram-api-auth：剧本式 mock 只按序应答工具调用，关闭启动期鉴权探测
        .env("COLDRAWDB_AUTH_PROBE", "off")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("spawn coldrawdb-mcp");
    let mut stdin = child.stdin.take().unwrap();
    let stdout = child.stdout.take().unwrap();
    let mut reader = BufReader::new(stdout);

    let import_req = json!({
        "jsonrpc":"2.0","id":1,"method":"tools/call",
        "params":{"name":"import_schema","arguments":{"format":"drawdb_json","payload":{
            "tables":[{"name":"t_ai","fields":[{"name":"f_ai","type":"INT"}]}]}}}
    });
    stdin.write_all(format!("{import_req}\n").as_bytes()).unwrap();
    let mut line = String::new();
    reader.read_line(&mut line).unwrap();
    let import_resp: Value = serde_json::from_str(&line).unwrap();
    assert_eq!(import_resp["result"]["isError"], json!(false));
    assert_eq!(
        import_resp["result"]["structuredContent"]["diagram_id"],
        json!(imported_id)
    );

    let get_req = json!({
        "jsonrpc":"2.0","id":2,"method":"tools/call",
        "params":{"name":"get_diagram","arguments":{"id":imported_id}}
    });
    stdin.write_all(format!("{get_req}\n").as_bytes()).unwrap();
    let mut line2 = String::new();
    reader.read_line(&mut line2).unwrap();
    let get_resp: Value = serde_json::from_str(&line2).unwrap();
    let sc = &get_resp["result"]["structuredContent"];
    assert!(sc["diagram"].is_object(), "structuredContent 必须含 diagram");
    assert_eq!(sc["diagram"]["name"], json!("imported_diagram"));

    // adapter 发出的 POST body：name 兜底 + 表/字段 id 补全（mock 捕获复核）
    let requests = captured.lock().unwrap();
    assert_eq!(requests.len(), 2);
    let sent = &requests[0].2["payload"];
    assert_eq!(sent["name"], json!("imported_diagram"));
    assert!(sent["tables"][0]["id"].as_str().unwrap_or("").starts_with("auto-"));
    assert!(sent["tables"][0]["fields"][0]["id"].as_str().unwrap_or("").starts_with("auto-"));

    drop(stdin);
    let _ = child.wait();

    record(&["ST-MCP-14"], started);
}

// ── UT-MCP-50: 全工具 schema 客户端兼容守卫（拍平回归锚点）──────────────────

/// 递归断言 schema 树中不存在客户端不兼容的条件关键字。
/// Claude Code 在工具转换阶段会丢弃含这些关键字的工具（历史回归：
/// fix-issues-70-73 的 allOf/if/then/else/oneOf/not/anyOf 使 update_table /
/// update_reference 从工具清单消失，仅剩 10/12）。整树检查强于契约的
/// 「顶层禁用」底线——resolve_refs 内联后的深层结构同样不得漏网。
fn assert_no_conditional_keys(value: &Value, path: &str) {
    if let Value::Object(map) = value {
        for (key, child) in map {
            assert!(
                !matches!(
                    key.as_str(),
                    "allOf" | "oneOf" | "anyOf" | "if" | "then" | "else" | "not"
                ),
                "{path}.{key} 使用了客户端不兼容的条件关键字（工具会被 Claude Code 丢弃）"
            );
            assert_no_conditional_keys(child, &format!("{path}.{key}"));
        }
    } else if let Value::Array(items) = value {
        for (index, item) in items.iter().enumerate() {
            assert_no_conditional_keys(item, &format!("{path}[{index}]"));
        }
    }
}

#[tokio::test]
async fn ut_mcp50_tool_schema_client_compat_guard() {
    let started = Instant::now();
    let tools = McpService::tools().expect("tools/list 必须可解析");

    // 断言 4：工具总数 14（fix-mcp-tool-gaps-issue76 新增 update_area/update_note），
    // 且含曾被客户端丢弃的两个工具（回归锚点：不得再退化为 10）
    assert_eq!(tools.len(), 14, "工具清单必须为 14 个，实际: {}", tools.len());
    let names: Vec<&str> = tools.iter().filter_map(|t| t["name"].as_str()).collect();
    assert!(names.contains(&"update_table"), "update_table 必须在工具清单中");
    assert!(names.contains(&"update_reference"), "update_reference 必须在工具清单中");
    assert!(names.contains(&"update_area"), "update_area 必须在工具清单中");
    assert!(names.contains(&"update_note"), "update_note 必须在工具清单中");

    // 断言 1：全工具 inputSchema/outputSchema 整树无条件关键字
    for tool in &tools {
        let name = tool["name"].as_str().unwrap_or("<unknown>");
        for field in ["inputSchema", "outputSchema"] {
            if !tool[field].is_null() {
                assert_no_conditional_keys(&tool[field], &format!("{name}.{field}"));
            }
        }
    }

    // 断言 2：update_table 拍平后关键属性保留，fields.items 的 $ref 已解析
    let update_table = tools
        .iter()
        .find(|t| t["name"] == json!("update_table"))
        .expect("必须存在 update_table 工具");
    let table_props = &update_table["inputSchema"]["properties"];
    for key in ["id", "table_id", "action", "name", "fields"] {
        assert!(
            table_props.get(key).is_some(),
            "update_table.inputSchema.properties 缺少 {key}"
        );
    }
    let field_items = &table_props["fields"]["items"];
    assert!(
        field_items.get("$ref").is_none(),
        "fields.items 的 $ref 必须已被 resolve_refs 内联，实际: {field_items}"
    );
    assert!(
        field_items["properties"].get("name").is_some()
            && field_items["properties"].get("type").is_some(),
        "fields.items 解析后必须含 name/type 属性，实际: {field_items}"
    );

    // 断言 3：update_reference 拍平后关键属性保留，references.items 解析后含端点字段
    let update_reference = tools
        .iter()
        .find(|t| t["name"] == json!("update_reference"))
        .expect("必须存在 update_reference 工具");
    let ref_props = &update_reference["inputSchema"]["properties"];
    for key in [
        "id",
        "action",
        "ref_id",
        "references",
        "start_table_id",
        "start_field_id",
        "end_table_id",
        "end_field_id",
        "cardinality",
        "on_delete",
        "on_update",
    ] {
        assert!(
            ref_props.get(key).is_some(),
            "update_reference.inputSchema.properties 缺少 {key}"
        );
    }
    let reference_items = &ref_props["references"]["items"];
    assert!(
        reference_items.get("$ref").is_none(),
        "references.items 的 $ref 必须已被 resolve_refs 内联，实际: {reference_items}"
    );
    assert!(
        reference_items["properties"].get("start_table_id").is_some(),
        "references.items 解析后必须含 start_table_id 端点属性，实际: {reference_items}"
    );

    record(&["UT-MCP-50"], started);
}

// ── UT-MCP-51: update_table action=delete 删表与引用清理（fix-mcp-tool-gaps-issue76）──

#[tokio::test]
async fn ut_mcp51_update_table_delete_cleans_references() {
    let started = Instant::now();
    // GIVEN：t1（f1/f2）+ t2，r1: t1→t2，r2: t2→t1
    let get = json!({
        "id":"d1","name":"删除契约","revision":5,
        "tables":[
            {"id":"t1","name":"users","x":0.0,"y":0.0,
             "fields":[{"id":"f1","name":"id","type":"INT"},{"id":"f2","name":"name","type":"TEXT"}]},
            {"id":"t2","name":"posts","x":10.0,"y":0.0,
             "fields":[{"id":"f3","name":"id","type":"INT"}]}
        ],
        "references":[
            {"id":"r1","start_table_id":"t1","start_field_id":"f1",
             "end_table_id":"t2","end_field_id":"f3","cardinality":"one_to_many"},
            {"id":"r2","start_table_id":"t2","start_field_id":"f3",
             "end_table_id":"t1","end_field_id":"f1","cardinality":"many_to_one"}
        ]
    });

    // 1) delete 成功：PUT 载荷只剩 t2、references 清空；响应含 deleted_* 清单
    let (base, captured) = get_put_backend(get.clone(), fat_put_data()).await;
    let resp = service(base)
        .call("update_table", json!({"id":"d1","action":"delete","table_id":"t1"}))
        .await
        .unwrap();
    assert_eq!(resp["deleted_table_id"], json!("t1"));
    assert_eq!(
        resp["deleted_reference_ids"],
        json!(["r1", "r2"]),
        "触碰 t1 的 r1/r2 必须被顺带清理"
    );
    {
        let requests = captured.lock().unwrap();
        let put = requests
            .iter()
            .find(|(method, _, _)| method == "PUT")
            .expect("delete 必须发出一次 PUT");
        let tables = put.2["diagram"]["tables"].as_array().unwrap();
        assert_eq!(tables.len(), 1, "PUT 载荷只剩 t2");
        assert_eq!(tables[0]["id"], json!("t2"));
        let references = put.2["diagram"]["references"].as_array().unwrap();
        assert!(references.is_empty(), "PUT 载荷 references 必须为空");
    }

    // 2) delete 携带 name → VALIDATION_ERROR，零写入（无 PUT）
    let (base, captured) = get_put_backend(get.clone(), fat_put_data()).await;
    let error = service(base)
        .call("update_table", json!({"id":"d1","action":"delete","table_id":"t1","name":"x"}))
        .await
        .unwrap_err();
    assert_eq!(error.code, "VALIDATION_ERROR");
    assert!(
        captured.lock().unwrap().iter().all(|(method, _, _)| method != "PUT"),
        "禁字段必须零写入"
    );

    // 3) delete 目标表不存在 → VALIDATION_ERROR，零写入
    let (base, captured) = get_put_backend(get, fat_put_data()).await;
    let error = service(base)
        .call("update_table", json!({"id":"d1","action":"delete","table_id":"ghost"}))
        .await
        .unwrap_err();
    assert_eq!(error.code, "VALIDATION_ERROR");
    assert!(
        captured.lock().unwrap().iter().all(|(method, _, _)| method != "PUT"),
        "目标不存在必须零写入"
    );

    record(&["UT-MCP-51"], started);
}

// ── UT-MCP-52: update_area create/update/delete 全动作（fix-mcp-tool-gaps-issue76）──

#[tokio::test]
async fn ut_mcp52_update_area_crud() {
    let started = Instant::now();

    // 1) create：必填 name，缺省字段落默认；服务端重键，响应 area_id 为实际 ID
    let (base, captured) =
        get_put_backend(json!({"id":"d1","name":"契约","revision":5,"tables":[],"references":[]}), fat_put_data())
            .await;
    let resp = service(base)
        .call(
            "update_area",
            json!({"id":"d1","action":"create","area_id":"a-raw","name":"核心域","x":10.0,"y":20.0,"width":100.0}),
        )
        .await
        .unwrap();
    let actual_area_id = resp["area_id"].as_str().unwrap_or_default().to_string();
    assert!(actual_area_id.starts_with("auto-"), "服务端必须重键，实际: {actual_area_id}");
    {
        let requests = captured.lock().unwrap();
        let put = requests.iter().find(|(m, _, _)| m == "PUT").unwrap();
        let areas = put.2["diagram"]["areas"].as_array().unwrap();
        assert_eq!(areas.len(), 1);
        assert_eq!(areas[0]["id"], json!(actual_area_id));
        assert_eq!(areas[0]["name"], json!("核心域"));
        assert_eq!(areas[0]["x"], json!(10.0));
        assert_eq!(areas[0]["y"], json!(20.0));
        assert_eq!(areas[0]["width"], json!(100.0));
        assert_eq!(areas[0]["height"], json!(0.0), "未携带 height 落默认 0");
        assert_eq!(areas[0]["locked"], json!(false), "未携带 locked 落默认 false");
    }

    // 2) update：改 name+width，未携带的 x/color/locked 保持原值
    let area_fixture = json!({
        "id":"d1","name":"契约","revision":5,"tables":[],"references":[],
        "areas":[{"id":"a1","name":"旧名","x":1.0,"y":2.0,"width":50.0,"height":60.0,"color":"#0f0","locked":true}]
    });
    let (base, captured) = get_put_backend(area_fixture.clone(), fat_put_data()).await;
    let resp = service(base)
        .call("update_area", json!({"id":"d1","area_id":"a1","name":"新名","width":120.0}))
        .await
        .unwrap();
    {
        let mut keys: Vec<&str> = resp.as_object().unwrap().keys().map(String::as_str).collect();
        keys.sort();
        assert_eq!(keys, ["id", "revision"], "update 响应必须恰好只含 id/revision");
        let requests = captured.lock().unwrap();
        let put = requests.iter().find(|(m, _, _)| m == "PUT").unwrap();
        let area = &put.2["diagram"]["areas"][0];
        assert_eq!(area["name"], json!("新名"));
        assert_eq!(area["width"], json!(120.0));
        assert_eq!(area["x"], json!(1.0), "未携带的 x 必须保持原值");
        assert_eq!(area["locked"], json!(true), "未携带的 locked 必须保持原值");
    }

    // 3) delete：区域消失；禁字段与不存在均 VALIDATION_ERROR 零写入
    let (base, captured) = get_put_backend(area_fixture.clone(), fat_put_data()).await;
    service(base)
        .call("update_area", json!({"id":"d1","action":"delete","area_id":"a1"}))
        .await
        .unwrap();
    {
        let requests = captured.lock().unwrap();
        let put = requests.iter().find(|(m, _, _)| m == "PUT").unwrap();
        assert!(put.2["diagram"]["areas"].as_array().unwrap().is_empty());
    }
    let (base, captured) = get_put_backend(area_fixture.clone(), fat_put_data()).await;
    let error = service(base)
        .call("update_area", json!({"id":"d1","action":"delete","area_id":"a1","name":"x"}))
        .await
        .unwrap_err();
    assert_eq!(error.code, "VALIDATION_ERROR");
    assert!(captured.lock().unwrap().iter().all(|(m, _, _)| m != "PUT"));
    let (base, captured) = get_put_backend(area_fixture, fat_put_data()).await;
    let error = service(base)
        .call("update_area", json!({"id":"d1","action":"delete","area_id":"ghost"}))
        .await
        .unwrap_err();
    assert_eq!(error.code, "VALIDATION_ERROR");
    assert!(captured.lock().unwrap().iter().all(|(m, _, _)| m != "PUT"));

    record(&["UT-MCP-52"], started);
}

// ── UT-MCP-53: update_note create/update/delete 全动作（fix-mcp-tool-gaps-issue76）──

#[tokio::test]
async fn ut_mcp53_update_note_crud() {
    let started = Instant::now();

    // 1) create：必填 content，服务端重键，响应 note_id；PUT 载荷含新便签
    let (base, captured) =
        get_put_backend(json!({"id":"d1","name":"契约","revision":5,"tables":[],"references":[]}), fat_put_data())
            .await;
    let resp = service(base)
        .call(
            "update_note",
            json!({"id":"d1","action":"create","note_id":"n-raw","content":"评审意见","x":5.0,"color":"#ff0"}),
        )
        .await
        .unwrap();
    let actual_note_id = resp["note_id"].as_str().unwrap_or_default().to_string();
    assert!(actual_note_id.starts_with("auto-"), "服务端必须重键，实际: {actual_note_id}");
    {
        let requests = captured.lock().unwrap();
        let put = requests.iter().find(|(m, _, _)| m == "PUT").unwrap();
        let notes = put.2["diagram"]["notes"].as_array().unwrap();
        assert_eq!(notes.len(), 1);
        assert_eq!(notes[0]["id"], json!(actual_note_id));
        assert_eq!(notes[0]["content"], json!("评审意见"));
        assert_eq!(notes[0]["color"], json!("#ff0"));
        assert_eq!(notes[0]["y"], json!(0.0));
    }

    // 2) update：改 content+color，x/y 保持原值
    let note_fixture = json!({
        "id":"d1","name":"契约","revision":5,"tables":[],"references":[],
        "notes":[{"id":"n1","content":"旧便签","x":1.5,"y":2.5,"color":"#fff"}]
    });
    let (base, captured) = get_put_backend(note_fixture.clone(), fat_put_data()).await;
    let resp = service(base)
        .call("update_note", json!({"id":"d1","note_id":"n1","content":"新意见","color":"#000"}))
        .await
        .unwrap();
    {
        let mut keys: Vec<&str> = resp.as_object().unwrap().keys().map(String::as_str).collect();
        keys.sort();
        assert_eq!(keys, ["id", "revision"], "update 响应必须恰好只含 id/revision");
        let requests = captured.lock().unwrap();
        let put = requests.iter().find(|(m, _, _)| m == "PUT").unwrap();
        let note = &put.2["diagram"]["notes"][0];
        assert_eq!(note["content"], json!("新意见"));
        assert_eq!(note["color"], json!("#000"));
        assert_eq!(note["x"], json!(1.5), "未携带的 x 必须保持原值");
        assert_eq!(note["y"], json!(2.5), "未携带的 y 必须保持原值");
    }

    // 3) delete：便签消失；禁字段与不存在均 VALIDATION_ERROR 零写入
    let (base, captured) = get_put_backend(note_fixture.clone(), fat_put_data()).await;
    service(base)
        .call("update_note", json!({"id":"d1","action":"delete","note_id":"n1"}))
        .await
        .unwrap();
    {
        let requests = captured.lock().unwrap();
        let put = requests.iter().find(|(m, _, _)| m == "PUT").unwrap();
        assert!(put.2["diagram"]["notes"].as_array().unwrap().is_empty());
    }
    let (base, captured) = get_put_backend(note_fixture.clone(), fat_put_data()).await;
    let error = service(base)
        .call("update_note", json!({"id":"d1","action":"delete","note_id":"n1","content":"x"}))
        .await
        .unwrap_err();
    assert_eq!(error.code, "VALIDATION_ERROR");
    assert!(captured.lock().unwrap().iter().all(|(m, _, _)| m != "PUT"));
    let (base, captured) = get_put_backend(note_fixture, fat_put_data()).await;
    let error = service(base)
        .call("update_note", json!({"id":"d1","action":"delete","note_id":"ghost"}))
        .await
        .unwrap_err();
    assert_eq!(error.code, "VALIDATION_ERROR");
    assert!(captured.lock().unwrap().iter().all(|(m, _, _)| m != "PUT"));

    record(&["UT-MCP-53"], started);
}

// ── UT-MCP-54: update_diagram 防误删守护 allow_mass_delete（fix-mcp-tool-gaps-issue76）──

#[tokio::test]
async fn ut_mcp54_update_diagram_mass_delete_guard() {
    let started = Instant::now();
    // GIVEN：base 3 表 t1/t2/t3（revision 5）
    let base_doc = json!({
        "id":"d1","name":"守护契约","revision":5,
        "tables":[
            {"id":"t1","name":"a","fields":[]},
            {"id":"t2","name":"b","fields":[]},
            {"id":"t3","name":"c","fields":[]}
        ],
        "references":[]
    });

    // 1) desired 缺 1/3（≤50% 且 ≤10）→ 正常放行（GET + PUT）
    let desired_partial = json!({
        "id":"d1","revision":5,
        "tables":[{"id":"t1","name":"a","fields":[]},{"id":"t3","name":"c","fields":[]}],
        "references":[]
    });
    let (base, captured) = get_put_backend(base_doc.clone(), fat_put_data()).await;
    let _ = service(base)
        .call(
            "update_diagram",
            json!({"id":"d1","expected_revision":5,"diagram":desired_partial}),
        )
        .await
        .unwrap();
    assert!(captured.lock().unwrap().iter().any(|(m, _, _)| m == "PUT"));

    // 2) desired 缺 2/3（>50%）且无 allow_mass_delete → VALIDATION_ERROR 零写入
    let desired_mass = json!({
        "id":"d1","revision":5,
        "tables":[{"id":"t1","name":"a","fields":[]}],
        "references":[]
    });
    let (base, captured) = scripted_backend(vec![json!({"code":0,"data":base_doc.clone()})]).await;
    let error = service(base)
        .call(
            "update_diagram",
            json!({"id":"d1","expected_revision":5,"diagram":desired_mass.clone()}),
        )
        .await
        .unwrap_err();
    assert_eq!(error.code, "VALIDATION_ERROR", "守护必须本地拦截");
    assert_eq!(error.details.as_ref().and_then(|d| d["missing_count"].as_i64()), Some(2));
    assert!(captured.lock().unwrap().iter().all(|(m, _, _)| m != "PUT"), "拦截必须零写入");

    // 3) 同 desired 但 allow_mass_delete=true → 放行执行（GET + PUT）
    let (base, captured) = get_put_backend(base_doc, fat_put_data()).await;
    let _ = service(base)
        .call(
            "update_diagram",
            json!({
                "id":"d1","expected_revision":5,
                "diagram":desired_mass,
                "allow_mass_delete":true
            }),
        )
        .await
        .unwrap();
    assert!(captured.lock().unwrap().iter().any(|(m, _, _)| m == "PUT"));

    record(&["UT-MCP-54"], started);
}
