use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

use coldrawdb_mcp::api::ApiClient;
use coldrawdb_mcp::collab::tool_log_line;
use coldrawdb_mcp::error::ToolError;
use coldrawdb_mcp::reporter;
use coldrawdb_mcp::Config;
use futures_util::{SinkExt, StreamExt};
use reqwest::StatusCode;
use serde_json::{json, Value};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio_tungstenite::tungstenite::Message;

fn config(base_url: String, token: Option<&str>) -> Config {
    let mut values = HashMap::from([
        ("COLDRAWDB_BASE_URL".into(), base_url),
        ("COLDRAWDB_REQUEST_TIMEOUT_SECS".into(), "5".into()),
    ]);
    if let Some(token) = token {
        values.insert("COLDRAWDB_ACCESS_TOKEN".into(), token.into());
    }
    Config::from_values(values).unwrap()
}

async fn read_http(stream: &mut TcpStream) -> (String, String, Value) {
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

fn http_response(status: &str, body: Value) -> String {
    let payload = body.to_string();
    format!(
        "HTTP/1.1 {status}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{payload}",
        payload.len()
    )
}

fn base_diagram() -> Value {
    json!({
        "id": "d1",
        "revision": 4,
        "tables": [{"id": "t1", "name": "users", "fields": [{"id": "f1", "name": "id"}]}],
        "references": [],
        "areas": [],
        "notes": []
    })
}

async fn room_server(ws_hits: Arc<AtomicUsize>) -> String {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    tokio::spawn(async move {
        loop {
            let (mut stream, _) = listener.accept().await.unwrap();
            let ws_hits = ws_hits.clone();
            tokio::spawn(async move {
                let mut buf = vec![0_u8; 4096];
                let n = loop {
                    let n = stream.peek(&mut buf).await.unwrap_or(0);
                    if n == 0 {
                        return;
                    }
                    if buf[..n].windows(4).any(|window| window == b"\r\n\r\n") || n == buf.len() {
                        break n;
                    }
                    tokio::time::sleep(Duration::from_millis(5)).await;
                };
                let head = String::from_utf8_lossy(&buf[..n]).to_ascii_lowercase();
                if head.contains("upgrade: websocket") {
                    ws_hits.fetch_add(1, Ordering::SeqCst);
                    let mut socket = tokio_tungstenite::accept_async(stream).await.unwrap();
                    while let Some(message) = socket.next().await {
                        let Message::Text(text) = message.unwrap() else {
                            continue;
                        };
                        assert!(!text.contains("secret-token"));
                        let frame: Value = serde_json::from_str(&text).unwrap();
                        assert_eq!(frame["op"]["type"], "table.update");
                        assert!(frame["op"]["changes"].get("fields").is_none());
                        let ack = json!({"type": "ack", "serverRev": 5}).to_string();
                        socket.send(Message::Text(ack.into())).await.unwrap();
                        break;
                    }
                    return;
                }
                let (method, path, _) = read_http(&mut stream).await;
                let (status, body) = match (method.as_str(), path.split('?').next().unwrap()) {
                    ("PUT", "/api/v1/diagrams/d1") => (
                        "409 Conflict",
                        json!({"code":409,"message":"USE_OP_CHANNEL: 房间图表请通过协作 op 通道写入","details":{"code":"USE_OP_CHANNEL"}}),
                    ),
                    ("GET", "/api/v1/rooms") => (
                        "200 OK",
                        json!({"items":[{"id":"r1","diagramId":"d1"}],"total":1}),
                    ),
                    ("GET", "/api/v1/diagrams/d1") => (
                        "200 OK",
                        json!({"code":0,"data": base_diagram()}),
                    ),
                    _ => ("500 Internal Server Error", json!({"message":"unexpected"})),
                };
                stream
                    .write_all(http_response(status, body).as_bytes())
                    .await
                    .unwrap();
            });
        }
    });
    format!("http://{address}")
}

#[test]
fn ut_mcp_25_use_op_channel_is_not_revision_conflict() {
    let started = Instant::now();
    let body = json!({"message":"USE_OP_CHANNEL: 房间图表请通过协作 op 通道写入","details":{"code":"USE_OP_CHANNEL"}});
    let error = ToolError::upstream(StatusCode::CONFLICT, &body);
    assert_eq!(error.code, "USE_OP_CHANNEL");
    assert_ne!(error.code, "REVISION_CONFLICT");
    assert_eq!(error.details.unwrap()["code"], "USE_OP_CHANNEL");

    let conflict = json!({"message":"revision conflict","details":{"current_revision":3}});
    let error = ToolError::upstream(StatusCode::CONFLICT, &conflict);
    assert_eq!(error.code, "REVISION_CONFLICT");
    assert_eq!(error.details.unwrap()["current_revision"], 3);
    reporter::report("UT-MCP-25", Ok(()), started.elapsed().as_millis());
}

#[tokio::test]
async fn ut_mcp_26_room_write_uses_op_channel() {
    let started = Instant::now();
    let ws_hits = Arc::new(AtomicUsize::new(0));
    let base_url = room_server(ws_hits.clone()).await;
    let api = ApiClient::new(config(base_url.clone(), Some("secret-token"))).unwrap();
    let mut desired = base_diagram();
    desired["tables"][0]["name"] = json!("accounts");
    let saved = api.update("d1", 4, desired).await.unwrap();
    assert_eq!(saved["id"], "d1");
    assert_eq!(saved["revision"], 5);
    assert_eq!(ws_hits.load(Ordering::SeqCst), 1);

    let unchanged = api.update("d1", 4, base_diagram()).await.unwrap();
    assert_eq!(unchanged["revision"], 4);
    assert_eq!(ws_hits.load(Ordering::SeqCst), 1);

    let conflict = api.update("d1", 3, base_diagram()).await.unwrap_err();
    assert_eq!(conflict.code, "REVISION_CONFLICT");
    assert_eq!(conflict.details.unwrap()["current_revision"], 4);
    assert_eq!(ws_hits.load(Ordering::SeqCst), 1);

    let anonymous = ApiClient::new(config(base_url, None)).unwrap();
    let missing = anonymous.update("d1", 4, base_diagram()).await.unwrap_err();
    assert_eq!(missing.code, "UNAUTHENTICATED");
    assert!(!missing.message.contains("secret-token"));
    assert!(!missing.message.contains("token="));
    reporter::report("UT-MCP-26", Ok(()), started.elapsed().as_millis());
}

#[test]
fn ut_mcp_27_ok_calls_do_not_log_stderr_line() {
    let started = Instant::now();
    assert!(tool_log_line("ok", "list_diagrams", 7, None, None).is_none());
    assert!(tool_log_line("ok", "get_diagram", 5, None, None).is_none());
    let line = tool_log_line(
        "error",
        "update_diagram",
        4,
        Some("USE_OP_CHANNEL"),
        Some("房间图表请通过协作 op 通道写入"),
    )
    .unwrap();
    assert_eq!(line["status"], "error");
    assert_eq!(line["code"], "USE_OP_CHANNEL");
    assert!(line.get("diagram").is_none());
    assert!(!line.to_string().contains("secret-token"));
    reporter::report("UT-MCP-27", Ok(()), started.elapsed().as_millis());
}
