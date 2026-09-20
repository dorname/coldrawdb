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

    // 1) update_diagram：上游 PUT 返回含额外字段的 data → 恰好 {id, revision}
    let (base, _) = put_only_backend(fat_put_data()).await;
    let svc = service(base);
    let resp = svc
        .call(
            "update_diagram",
            json!({"id":"d1","expected_revision":5,"diagram":{"id":"d1","revision":5}}),
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
    let (base, _) = put_only_backend(json!({"revision":7,"request_id":"r-no-id"})).await;
    let resp = service(base)
        .call(
            "update_diagram",
            json!({"id":"d1","expected_revision":5,"diagram":{"id":"d1"}}),
        )
        .await
        .unwrap();
    assert_update_output(&resp, "d1", 7);

    // 7) 上游 data 缺 revision → 错误映射，不得产出缺字段的成功响应
    let (base, _) = put_only_backend(json!({"id":"d1","request_id":"r-no-rev"})).await;
    let error = service(base)
        .call(
            "update_diagram",
            json!({"id":"d1","expected_revision":5,"diagram":{"id":"d1"}}),
        )
        .await
        .unwrap_err();
    assert_eq!(error.code, "UPSTREAM_ERROR");

    // 8) revision 非数 → 错误映射
    let (base, _) = put_only_backend(json!({"id":"d1","revision":"6"})).await;
    let error = service(base)
        .call(
            "update_diagram",
            json!({"id":"d1","expected_revision":5,"diagram":{"id":"d1"}}),
        )
        .await
        .unwrap_err();
    assert_eq!(error.code, "UPSTREAM_ERROR");

    // 9) revision = 0（不满足 outputSchema minimum:1）→ 错误映射
    let (base, _) = put_only_backend(json!({"id":"d1","revision":0})).await;
    let error = service(base)
        .call(
            "update_diagram",
            json!({"id":"d1","expected_revision":5,"diagram":{"id":"d1"}}),
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
    assert_eq!(r1["cardinality"], json!("one_to_one"));
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
