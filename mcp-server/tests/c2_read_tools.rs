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
    assert_eq!(tools.len(), 12); // mcp-dictionary-tools（2026-09-28）：+update_dictionary
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

fn sql_table_body<'a>(sql: &'a str, quoted_table: &str) -> &'a str {
    let marker = format!("CREATE TABLE {quoted_table} (");
    let (_, body) = sql.split_once(&marker).unwrap();
    body.split("\n)").next().unwrap()
}

#[tokio::test]
async fn ut_mcp_05_and_st_mcp_02_get_and_export() {
    let started = Instant::now();
    let diagram = json!({
        "id":"d1","name":"示例","revision":4,
        "tables":[
            {
                "id":"t1","name":"users","comment":"用户's 主表",
                "fields":[
                    {"id":"f1","name":"id","type_":"UUID","primary":true,"not_null":true,"comment":"用户's 标识"},
                    {"id":"f2","name":"email","type_":"VARCHAR(255)","not_null":true,"unique":true,"default":"'unknown@example.com'","comment":"登录's 邮箱"}
                ]
            },
            {
                "id":"t2","name":"membership","comment":"成员's 关系",
                "fields":[
                    {"id":"f3","name":"tenant_id","type_":"UUID","primary":true,"not_null":true},
                    {"id":"f4","name":"user_id","type_":"UUID","primary":true,"not_null":true,"comment":"成员's 用户"},
                    {"id":"f5","name":"status","type_":"VARCHAR(16)","not_null":true,"unique":true,"default":"'active'","comment":"状态's 说明"}
                ]
            }
        ],
        "references":[
            {"id":"r1","start_table_id":"t2","start_field_id":"f4","end_table_id":"t1","end_field_id":"f1","on_delete":"CASCADE","on_update":"SET_NULL"},
            {"id":"r2","start_table_id":"t2","start_field_id":"f5","end_table_id":"missing","end_field_id":"missing"}
        ],
        "areas":[],"notes":[]
    });
    let base = mock_response(
        "200 OK",
        json!({"code":0,"data":diagram.clone(),"request_id":"r1"}),
    )
    .await;
    let api = ApiClient::new(config(base)).unwrap();
    let result = api.get("d1").await.unwrap();
    assert_eq!(result.pointer("/diagram/revision"), Some(&json!(4)));
    assert_eq!(result["diagram"], diagram);

    let json_export = export_diagram(&result["diagram"], "json").unwrap();
    assert_eq!(json_export["diagram_id"], "d1");
    assert_eq!(json_export["mime_type"], "application/json");
    assert_eq!(
        json_export["content"].as_str().unwrap(),
        serde_json::to_string_pretty(&diagram).unwrap()
    );
    assert_eq!(
        json_export,
        export_diagram(&result["diagram"], "json").unwrap()
    );

    let dbml = export_diagram(&result["diagram"], "dbml").unwrap();
    assert_eq!(dbml["diagram_id"], "d1");
    assert_eq!(dbml["revision"], 4);
    assert_eq!(dbml["format"], "dbml");
    assert_eq!(dbml["mime_type"], "text/plain");
    assert_eq!(dbml, export_diagram(&result["diagram"], "dbml").unwrap());
    let dbml_content = dbml["content"].as_str().unwrap();
    assert!(dbml_content.contains("  id UUID [pk, not null, note: '用户\\'s 标识']"));
    assert!(dbml_content.contains("  tenant_id UUID [not null]"));
    assert!(dbml_content.contains("  user_id UUID [not null, note: '成员\\'s 用户']"));
    assert!(dbml_content.contains(
        "  status VARCHAR(16) [not null, unique, default: `'active'`, note: '状态\\'s 说明']"
    ));
    assert!(dbml_content.contains("  indexes {\n    (tenant_id, user_id) [pk]\n  }"));
    assert!(dbml_content.contains("  Note: '成员\\'s 关系'"));
    assert!(dbml_content
        .contains("Ref: membership.user_id > users.id [delete: cascade, update: set null]"));
    assert_eq!(dbml_content.matches("Ref:").count(), 1);
    assert!(!dbml_content.contains('?'));

    for format in [
        "mysql",
        "postgresql",
        "sqlite",
        "mariadb",
        "mssql",
        "oracle",
        "generic",
    ] {
        let exported = export_diagram(&result["diagram"], format).unwrap();
        assert_eq!(exported["diagram_id"], "d1");
        assert_eq!(exported["revision"], 4);
        assert_eq!(exported["format"], format);
        assert_eq!(exported["mime_type"], "text/sql");
        assert_eq!(
            exported,
            export_diagram(&result["diagram"], format).unwrap()
        );
        let content = exported["content"].as_str().unwrap();
        let quoted_membership = match format {
            "mysql" | "mariadb" => "`membership`",
            "mssql" => "[membership]",
            _ => "\"membership\"",
        };
        let membership = sql_table_body(content, quoted_membership);
        let primary_lines: Vec<&str> = membership
            .lines()
            .filter(|line| line.contains("PRIMARY KEY"))
            .collect();
        assert_eq!(
            primary_lines.len(),
            1,
            "unexpected {format} DDL:\n{content}"
        );
        assert!(primary_lines[0].trim_start().starts_with("PRIMARY KEY ("));
        assert!(membership.contains("NOT NULL"));
        assert!(membership.contains("UNIQUE"));
        assert!(membership.contains("DEFAULT 'active'"));
        assert!(membership.contains("FOREIGN KEY"));
        assert_eq!(content.matches("FOREIGN KEY").count(), 1);
        assert!(membership.contains("ON DELETE CASCADE"));
        assert!(membership.contains("ON UPDATE SET NULL"));
        assert!(!content.contains('?'));
        match format {
            "mysql" | "mariadb" => {
                assert!(content.contains("COMMENT '状态''s 说明'"));
                assert!(content.contains(") COMMENT='成员''s 关系';"));
                assert!(!content.contains("COMMENT ON"));
            }
            "sqlite" | "mssql" => assert!(!content.contains("COMMENT")),
            _ => {
                assert!(content.contains("COMMENT ON TABLE \"membership\" IS '成员''s 关系';"));
                assert!(content
                    .contains("COMMENT ON COLUMN \"membership\".\"status\" IS '状态''s 说明';"));
            }
        }
    }

    let postgresql = export_diagram(&result["diagram"], "postgresql").unwrap();
    let postgresql = postgresql["content"].as_str().unwrap();
    let users = sql_table_body(postgresql, "\"users\"");
    assert!(users.contains("  \"id\" UUID PRIMARY KEY NOT NULL"));
    let membership = sql_table_body(postgresql, "\"membership\"");
    assert!(membership.contains("  PRIMARY KEY (\"tenant_id\", \"user_id\")"));
    assert!(membership.contains(
        "  FOREIGN KEY (\"user_id\") REFERENCES \"users\" (\"id\") ON DELETE CASCADE ON UPDATE SET NULL"
    ));
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
    // 黑洞端口(127.0.0.1:9)的 reqwest 行为在宿主间不稳定:
    //   - 立即 connection refused → transport() else 分支 → UPSTREAM_UNAVAILABLE
    //   - connect timeout          → transport() is_timeout() 分支 → UPSTREAM_TIMEOUT
    //   - 被识别为 HTTP 5xx 响应   → upstream() 分支 → UPSTREAM_ERROR(retryable=true)
    // 本测试的本意是验证上游不可达时被标记 retryable,不强锁字面值。
    let code = error.code.as_str();
    assert!(
        code == "UPSTREAM_UNAVAILABLE" || code == "UPSTREAM_TIMEOUT" || code == "UPSTREAM_ERROR",
        "expected unreachable-upstream error code, got {}",
        code
    );
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
        12 // mcp-dictionary-tools（2026-09-28）：+update_dictionary
    );
    assert!(child.wait().unwrap().success());
    record(&["ST-MCP-01"], started);
}
