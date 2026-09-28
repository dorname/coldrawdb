//! fix-issues-42-44-mcp-and-relation-hit（#42）：update_diagram 错误可诊断 +
//! USE_OP_CHANNEL 非 JSON 鲁棒识别 + import_schema 仅新建文案（core-S06 §4.5）
//!
//! - UT-MCP-36：409 非 JSON 含 USE_OP_CHANNEL 字样 → 仍走 WS op 通道回退（R-MCPERR-02）
//! - UT-MCP-37：502 HTML 网关错误 → details 含 http_status/content_type/body_excerpt（R-MCPERR-01）
//! - UT-MCP-38：import_schema tool 描述含「仅新建/不覆盖已有 id」（R-MCPERR-04）

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Instant;

use coldrawdb_mcp::api::ApiClient;
use coldrawdb_mcp::reporter;
use coldrawdb_mcp::{Config, McpService};
use serde_json::json;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;

fn record(ids: &[&str], started: Instant) {
    for id in ids {
        reporter::report(id, Ok(()), started.elapsed().as_millis());
    }
}

fn config(base_url: String) -> Config {
    Config::from_values(HashMap::from([("COLDRAWDB_BASE_URL".into(), base_url)])).unwrap()
}

/// 房间图 op 通道回退需要 access token（write_room_diagram 前置校验）
fn config_with_token(base_url: String) -> Config {
    Config::from_values(HashMap::from([
        ("COLDRAWDB_BASE_URL".into(), base_url),
        ("COLDRAWDB_ACCESS_TOKEN".into(), "test-token".into()),
    ]))
    .unwrap()
}

/// 依次服务多个原始响应（状态/Content-Type/body 任意，可非 JSON），并记录请求行
async fn mock_raw(
    responses: Vec<(&'static str, &'static str, &'static str)>,
    requests: Arc<Mutex<Vec<String>>>,
) -> String {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    tokio::spawn(async move {
        for (status, content_type, body) in responses {
            let (mut stream, _) = listener.accept().await.unwrap();
            let mut buf = vec![0_u8; 65536];
            let n = stream.read(&mut buf).await.unwrap();
            requests
                .lock()
                .unwrap()
                .push(String::from_utf8_lossy(&buf[..n]).to_string());
            let response = format!(
                "HTTP/1.1 {status}\r\nContent-Type: {content_type}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
                body.len()
            );
            stream.write_all(response.as_bytes()).await.unwrap();
        }
    });
    format!("http://{address}")
}

#[tokio::test]
async fn ut_mcp_36_non_json_409_use_op_channel_falls_back_to_op_channel() {
    let started = Instant::now();
    let requests = Arc::new(Mutex::new(Vec::new()));
    // ①PUT 返回 409 非 JSON 但含 USE_OP_CHANNEL 字样（网关/代理改写的文本页场景）
    // ②回退路径 GET /rooms → 空房间列表  ③GET /diagrams/{id} → 物化文档
    let base = mock_raw(
        vec![
            (
                "409 Conflict",
                "text/plain",
                "409 USE_OP_CHANNEL: 房间图表请通过协作 op 通道写入",
            ),
            ("200 OK", "application/json", r#"{"items":[]}"#),
            (
                "200 OK",
                "application/json",
                r#"{"code":0,"data":{"revision":1,"tables":[]}}"#,
            ),
        ],
        requests.clone(),
    )
    .await;
    let api = ApiClient::new(config_with_token(base)).unwrap();
    let error = api
        .update("d1", 1, json!({"tables": [], "references": [], "areas": [], "notes": []}))
        .await
        .unwrap_err();
    // 房间不存在 → plan_room_save 报 NOT_FOUND；关键是**没有**落入笼统 VALIDATION_ERROR，
    // 证明 409 非 JSON 仍被识别为房间图收编并进入了 op 通道回退路径
    assert_eq!(
        error.code, "NOT_FOUND",
        "UT-MCP-36: 409 非 JSON 含 USE_OP_CHANNEL 必须走 op 通道回退（房间缺失才报 NOT_FOUND），实际: {}",
        error.message
    );
    let seen = requests.lock().unwrap();
    assert!(
        seen.first().map(|r| r.starts_with("PUT /api/v1/diagrams/d1")).unwrap_or(false),
        "UT-MCP-36: 首个请求必须为 PUT update_diagram"
    );
    assert!(
        seen.get(1).map(|r| r.starts_with("GET /api/v1/rooms")).unwrap_or(false),
        "UT-MCP-36: 回退路径必须请求 GET /rooms（证明进入 write_room_diagram）"
    );
    record(&["UT-MCP-36"], started);
}

#[tokio::test]
async fn ut_mcp_37_non_json_upstream_error_carries_diagnostics() {
    let started = Instant::now();
    let requests = Arc::new(Mutex::new(Vec::new()));
    let base = mock_raw(
        vec![(
            "502 Bad Gateway",
            "text/html",
            "<html><body><h1>502 Bad Gateway</h1>nginx</body></html>",
        )],
        requests,
    )
    .await;
    let api = ApiClient::new(config(base)).unwrap();
    let error = api
        .update("d1", 1, json!({"tables": []}))
        .await
        .unwrap_err();
    assert_eq!(error.code, "UPSTREAM_ERROR", "UT-MCP-37: 5xx 必须为 UPSTREAM_ERROR");
    assert!(error.retryable, "UT-MCP-37: 5xx 必须可重试");
    let details = error.details.expect("UT-MCP-37: 错误必须携带 details");
    assert_eq!(details["http_status"], 502, "UT-MCP-37: details.http_status 必须为 502");
    assert!(
        details["content_type"].as_str().unwrap_or("").contains("text/html"),
        "UT-MCP-37: details.content_type 必须如实记录 text/html"
    );
    assert!(
        details["body_excerpt"].as_str().unwrap_or("").contains("502 Bad Gateway"),
        "UT-MCP-37: details.body_excerpt 必须含 body 摘要（可辨网关错误页）"
    );
    assert!(
        error.message.contains("502"),
        "UT-MCP-37: message 必须含 HTTP 状态（不再只有「非 JSON」）"
    );
    record(&["UT-MCP-37"], started);
}

#[test]
fn ut_mcp_38_import_schema_description_create_only() {
    let started = Instant::now();
    let tools = McpService::tools().unwrap();
    let import = tools
        .iter()
        .find(|t| t.get("name").and_then(serde_json::Value::as_str) == Some("import_schema"))
        .expect("UT-MCP-38: import_schema 工具必须存在");
    let description = import
        .get("description")
        .and_then(serde_json::Value::as_str)
        .unwrap_or("");
    assert!(
        description.contains("仅新建") && description.contains("不覆盖已有 id"),
        "UT-MCP-38: import_schema 描述必须明确「仅新建，不覆盖已有 id」，实际: {description}"
    );
    record(&["UT-MCP-38"], started);
}
