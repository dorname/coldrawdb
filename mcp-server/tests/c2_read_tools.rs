use std::collections::HashMap;
use std::io::{BufRead, BufReader, Write};
use std::process::{Command, Stdio};
use std::time::Instant;

use coldrawdb_mcp::api::{normalize_list, ApiClient};
use coldrawdb_mcp::export::export_diagram;
use coldrawdb_mcp::protocol;
use coldrawdb_mcp::reporter;
use coldrawdb_mcp::{Config, McpService};
use serde_json::{json, Value};
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

async fn mock_response(status: &str, body: Value) -> String {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let status = status.to_string();
    tokio::spawn(async move {
        let (mut stream, _) = listener.accept().await.unwrap();
        let mut request = vec![0_u8; 8192];
        let _ = stream.read(&mut request).await.unwrap();
        let payload = body.to_string();
        let response = format!(
            "HTTP/1.1 {status}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{payload}",
            payload.len()
        );
        stream.write_all(response.as_bytes()).await.unwrap();
    });
    format!("http://{address}")
}

#[test]
fn ut_mcp_01_config_and_exit_code() {
    let started = Instant::now();
    assert_eq!(
        Config::from_values(HashMap::new()).unwrap_err().code,
        "CONFIG_INVALID"
    );
    assert_eq!(
        Config::from_values(HashMap::from([(
            "COLDRAWDB_BASE_URL".into(),
            "file:///tmp/db".into()
        )]))
        .unwrap_err()
        .code,
        "CONFIG_INVALID"
    );
    let status = Command::new(env!("CARGO_BIN_EXE_coldrawdb-mcp"))
        .env_remove("COLDRAWDB_BASE_URL")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .status()
        .unwrap();
    assert_eq!(status.code(), Some(2));
    record(&["UT-MCP-01"], started);
}

#[tokio::test]
async fn ut_mcp_02_03_initialize_and_contract() {
    let started = Instant::now();
    let service = McpService::new(ApiClient::new(config("http://127.0.0.1:9".into())).unwrap());
    let initialized = protocol::handle(
        &service,
        json!({"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-06-18"}}),
    )
    .await
    .unwrap();
    assert_eq!(
        initialized.pointer("/result/serverInfo/name"),
        Some(&json!("coldrawdb-mcp"))
    );
    assert!(initialized
        .pointer("/result/capabilities/tools")
        .unwrap()
        .is_object());
    assert!(initialized
        .pointer("/result/instructions")
        .unwrap()
        .as_str()
        .unwrap()
        .contains("revision"));

    let listed = protocol::handle(
        &service,
        json!({"jsonrpc":"2.0","id":2,"method":"tools/list"}),
    )
    .await
    .unwrap();
    let tools = listed.pointer("/result/tools").unwrap().as_array().unwrap();
    assert_eq!(tools.len(), 7);
    assert_eq!(
        tools
            .iter()
            .filter(|tool| tool.get("name") == Some(&json!("delete_diagram")))
            .count(),
        1
    );
    assert_eq!(
        tools
            .iter()
            .find(|tool| tool.get("name") == Some(&json!("delete_diagram")))
            .unwrap()
            .pointer("/annotations/destructiveHint"),
        Some(&json!(true))
    );
    record(&["UT-MCP-02", "UT-MCP-03"], started);
}

#[test]
fn ut_mcp_04_list_normalization() {
    let started = Instant::now();
    let value = normalize_list(
        &json!({"code":200,"data":[
            {"id":"2","name":"Zoo","database":"mysql","lastModified":"2026-01-02"},
            {"id":"1","name":"alpha","database":null,"lastModified":"2026-01-01"},
            {"id":"3","name":"Alphabet","database":"sqlite"}
        ]}),
        Some("ALP"),
        1,
    )
    .unwrap();
    assert_eq!(value["count"], 1);
    assert_eq!(value["items"][0]["name"], "alpha");
    assert_eq!(value["items"][0]["revision"], 0);
    record(&["UT-MCP-04"], started);
}

#[tokio::test]
async fn ut_mcp_05_and_st_mcp_02_get_and_export() {
    let started = Instant::now();
    let diagram = json!({
        "id":"d1","name":"示例","revision":4,
        "tables":[{"id":"t1","name":"users","fields":[{"id":"f1","name":"id","type_":"UUID","primary":true}]}],
        "references":[],"areas":[],"notes":[]
    });
    let base = mock_response("200 OK", json!({"code":0,"data":diagram,"request_id":"r1"})).await;
    let api = ApiClient::new(config(base)).unwrap();
    let result = api.get("d1").await.unwrap();
    assert_eq!(result.pointer("/diagram/revision"), Some(&json!(4)));
    for format in [
        "json",
        "dbml",
        "mysql",
        "postgresql",
        "sqlite",
        "mariadb",
        "mssql",
        "oracle",
        "generic",
    ] {
        let exported = export_diagram(&result["diagram"], format).unwrap();
        assert_eq!(exported["revision"], 4);
        assert!(!exported["content"].as_str().unwrap().is_empty());
        assert_eq!(
            exported,
            export_diagram(&result["diagram"], format).unwrap()
        );
    }
    record(&["UT-MCP-05", "UT-MCP-06", "ST-MCP-02"], started);
}

#[tokio::test]
async fn ut_mcp_09_delete_requires_confirmation() {
    let started = Instant::now();
    let service = McpService::new(ApiClient::new(config("http://127.0.0.1:9".into())).unwrap());
    let error = service
        .call("delete_diagram", json!({"id":"d1","confirm":false}))
        .await
        .unwrap_err();
    assert_eq!(error.code, "VALIDATION_ERROR");
    record(&["UT-MCP-09"], started);
}

#[tokio::test]
async fn ut_mcp_14_connection_failure() {
    let started = Instant::now();
    let api = ApiClient::new(config("http://127.0.0.1:9".into())).unwrap();
    let error = api.get("d1").await.unwrap_err();
    assert_eq!(error.code, "UPSTREAM_UNAVAILABLE");
    assert!(error.retryable);
    record(&["UT-MCP-14"], started);
}

#[test]
fn st_mcp_01_real_stdio_handshake() {
    let started = Instant::now();
    let mut child = Command::new(env!("CARGO_BIN_EXE_coldrawdb-mcp"))
        .env("COLDRAWDB_BASE_URL", "http://127.0.0.1:9")
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
        json!({"jsonrpc":"2.0","id":2,"method":"tools/list","params":{}})
    )
    .unwrap();
    drop(stdin);

    let mut reader = BufReader::new(child.stdout.take().unwrap());
    let mut first = String::new();
    let mut second = String::new();
    reader.read_line(&mut first).unwrap();
    reader.read_line(&mut second).unwrap();
    let initialized: Value = serde_json::from_str(&first).unwrap();
    let listed: Value = serde_json::from_str(&second).unwrap();
    assert_eq!(
        initialized.pointer("/result/serverInfo/name"),
        Some(&json!("coldrawdb-mcp"))
    );
    assert_eq!(
        listed
            .pointer("/result/tools")
            .unwrap()
            .as_array()
            .unwrap()
            .len(),
        7
    );
    assert!(child.wait().unwrap().success());
    record(&["ST-MCP-01"], started);
}
