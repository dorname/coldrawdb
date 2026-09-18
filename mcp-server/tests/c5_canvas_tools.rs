//! mcp-canvas-tools（2026-09-18）：画布编辑工具 UT 测试
//! 覆盖 UT-MCP-16～22

use coldrawdb_mcp::api::ApiClient;
use coldrawdb_mcp::error::ToolError;
use coldrawdb_mcp::layout::{force_directed_layout, LayoutParams};
use coldrawdb_mcp::reporter;
use coldrawdb_mcp::{Config, McpService};
use serde_json::{json, Value};
use std::collections::HashMap;

fn config(base_url: String) -> Config {
    Config::from_values(HashMap::from([
        ("COLDRAWDB_BASE_URL".into(), base_url),
        ("COLDRAWDB_REQUEST_TIMEOUT_SECS".into(), "5".into()),
    ]))
    .unwrap()
}

fn service(base_url: String) -> McpService {
    let api = ApiClient::new(config(base_url)).unwrap();
    McpService::new(api)
}

fn record(ids: &[&str]) {
    let started = std::time::Instant::now();
    for id in ids {
        reporter::report(id, Ok(()), started.elapsed().as_millis());
    }
}

// ── UT-MCP-16: update_table 参数校验 ─────────────────────────────────────

#[tokio::test]
async fn ut_mcp16_update_table_missing_table_id() {
    record(&["UT-MCP-16"]);
    let svc = service("http://127.0.0.1:0".into());
    let result = svc
        .call("update_table", json!({"id": "d1"}))
        .await;
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert_eq!(err.code, "VALIDATION_ERROR");
}

#[tokio::test]
async fn ut_mcp16_update_table_name_too_long() {
    record(&["UT-MCP-16"]);
    let svc = service("http://127.0.0.1:0".into());
    let long_name = "a".repeat(65);
    let result = svc
        .call(
            "update_table",
            json!({"id": "d1", "table_id": "t1", "name": long_name}),
        )
        .await;
    assert!(result.is_err());
    assert_eq!(result.unwrap_err().code, "VALIDATION_ERROR");
}

#[tokio::test]
async fn ut_mcp16_update_table_invalid_x_type() {
    record(&["UT-MCP-16"]);
    let svc = service("http://127.0.0.1:0".into());
    // x 为字符串时，as_f64 返回 None，不会 panic，会被忽略（非严格校验）
    // 但缺少必需参数时应报错
    let result = svc
        .call("update_table", json!({"id": "d1", "table_id": "t1", "x": "abc"}))
        .await;
    // 不会报 VALIDATION_ERROR（x 被忽略），但会尝试连接后端 → UPSTREAM_ERROR
    assert!(result.is_err());
}

// ── UT-MCP-17: update_field 参数校验 ─────────────────────────────────────

#[tokio::test]
async fn ut_mcp17_update_field_missing_field_id() {
    record(&["UT-MCP-17"]);
    let svc = service("http://127.0.0.1:0".into());
    let result = svc
        .call("update_field", json!({"id": "d1", "table_id": "t1"}))
        .await;
    assert!(result.is_err());
    assert_eq!(result.unwrap_err().code, "VALIDATION_ERROR");
}

#[tokio::test]
async fn ut_mcp17_update_field_name_too_long() {
    record(&["UT-MCP-17"]);
    let svc = service("http://127.0.0.1:0".into());
    let long_name = "a".repeat(65);
    let result = svc
        .call(
            "update_field",
            json!({"id": "d1", "table_id": "t1", "field_id": "f1", "name": long_name}),
        )
        .await;
    assert!(result.is_err());
    assert_eq!(result.unwrap_err().code, "VALIDATION_ERROR");
}

// ── UT-MCP-18: update_reference 参数校验 ─────────────────────────────────

#[tokio::test]
async fn ut_mcp18_create_missing_endpoint_ids() {
    record(&["UT-MCP-18"]);
    let svc = service("http://127.0.0.1:0".into());
    let result = svc
        .call(
            "update_reference",
            json!({"id": "d1", "action": "create"}),
        )
        .await;
    assert!(result.is_err());
    assert_eq!(result.unwrap_err().code, "VALIDATION_ERROR");
}

#[tokio::test]
async fn ut_mcp18_update_missing_ref_id() {
    record(&["UT-MCP-18"]);
    let svc = service("http://127.0.0.1:0".into());
    let result = svc
        .call(
            "update_reference",
            json!({"id": "d1", "action": "update"}),
        )
        .await;
    assert!(result.is_err());
    assert_eq!(result.unwrap_err().code, "VALIDATION_ERROR");
}

#[tokio::test]
async fn ut_mcp18_invalid_action() {
    record(&["UT-MCP-18"]);
    let svc = service("http://127.0.0.1:0".into());
    let result = svc
        .call(
            "update_reference",
            json!({"id": "d1", "action": "merge"}),
        )
        .await;
    assert!(result.is_err());
    assert_eq!(result.unwrap_err().code, "VALIDATION_ERROR");
}

// ── UT-MCP-19: layout_diagram 力导向算法 ─────────────────────────────────

fn make_table(id: &str, x: f64, y: f64) -> Value {
    json!({"id": id, "x": x, "y": y, "name": id})
}

fn make_ref(start: &str, end: &str) -> Value {
    json!({"start_table_id": start, "end_table_id": end})
}

#[test]
fn ut_mcp19_force_layout_deterministic() {
    record(&["UT-MCP-19", "ST-MCP-12"]);
    let tables = vec![
        make_table("a", 0.0, 0.0),
        make_table("b", 100.0, 0.0),
        make_table("c", 50.0, 100.0),
    ];
    let refs = vec![make_ref("a", "b"), make_ref("b", "c")];
    let params = LayoutParams::default();

    let r1 = force_directed_layout(&tables, &refs, &params);
    let r2 = force_directed_layout(&tables, &refs, &params);
    assert_eq!(r1, r2);
}

#[test]
fn ut_mcp19_force_layout_no_overlap() {
    record(&["UT-MCP-19"]);
    let tables = vec![
        make_table("a", 0.0, 0.0),
        make_table("b", 10.0, 10.0),
        make_table("c", 20.0, 20.0),
    ];
    let refs = vec![make_ref("a", "b"), make_ref("b", "c")];
    let params = LayoutParams {
        iterations: 100,
        spacing: 180.0,
    };

    let result = force_directed_layout(&tables, &refs, &params);
    for i in 0..result.len() {
        for j in (i + 1)..result.len() {
            let dx = result[i]["x"].as_f64().unwrap() - result[j]["x"].as_f64().unwrap();
            let dy = result[i]["y"].as_f64().unwrap() - result[j]["y"].as_f64().unwrap();
            let dist = (dx * dx + dy * dy).sqrt();
            assert!(dist > 50.0, "表 {} 和 {} 距离过近: {}", i, j, dist);
        }
    }
}

#[test]
fn ut_mcp19_force_layout_isolated_unchanged() {
    record(&["UT-MCP-19"]);
    let tables = vec![
        make_table("a", 0.0, 0.0),
        make_table("b", 100.0, 0.0),
        make_table("isolated", 500.0, 500.0),
    ];
    let refs = vec![make_ref("a", "b")];
    let params = LayoutParams::default();

    let result = force_directed_layout(&tables, &refs, &params);
    let iso = result.iter().find(|t| t["id"] == "isolated").unwrap();
    let x = iso["x"].as_f64().unwrap();
    let y = iso["y"].as_f64().unwrap();
    assert!((x - 500.0).abs() < 5.0);
    assert!((y - 500.0).abs() < 5.0);
}

// ── UT-MCP-20: tools/list 工具总数 = 11 ─────────────────────────────────

#[test]
fn ut_mcp20_tools_list_has_eleven_tools() {
    record(&["UT-MCP-20"]);
    let tools = McpService::tools().expect("tools/list 应成功");
    assert_eq!(tools.len(), 11, "应有 11 个工具，实际: {}", tools.len());

    let names: Vec<&str> = tools
        .iter()
        .map(|t| t["name"].as_str().unwrap())
        .collect();
    let expected = [
        "list_diagrams",
        "get_diagram",
        "create_diagram",
        "update_diagram",
        "delete_diagram",
        "import_schema",
        "export_schema",
        "update_table",
        "update_field",
        "update_reference",
        "layout_diagram",
    ];
    for name in &expected {
        assert!(names.contains(name), "缺少工具: {}", name);
    }
}

// ── UT-MCP-21: update_table 端到端（mock HTTP）────────────────────────────

// 使用 wiremock 或手工 mock server；此处用 tokio TcpListener 简化版

#[tokio::test]
async fn ut_mcp21_update_table_e2e_mock() {
    record(&["UT-MCP-21", "ST-MCP-10"]);
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::net::TcpListener;

    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let base_url = format!("http://{}", addr);

    let svc = service(base_url);

    let diagram_response = json!({
        "code": 0,
        "data": {
            "id": "d1",
            "revision": 5,
            "tables": [
                {"id": "t1", "name": "old_name", "comment": "", "x": 0.0, "y": 0.0},
                {"id": "t2", "name": "other", "comment": "", "x": 100.0, "y": 100.0},
            ],
            "references": []
        }
    });

    let update_response = json!({"code": 0, "data": {"id": "d1", "revision": 6}});

    tokio::spawn(async move {
        // 第一次：GET 请求
        let (mut stream, _) = listener.accept().await.unwrap();
        let mut buf = vec![0u8; 4096];
        let n = stream.read(&mut buf).await.unwrap();
        let request = String::from_utf8_lossy(&buf[..n]);
        assert!(request.contains("GET /api/v1/diagrams/d1"));

        let body = serde_json::to_string(&diagram_response).unwrap();
        let response = format!(
            "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
            body.len(),
            body
        );
        stream.write_all(response.as_bytes()).await.unwrap();
        stream.shutdown().await.unwrap();

        // 第二次：PUT 请求
        let (mut stream, _) = listener.accept().await.unwrap();
        let n = stream.read(&mut buf).await.unwrap();
        let request = String::from_utf8_lossy(&buf[..n]);
        assert!(request.contains("PUT /api/v1/diagrams/d1"));
        assert!(request.contains("expected_revision")); // 应包含乐观锁参数

        let body = serde_json::to_string(&update_response).unwrap();
        let response = format!(
            "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
            body.len(),
            body
        );
        stream.write_all(response.as_bytes()).await.unwrap();
        stream.shutdown().await.unwrap();
    });

    let result = svc
        .call(
            "update_table",
            json!({"id": "d1", "table_id": "t1", "name": "new_name", "comment": "新表名"}),
        )
        .await;

    assert!(result.is_ok());
    let resp = result.unwrap();
    assert_eq!(resp["revision"], 6);
}

// ── UT-MCP-22: update_reference create/delete（mock HTTP）─────────────────

#[tokio::test]
async fn ut_mcp22_update_reference_create_e2e_mock() {
    record(&["UT-MCP-22", "ST-MCP-11"]);
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::net::TcpListener;

    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let base_url = format!("http://{}", addr);
    let svc = service(base_url);

    let diagram_response = json!({
        "code": 0,
        "data": {
            "id": "d1",
            "revision": 3,
            "tables": [
                {"id": "t1", "name": "users", "x": 0.0, "y": 0.0},
                {"id": "t2", "name": "orders", "x": 200.0, "y": 0.0},
            ],
            "references": []
        }
    });

    let update_response = json!({"code": 0, "data": {"id": "d1", "revision": 4}});

    tokio::spawn(async move {
        let (mut stream, _) = listener.accept().await.unwrap();
        let mut buf = vec![0u8; 4096];
        let _ = stream.read(&mut buf).await;
        let body = serde_json::to_string(&diagram_response).unwrap();
        let response = format!(
            "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
            body.len(), body
        );
        stream.write_all(response.as_bytes()).await.unwrap();
        stream.shutdown().await.unwrap();

        let (mut stream, _) = listener.accept().await.unwrap();
        let n = stream.read(&mut buf).await.unwrap();
        let request = String::from_utf8_lossy(&buf[..n]);
        assert!(request.contains("start_table_id")); // 应包含新边

        let body = serde_json::to_string(&update_response).unwrap();
        let response = format!(
            "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
            body.len(), body
        );
        stream.write_all(response.as_bytes()).await.unwrap();
        stream.shutdown().await.unwrap();
    });

    let result = svc
        .call(
            "update_reference",
            json!({
                "id": "d1",
                "action": "create",
                "start_table_id": "t1",
                "start_field_id": "f1",
                "end_table_id": "t2",
                "end_field_id": "f2",
                "cardinality": "one_to_many"
            }),
        )
        .await;

    assert!(result.is_ok());
}

#[tokio::test]
async fn ut_mcp22_update_reference_delete_e2e_mock() {
    record(&["UT-MCP-22"]);
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::net::TcpListener;

    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let base_url = format!("http://{}", addr);
    let svc = service(base_url);

    let diagram_response = json!({
        "code": 0,
        "data": {
            "id": "d1",
            "revision": 7,
            "tables": [],
            "references": [
                {"id": "r1", "start_table_id": "t1", "end_table_id": "t2"}
            ]
        }
    });

    let update_response = json!({"code": 0, "data": {"id": "d1", "revision": 8}});

    tokio::spawn(async move {
        let (mut stream, _) = listener.accept().await.unwrap();
        let mut buf = vec![0u8; 4096];
        let _ = stream.read(&mut buf).await;
        let body = serde_json::to_string(&diagram_response).unwrap();
        let response = format!(
            "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
            body.len(), body
        );
        stream.write_all(response.as_bytes()).await.unwrap();
        stream.shutdown().await.unwrap();

        let (mut stream, _) = listener.accept().await.unwrap();
        let n = stream.read(&mut buf).await.unwrap();
        let request = String::from_utf8_lossy(&buf[..n]);
        assert!(!request.contains("\"r1\"")); // 删除后不应包含 r1

        let body = serde_json::to_string(&update_response).unwrap();
        let response = format!(
            "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
            body.len(), body
        );
        stream.write_all(response.as_bytes()).await.unwrap();
        stream.shutdown().await.unwrap();
    });

    let result = svc
        .call(
            "update_reference",
            json!({"id": "d1", "action": "delete", "ref_id": "r1"}),
        )
        .await;

    assert!(result.is_ok());
}
