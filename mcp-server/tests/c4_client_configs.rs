use std::io::{BufRead, BufReader, Write};
use std::process::{Command, Stdio};
use std::time::Instant;

use coldrawdb_mcp::reporter;
use serde_json::{json, Value};

fn handshake(case_id: &str) {
    let started = Instant::now();
    let mut child = Command::new(env!("CARGO_BIN_EXE_coldrawdb-mcp"))
        .env("COLDRAWDB_BASE_URL", "http://127.0.0.1:9")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let mut input = child.stdin.take().unwrap();
    writeln!(input, "{}", json!({"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-06-18","capabilities":{},"clientInfo":{"name":"config-fixture-test","version":"1"}}})).unwrap();
    writeln!(
        input,
        "{}",
        json!({"jsonrpc":"2.0","method":"notifications/initialized","params":{}})
    )
    .unwrap();
    writeln!(
        input,
        "{}",
        json!({"jsonrpc":"2.0","id":2,"method":"tools/list","params":{}})
    )
    .unwrap();
    drop(input);

    let mut output = BufReader::new(child.stdout.take().unwrap());
    let mut initialized = String::new();
    let mut listed = String::new();
    output.read_line(&mut initialized).unwrap();
    output.read_line(&mut listed).unwrap();
    let initialized: Value = serde_json::from_str(&initialized).unwrap();
    let listed: Value = serde_json::from_str(&listed).unwrap();
    assert_eq!(
        initialized.pointer("/result/serverInfo/name"),
        Some(&json!("coldrawdb-mcp"))
    );
    let tools = listed.pointer("/result/tools").unwrap().as_array().unwrap();
    assert_eq!(tools.len(), 7);
    assert!(!serde_json::to_string(tools).unwrap().contains("#/schemas/"));
    assert!(child.wait().unwrap().success());
    reporter::report(case_id, Ok(()), started.elapsed().as_millis());
}

#[test]
fn ut_mcp_11_and_four_client_handshakes() {
    let started = Instant::now();
    let claude: Value = serde_json::from_str(include_str!("../examples/claude.mcp.json")).unwrap();
    let cursor: Value = serde_json::from_str(include_str!("../examples/cursor.mcp.json")).unwrap();
    let opencode: Value = serde_json::from_str(include_str!("../examples/opencode.json")).unwrap();
    let codex: toml::Value = toml::from_str(include_str!("../examples/codex.config.toml")).unwrap();

    for fixture in [&claude, &cursor] {
        assert_eq!(
            fixture.pointer("/mcpServers/coldrawdb/type"),
            Some(&json!("stdio"))
        );
        assert_eq!(
            fixture.pointer("/mcpServers/coldrawdb/args"),
            Some(&json!([]))
        );
        assert_eq!(
            fixture.pointer("/mcpServers/coldrawdb/env/COLDRAWDB_BASE_URL"),
            Some(&json!("http://localhost:3000"))
        );
        assert!(fixture
            .pointer("/mcpServers/coldrawdb/env/COLDRAWDB_ACCESS_TOKEN")
            .is_none());
    }
    assert_eq!(
        codex["mcp_servers"]["coldrawdb"]["command"].as_str(),
        Some("/ABS/PATH/coldrawdb-mcp")
    );
    assert_eq!(
        codex["mcp_servers"]["coldrawdb"]["default_tools_approval_mode"].as_str(),
        Some("writes")
    );
    assert_eq!(
        codex["mcp_servers"]["coldrawdb"]["env"]["COLDRAWDB_BASE_URL"].as_str(),
        Some("http://localhost:3000")
    );
    assert_eq!(
        opencode.pointer("/mcp/coldrawdb/type"),
        Some(&json!("local"))
    );
    assert_eq!(
        opencode.pointer("/mcp/coldrawdb/command/0"),
        Some(&json!("/ABS/PATH/coldrawdb-mcp"))
    );
    assert_eq!(
        opencode.pointer("/mcp/coldrawdb/environment/COLDRAWDB_BASE_URL"),
        Some(&json!("http://localhost:3000"))
    );
    reporter::report("UT-MCP-11", Ok(()), started.elapsed().as_millis());

    handshake("ST-MCP-06");
    handshake("ST-MCP-07");
    handshake("ST-MCP-08");
    handshake("ST-MCP-09");
}
