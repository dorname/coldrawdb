//! fix-open-issues-26-33（2026-09-27，issue #26）：update_diagram 参数体积软上限与分批写入
//! 覆盖 UT-MCP-30 / UT-MCP-31 / UT-MCP-32 / ST-MCP-15 / ST-MCP-16：
//! - 体积恰 ≤ 软上限直写；超限本地拦截 PAYLOAD_TOO_LARGE（details 含体积/上限/分片建议），零上游请求；
//! - COLDRAWDB_MCP_PAYLOAD_SOFT_LIMIT_BYTES 配置生效，非法值回退 256 KiB 并 warn；
//! - batch_hint 按主导实体推荐 update_table / update_reference / layout_diagram；
//! - stdio 端到端：超限拦截后按官方分批指引串行写成功、revision 连续递增；
//! - 分批链路单批 409 → REVISION_CONFLICT、已成功批不回滚、最新 revision 重试成功。

use std::collections::HashMap;
use std::io::{BufRead, BufReader, Read, Write};
use std::process::{Command, Stdio};
use std::sync::{Arc, Mutex};
use std::time::Instant;

use coldrawdb_mcp::api::ApiClient;
use coldrawdb_mcp::payload_limit::{
    self, DEFAULT_SOFT_LIMIT_BYTES, MIN_SOFT_LIMIT_BYTES,
};
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

fn service_with(base_url: String, soft_limit: Option<&str>) -> McpService {
    let mut values = HashMap::from([
        ("COLDRAWDB_BASE_URL".into(), base_url),
        ("COLDRAWDB_REQUEST_TIMEOUT_SECS".into(), "5".into()),
    ]);
    if let Some(limit) = soft_limit {
        values.insert(
            payload_limit::ENV_SOFT_LIMIT.into(),
            limit.into(),
        );
    }
    let config = Config::from_values(values).unwrap();
    McpService::new(ApiClient::new(config).unwrap())
}

// ── mock 后端（沿用 c3/c5/c6 的手工 TcpListener 模式，响应携带状态码）──────

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

async fn write_response(stream: &mut TcpStream, status: u16, body: Value) {
    let reason = match status {
        200 => "OK",
        409 => "Conflict",
        _ => "Status",
    };
    let payload = body.to_string();
    let response = format!(
        "HTTP/1.1 {status} {reason}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{payload}",
        payload.len()
    );
    stream.write_all(response.as_bytes()).await.unwrap();
}

/// 剧本式 mock 后端：每个连接按顺序应答一个 (status, body)，捕获全部 (method, path, body)。
async fn scripted_backend(
    responses: Vec<(u16, Value)>,
) -> (String, Arc<Mutex<Vec<(String, String, Value)>>>) {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let captured = Arc::new(Mutex::new(Vec::new()));
    let requests = captured.clone();
    tokio::spawn(async move {
        for (status, response) in responses {
            let (mut stream, _) = listener.accept().await.unwrap();
            let (method, path, body) = read_request(&mut stream).await;
            requests
                .lock()
                .unwrap()
                .push((method, path, body));
            write_response(&mut stream, status, response).await;
        }
    });
    (format!("http://{address}"), captured)
}

/// 构造序列化体积恰为 target 字节的 diagram（pad 字段填充）。
fn padded_diagram(target: usize) -> Value {
    let mut diagram = json!({"id":"d1","revision":5,"tables":[],"references":[],"pad":""});
    let fixed = payload_limit::estimate_payload_bytes(&diagram);
    assert!(target >= fixed, "target 必须不小于固定部分体积 {fixed}");
    diagram["pad"] = json!("x".repeat(target - fixed));
    assert_eq!(payload_limit::estimate_payload_bytes(&diagram), target);
    diagram
}

// ── UT-MCP-30: 软上限拦截 + PAYLOAD_TOO_LARGE 结构 + 零请求 ────────────────

#[tokio::test]
async fn ut_mcp30_soft_limit_intercept() {
    let started = Instant::now();
    let limit = MIN_SOFT_LIMIT_BYTES;

    // 1) 体积恰 == 软上限 → 直写不拦截
    let (base, captured) = scripted_backend(vec![(
        200,
        json!({"code":0,"data":{"id":"d1","revision":6}}),
    )])
    .await;
    let resp = service_with(base, Some("16384"))
        .call(
            "update_diagram",
            json!({"id":"d1","expected_revision":5,"diagram":padded_diagram(limit)}),
        )
        .await
        .unwrap();
    assert_eq!(resp, json!({"id":"d1","revision":6}));
    assert_eq!(
        captured.lock().unwrap().len(),
        1,
        "体积恰达软上限必须直写（恰好 1 个 PUT）"
    );

    // 2) 体积 == 上限 + 1 → 本地拦截，mock 断言零请求
    let (base, captured) = scripted_backend(vec![]).await;
    let error = service_with(base, Some("16384"))
        .call(
            "update_diagram",
            json!({"id":"d1","expected_revision":5,"diagram":padded_diagram(limit + 1)}),
        )
        .await
        .unwrap_err();
    assert_eq!(error.code, "PAYLOAD_TOO_LARGE");
    assert!(!error.retryable, "PAYLOAD_TOO_LARGE 必须 retryable=false");
    let details = error.details.as_ref().expect("details 必须存在");
    assert_eq!(details["payload_bytes"], json!(limit + 1));
    assert_eq!(details["soft_limit_bytes"], json!(limit));
    assert!(
        details["suggestion"]
            .as_str()
            .unwrap_or("")
            .contains("update_table"),
        "suggestion 必须包含分批指引"
    );
    assert!(details["batch_hint"].is_object(), "details 必须含 batch_hint");
    assert!(
        captured.lock().unwrap().is_empty(),
        "超限拦截必须零上游请求（diagram 数据不变）"
    );

    record(&["UT-MCP-30"], started);
}

// ── UT-MCP-31: 软上限环境变量配置生效与非法值回退 ──────────────────────────

#[test]
fn ut_mcp31_soft_limit_config() {
    let started = Instant::now();

    // 未配置 / 空白 → 默认 256 KiB，无 warn
    assert_eq!(
        payload_limit::parse_soft_limit(None),
        (DEFAULT_SOFT_LIMIT_BYTES, None)
    );
    assert_eq!(
        payload_limit::parse_soft_limit(Some("  ")),
        (DEFAULT_SOFT_LIMIT_BYTES, None)
    );
    // 合法覆盖（≥ 16 KiB 整数）
    assert_eq!(
        payload_limit::parse_soft_limit(Some("131072")),
        (131072, None)
    );
    assert_eq!(
        payload_limit::parse_soft_limit(Some("16384")),
        (MIN_SOFT_LIMIT_BYTES, None)
    );
    // 非法值（< 16 KiB / 非整数）→ 回退默认 + warn
    for raw in ["1024", "16383", "abc", "12.5", "-1"] {
        let (value, warn) = payload_limit::parse_soft_limit(Some(raw));
        assert_eq!(value, DEFAULT_SOFT_LIMIT_BYTES, "{raw} 必须回退默认 256 KiB");
        let warn = warn.unwrap_or_else(|| panic!("{raw} 必须产生 warn"));
        assert!(warn.contains(payload_limit::ENV_SOFT_LIMIT), "warn 须含变量名");
    }

    // Config 集成：环境变量值进入生效配置
    let config_limit = |raw: Option<&str>| {
        let mut values = HashMap::from([(
            "COLDRAWDB_BASE_URL".to_string(),
            "http://127.0.0.1:9".to_string(),
        )]);
        if let Some(raw) = raw {
            values.insert(payload_limit::ENV_SOFT_LIMIT.to_string(), raw.to_string());
        }
        Config::from_values(values).unwrap().payload_soft_limit_bytes
    };
    assert_eq!(config_limit(None), DEFAULT_SOFT_LIMIT_BYTES);
    assert_eq!(config_limit(Some("131072")), 131072);
    assert_eq!(config_limit(Some("1024")), DEFAULT_SOFT_LIMIT_BYTES);
    assert_eq!(config_limit(Some("abc")), DEFAULT_SOFT_LIMIT_BYTES);

    // 超限错误 retryable=false
    assert!(!payload_limit::payload_too_large_error(1, 0, &json!({})).retryable);

    record(&["UT-MCP-31"], started);
}

// ── UT-MCP-32: batch_hint 构成建议与计数 ──────────────────────────────────

#[test]
fn ut_mcp32_batch_hint() {
    let started = Instant::now();

    // 表为主 → update_table，计数与 diagram 实际一致
    let hint = payload_limit::batch_hint(&json!({"tables":[{},{},{}],"references":[{}]}));
    assert_eq!(hint["suggested_batch"], json!("update_table"));
    assert_eq!(hint["tables"], json!(3));
    assert_eq!(hint["references"], json!(1));

    // 关系为主 → update_reference
    let hint = payload_limit::batch_hint(&json!({"tables":[{}],"references":[{},{},{}]}));
    assert_eq!(hint["suggested_batch"], json!("update_reference"));
    assert_eq!(hint["tables"], json!(1));
    assert_eq!(hint["references"], json!(3));

    // 纯布局（无表无关系）→ layout_diagram
    for diagram in [
        json!({}),
        json!({"tables":[],"references":[]}),
        json!({"notes":[{}]}),
    ] {
        assert_eq!(
            payload_limit::batch_hint(&diagram)["suggested_batch"],
            json!("layout_diagram"),
            "{diagram} 必须建议 layout_diagram"
        );
    }

    // 经 payload_too_large_error 的 details.batch_hint 计数一致
    let error = payload_limit::payload_too_large_error(
        999,
        100,
        &json!({"tables":[{},{}],"references":[{},{},{},{}]}),
    );
    let hint = &error.details.as_ref().unwrap()["batch_hint"];
    assert_eq!(hint["tables"], json!(2));
    assert_eq!(hint["references"], json!(4));
    assert_eq!(hint["suggested_batch"], json!("update_reference"));

    record(&["UT-MCP-32"], started);
}

// ── stdio 辅助（沿用 ST-MCP-14 模式：直接 tools/call，逐行一应一答）────────

fn spawn_mcp(base_url: &str, soft_limit: Option<&str>) -> std::process::Child {
    let mut command = Command::new(env!("CARGO_BIN_EXE_coldrawdb-mcp"));
    command
        .env("COLDRAWDB_BASE_URL", base_url)
        // diagram-api-auth：剧本式 mock 只按序应答工具调用，关闭启动期鉴权探测
        .env("COLDRAWDB_AUTH_PROBE", "off")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null());
    if let Some(limit) = soft_limit {
        command.env(payload_limit::ENV_SOFT_LIMIT, limit);
    }
    command.spawn().expect("spawn coldrawdb-mcp")
}

fn stdio_call(
    stdin: &mut impl Write,
    reader: &mut BufReader<impl Read>,
    id: i64,
    name: &str,
    arguments: Value,
) -> Value {
    let request = json!({"jsonrpc":"2.0","id":id,"method":"tools/call",
                         "params":{"name":name,"arguments":arguments}});
    stdin.write_all(format!("{request}\n").as_bytes()).unwrap();
    let mut line = String::new();
    reader.read_line(&mut line).unwrap();
    serde_json::from_str(&line).unwrap()
}

/// 两表（各一字段）的 GET diagram fixture，revision 可指定。
fn two_table_diagram(revision: i64) -> Value {
    json!({
        "id":"d1","name":"分批","revision":revision,
        "tables":[{"id":"t1","name":"users","x":0.0,"y":0.0,
                   "fields":[{"id":"f1","name":"id","type":"INT"}]},
                  {"id":"t2","name":"orders","x":400.0,"y":0.0,
                   "fields":[{"id":"f2","name":"uid","type":"INT"}]}],
        "references":[]
    })
}

// ── ST-MCP-15: 超限拦截 → 按官方分批指引串行写成功 ─────────────────────────

// 使用 multi_thread runtime：mock 后端任务在 worker 线程上运行，
// 主线程可阻塞式读写真实子进程的 stdio 管道而不死锁。
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn st_mcp15_oversize_intercept_then_batched_writes() {
    let started = Instant::now();

    // 剧本：update_table×2 → update_reference×1 → layout×1（各 GET+PUT）→ 最终 GET。
    // PUT 响应 revision 连续递增 6→7→8→9；拦截的 update_diagram 不产生任何请求。
    let mut final_diagram = two_table_diagram(9);
    final_diagram["tables"][0]["name"] = json!("users_v2");
    final_diagram["tables"][1]["name"] = json!("orders_v2");
    final_diagram["references"] = json!([{
        "id":"ref-new","start_table_id":"t1","start_field_id":"f1",
        "end_table_id":"t2","end_field_id":"f2","cardinality":"many_to_one"
    }]);
    let (base, captured) = scripted_backend(vec![
        (200, json!({"code":0,"data":two_table_diagram(5)})),
        (200, json!({"code":0,"data":{"id":"d1","revision":6}})),
        (200, json!({"code":0,"data":two_table_diagram(6)})),
        (200, json!({"code":0,"data":{"id":"d1","revision":7}})),
        (200, json!({"code":0,"data":two_table_diagram(7)})),
        (200, json!({"code":0,"data":{"id":"d1","revision":8}})),
        (200, json!({"code":0,"data":two_table_diagram(8)})),
        (200, json!({"code":0,"data":{"id":"d1","revision":9}})),
        (200, json!({"code":0,"data":final_diagram})),
    ])
    .await;

    let mut child = spawn_mcp(&base, Some("16384"));
    let mut stdin = child.stdin.take().unwrap();
    let mut reader = BufReader::new(child.stdout.take().unwrap());

    // 1) 超软上限整图写入 → 本地拦截，结构可诊断
    let resp = stdio_call(
        &mut stdin,
        &mut reader,
        1,
        "update_diagram",
        json!({"id":"d1","expected_revision":5,"diagram":padded_diagram(20000)}),
    );
    assert_eq!(resp.pointer("/result/isError"), Some(&json!(true)));
    let error = &resp["result"]["structuredContent"];
    assert_eq!(error["code"], json!("PAYLOAD_TOO_LARGE"));
    assert_eq!(error["retryable"], json!(false));
    assert_eq!(error["details"]["payload_bytes"], json!(20000));
    assert_eq!(error["details"]["soft_limit_bytes"], json!(16384));
    assert!(error["details"]["suggestion"].as_str().unwrap_or("").contains("update_table"));
    assert!(error["details"]["batch_hint"].is_object());

    // 2) 按指引分批：update_table × 2 → update_reference × 1 → layout_diagram，
    //    每批携带最新 revision 严格串行 → revision 连续递增 6/7/8/9 无跳号
    let resp = stdio_call(&mut stdin, &mut reader, 2, "update_table",
        json!({"id":"d1","table_id":"t1","name":"users_v2"}));
    assert_eq!(resp.pointer("/result/isError"), Some(&json!(false)));
    assert_eq!(resp["result"]["structuredContent"]["revision"], json!(6));

    let resp = stdio_call(&mut stdin, &mut reader, 3, "update_table",
        json!({"id":"d1","table_id":"t2","name":"orders_v2"}));
    assert_eq!(resp["result"]["structuredContent"]["revision"], json!(7));

    let resp = stdio_call(&mut stdin, &mut reader, 4, "update_reference",
        json!({"id":"d1","action":"create","start_table_id":"t1","start_field_id":"f1",
               "end_table_id":"t2","end_field_id":"f2"}));
    assert_eq!(resp["result"]["structuredContent"]["revision"], json!(8));

    let resp = stdio_call(&mut stdin, &mut reader, 5, "layout_diagram",
        json!({"id":"d1","mode":"force"}));
    let sc = &resp["result"]["structuredContent"];
    assert_eq!(sc["revision"], json!(9));
    assert_eq!(sc["tables_repositioned"], json!(2));

    // 3) get_diagram 验证最终状态与目标变更一致
    let resp = stdio_call(&mut stdin, &mut reader, 6, "get_diagram", json!({"id":"d1"}));
    let diagram = &resp["result"]["structuredContent"]["diagram"];
    assert_eq!(diagram["revision"], json!(9));
    assert_eq!(diagram["tables"][0]["name"], json!("users_v2"));
    assert_eq!(diagram["tables"][1]["name"], json!("orders_v2"));
    assert_eq!(diagram["references"].as_array().unwrap().len(), 1);

    drop(stdin);
    let _ = child.wait();

    // 4) 请求面复核：拦截零请求；分批 4 工具 × (GET+PUT) + 最终 GET = 9 个请求；
    //    PUT 携带的 expected_revision 严格串行 5→6→7→8
    let requests = captured.lock().unwrap();
    assert_eq!(requests.len(), 9, "拦截必须零请求，分批恰好 9 个请求");
    assert_eq!(requests[0].0, "GET", "首个请求必须是分批的 GET（拦截未发请求）");
    for (index, expected) in [(1, 5), (3, 6), (5, 7), (7, 8)] {
        assert_eq!(requests[index].0, "PUT");
        assert_eq!(requests[index].1, "/api/v1/diagrams/d1");
        assert_eq!(
            requests[index].2["expected_revision"],
            json!(expected),
            "第 {index} 个请求 expected_revision 必须串行携带最新值"
        );
    }

    record(&["ST-MCP-15"], started);
}

// ── ST-MCP-16: 分批链路单批 409 重试不损坏 revision ────────────────────────

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn st_mcp16_batched_chain_conflict_retry() {
    let started = Instant::now();

    // 剧本：批1 update_table(t1) GET+PUT 成功（rev5→6）；
    // 批2 update_table(t2) GET(rev6) 后 PUT 409（他方已推进到 rev7）；
    // 重试批2：GET(rev7) + PUT 成功（rev7→8）。
    let (base, captured) = scripted_backend(vec![
        (200, json!({"code":0,"data":two_table_diagram(5)})),
        (200, json!({"code":0,"data":{"id":"d1","revision":6}})),
        (200, json!({"code":0,"data":two_table_diagram(6)})),
        (409, json!({"message":"revision conflict","details":{"current_revision":7}})),
        (200, json!({"code":0,"data":two_table_diagram(7)})),
        (200, json!({"code":0,"data":{"id":"d1","revision":8}})),
    ])
    .await;

    let mut child = spawn_mcp(&base, None);
    let mut stdin = child.stdin.take().unwrap();
    let mut reader = BufReader::new(child.stdout.take().unwrap());

    // 批1：成功
    let resp = stdio_call(&mut stdin, &mut reader, 1, "update_table",
        json!({"id":"d1","table_id":"t1","name":"users_v2"}));
    assert_eq!(resp.pointer("/result/isError"), Some(&json!(false)));
    assert_eq!(resp["result"]["structuredContent"]["revision"], json!(6));

    // 批2：人为 409 → REVISION_CONFLICT，details 给出最新 revision
    let resp = stdio_call(&mut stdin, &mut reader, 2, "update_table",
        json!({"id":"d1","table_id":"t2","name":"orders_v2"}));
    assert_eq!(resp.pointer("/result/isError"), Some(&json!(true)));
    let error = &resp["result"]["structuredContent"];
    assert_eq!(error["code"], json!("REVISION_CONFLICT"));
    assert_eq!(error["retryable"], json!(false));
    assert_eq!(error["details"]["current_revision"], json!(7));

    // 重试批2：以最新 revision 重试 → 成功
    let resp = stdio_call(&mut stdin, &mut reader, 3, "update_table",
        json!({"id":"d1","table_id":"t2","name":"orders_v2"}));
    assert_eq!(resp.pointer("/result/isError"), Some(&json!(false)));
    assert_eq!(resp["result"]["structuredContent"]["revision"], json!(8));

    drop(stdin);
    let _ = child.wait();

    // 请求面复核：已成功批不回滚——expected_revision 单调推进 5→6（冲突）→7，
    // 且无对批1的任何补偿性写请求（总计 6 个请求：3×(GET+PUT)）
    let requests = captured.lock().unwrap();
    assert_eq!(requests.len(), 6);
    assert_eq!(requests[1].0, "PUT");
    assert_eq!(requests[1].2["expected_revision"], json!(5));
    assert_eq!(requests[3].0, "PUT");
    assert_eq!(requests[3].2["expected_revision"], json!(6));
    assert_eq!(requests[5].0, "PUT");
    assert_eq!(requests[5].2["expected_revision"], json!(7));
    // 重试批 PUT 的 diagram.revision 必须基于最新 GET（7），最终落到 8
    assert_eq!(requests[5].2["diagram"]["revision"], json!(7));

    record(&["ST-MCP-16"], started);
}
