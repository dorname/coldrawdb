use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use coldrawdb_mcp::api::ApiClient;
use coldrawdb_mcp::error::ToolError;
use coldrawdb_mcp::reporter;
use coldrawdb_mcp::{Config, McpService};
use reqwest::StatusCode;
use serde_json::{json, Value};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

fn record(ids: &[&str], started: Instant) {
    for id in ids {
        reporter::report(id, Ok(()), started.elapsed().as_millis());
    }
}

fn config(base_url: String, token: Option<&str>, timeout: u64) -> Config {
    let mut values = HashMap::from([
        ("COLDRAWDB_BASE_URL".into(), base_url),
        ("COLDRAWDB_REQUEST_TIMEOUT_SECS".into(), timeout.to_string()),
    ]);
    if let Some(token) = token {
        values.insert("COLDRAWDB_ACCESS_TOKEN".into(), token.into());
    }
    Config::from_values(values).unwrap()
}

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

async fn write_response(stream: &mut TcpStream, status: &str, body: Value) {
    let payload = body.to_string();
    let response = format!(
        "HTTP/1.1 {status}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{payload}",
        payload.len()
    );
    stream.write_all(response.as_bytes()).await.unwrap();
}

async fn stateful_backend() -> (String, Arc<Mutex<Vec<(String, String, Value)>>>) {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let requests = Arc::new(Mutex::new(Vec::new()));
    let captured = requests.clone();
    tokio::spawn(async move {
        let mut revision = 0_i64;
        for _ in 0..8 {
            let (mut stream, _) = listener.accept().await.unwrap();
            let (method, path, body) = read_request(&mut stream).await;
            captured
                .lock()
                .unwrap()
                .push((method.clone(), path.clone(), body.clone()));
            let (status, response) = match (method.as_str(), path.as_str()) {
                ("POST", "/api/v1/diagrams") => (
                    "200 OK",
                    json!({"code":0,"data":{"id":"d1"},"request_id":"r-create"}),
                ),
                ("GET", "/api/v1/diagrams/d1") => (
                    "200 OK",
                    json!({"code":0,"data":{"id":"d1","name":"主链","revision":revision,"tables":[],"references":[],"areas":[],"notes":[]},"request_id":"r-get"}),
                ),
                ("PUT", "/api/v1/diagrams/d1") if body["expected_revision"] == revision => {
                    revision += 1;
                    (
                        "200 OK",
                        json!({"code":0,"data":{"id":"d1","revision":revision},"request_id":"r-update"}),
                    )
                }
                ("PUT", "/api/v1/diagrams/d1") => (
                    "409 Conflict",
                    json!({"code":409,"message":"revision conflict","request_id":"r-conflict","details":{"current_revision":revision,"secret":"不得透传"}}),
                ),
                ("POST", "/api/v1/diagrams/import") => (
                    "200 OK",
                    json!({"code":0,"data":{"diagram_id":"d2","imported_tables":0,"imported_fields":0,"warnings":[]},"request_id":"r-import"}),
                ),
                ("DELETE", "/api/v1/diagrams/d1") => (
                    "200 OK",
                    json!({"code":0,"data":{"id":"d1"},"request_id":"r-delete-1"}),
                ),
                ("DELETE", "/api/v1/diagrams/d2") => (
                    "200 OK",
                    json!({"code":0,"data":{"id":"d2"},"request_id":"r-delete-2"}),
                ),
                _ => (
                    "500 Internal Server Error",
                    json!({"message":"unexpected request"}),
                ),
            };
            write_response(&mut stream, status, response).await;
        }
    });
    (format!("http://{address}"), requests)
}

#[tokio::test]
async fn write_chain_revision_and_request_shapes() {
    let started = Instant::now();
    let (base_url, requests) = stateful_backend().await;
    let service = McpService::new(ApiClient::new(config(base_url, None, 5)).unwrap());

    let created = service
        .call(
            "create_diagram",
            json!({"name":"主链","database":"postgresql"}),
        )
        .await
        .unwrap();
    assert_eq!(created["id"], "d1");
    let diagram = service
        .call("get_diagram", json!({"id":"d1"}))
        .await
        .unwrap()["diagram"]
        .clone();
    let updated = service
        .call(
            "update_diagram",
            json!({"id":"d1","expected_revision":0,"diagram":diagram}),
        )
        .await
        .unwrap();
    assert_eq!(updated["revision"], 1);
    let conflict = service
        .call(
            "update_diagram",
            json!({"id":"d1","expected_revision":0,"diagram":{"id":"d1"}}),
        )
        .await
        .unwrap_err();
    assert_eq!(conflict.code, "REVISION_CONFLICT");
    assert_eq!(conflict.details.as_ref().unwrap()["current_revision"], 1);
    assert!(conflict.details.as_ref().unwrap().get("secret").is_none());
    assert!(!conflict.retryable);

    let exported = service
        .call("export_schema", json!({"id":"d1","format":"json"}))
        .await
        .unwrap();
    assert_eq!(exported["revision"], 1);
    let imported = service
        .call(
            "import_schema",
            json!({"format":"drawdb_json","source":"test","payload":{"name":"导入","tables":[]}}),
        )
        .await
        .unwrap();
    assert_eq!(imported["diagram_id"], "d2");
    assert_eq!(
        service
            .call("delete_diagram", json!({"id":"d1","confirm":true}))
            .await
            .unwrap()["deleted"],
        true
    );
    assert_eq!(
        service
            .call("delete_diagram", json!({"id":"d2","confirm":true}))
            .await
            .unwrap()["deleted"],
        true
    );

    let captured = requests.lock().unwrap();
    assert_eq!(
        captured[0].2,
        json!({"name":"主链","database":"postgresql"})
    );
    assert_eq!(captured[2].2["expected_revision"], 0);
    assert!(captured[2].2["diagram"].is_object());
    record(
        &[
            "UT-MCP-07",
            "UT-MCP-08",
            "UT-MCP-12",
            "ST-MCP-03",
            "ST-MCP-04",
        ],
        started,
    );
}

#[tokio::test]
async fn ut_mcp_10_import_validation_is_local() {
    let started = Instant::now();
    let service =
        McpService::new(ApiClient::new(config("http://127.0.0.1:9".into(), None, 2)).unwrap());
    for arguments in [
        json!({"format":"sql","payload":{}}),
        json!({"format":"drawdb_json","payload":"invalid"}),
    ] {
        assert_eq!(
            service
                .call("import_schema", arguments)
                .await
                .unwrap_err()
                .code,
            "VALIDATION_ERROR"
        );
    }
    record(&["UT-MCP-10"], started);
}

#[test]
fn ut_mcp_13_http_error_mapping() {
    let started = Instant::now();
    for (status, expected, retryable) in [
        (400, "VALIDATION_ERROR", false),
        (401, "UNAUTHENTICATED", false),
        (403, "PERMISSION_DENIED", false),
        (404, "NOT_FOUND", false),
        (422, "VALIDATION_ERROR", false),
        (500, "UPSTREAM_ERROR", true),
    ] {
        let error = ToolError::upstream(
            StatusCode::from_u16(status).unwrap(),
            &json!({"message":"safe","request_id":"r1","details":{"field":"name","token":"secret"}}),
        );
        assert_eq!(error.code, expected);
        assert_eq!(error.retryable, retryable);
        assert!(error.details.as_ref().unwrap().get("token").is_none());
    }
    record(&["UT-MCP-13"], started);
}

#[tokio::test]
async fn st_mcp_05_timeout_and_redaction() {
    let started = Instant::now();
    let token = "mcp-test-token-never-log";
    let debug = format!("{:?}", config("http://127.0.0.1:9".into(), Some(token), 1));
    assert!(!debug.contains(token));
    assert!(debug.contains("[REDACTED]"));

    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    tokio::spawn(async move {
        let (_stream, _) = listener.accept().await.unwrap();
        tokio::time::sleep(Duration::from_secs(2)).await;
    });
    let api = ApiClient::new(config(format!("http://{address}"), Some(token), 1)).unwrap();
    let error = api.get("d1").await.unwrap_err();
    assert_eq!(error.code, "UPSTREAM_TIMEOUT");
    let serialized = serde_json::to_string(&error).unwrap();
    assert!(!serialized.contains(token));
    record(&["UT-MCP-15", "ST-MCP-05"], started);
}
