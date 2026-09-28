//! mcp-dictionary-tools（2026-09-28）：数据字典管理工具 UT 测试
//! 覆盖 UT-MCP-33 / UT-MCP-34 / UT-MCP-35
//!
//! 「零请求」口径说明：编码冲突 / 悬空编码等存在性校验必须先 GET 读取图数据，
//! 规格中的「零请求」指**零写请求（不发 PUT）**——测试通过 mock 捕获请求序列断言
//! 仅发生 GET、无任何 PUT。

use coldrawdb_mcp::api::ApiClient;
use coldrawdb_mcp::reporter;
use coldrawdb_mcp::{Config, McpService};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

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

/// 手工 mock server：依次应答 GET / PUT，捕获每个请求的方法+body 到 captured。
/// 返回 (base_url, captured, join_handle)。
async fn mock_server(
    get_response: Value,
    put_response: Value,
) -> (
    String,
    Arc<Mutex<Vec<String>>>,
    tokio::task::JoinHandle<()>,
) {
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::net::TcpListener;

    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let base_url = format!("http://{}", addr);
    let captured = Arc::new(Mutex::new(Vec::<String>::new()));
    let captured_clone = captured.clone();

    let handle = tokio::spawn(async move {
        for response_json in [get_response, put_response] {
            let Ok((mut stream, _)) = listener.accept().await else {
                return;
            };
            let mut buf = vec![0u8; 65536];
            let n = stream.read(&mut buf).await.unwrap();
            let request = String::from_utf8_lossy(&buf[..n]).to_string();
            captured_clone.lock().unwrap().push(request);

            let body = serde_json::to_string(&response_json).unwrap();
            let response = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                body.len(),
                body
            );
            stream.write_all(response.as_bytes()).await.unwrap();
            stream.shutdown().await.unwrap();
        }
    });

    (base_url, captured, handle)
}

fn put_body(captured: &Arc<Mutex<Vec<String>>>) -> Value {
    let reqs = captured.lock().unwrap();
    let put = reqs
        .iter()
        .find(|r| r.starts_with("PUT "))
        .expect("应捕获到 PUT 请求");
    let body_start = put.find("\r\n\r\n").expect("PUT 应有 body") + 4;
    serde_json::from_str(&put[body_start..]).expect("PUT body 应为 JSON")
}

fn put_count(captured: &Arc<Mutex<Vec<String>>>) -> usize {
    captured
        .lock()
        .unwrap()
        .iter()
        .filter(|r| r.starts_with("PUT "))
        .count()
}

/// 基线图：2 字典（yes_no / gender）+ 1 表 2 字段（f1 绑 yes_no，f2 未绑）
fn base_diagram() -> Value {
    json!({
        "code": 0,
        "data": {
            "id": "d1",
            "revision": 5,
            "tables": [{
                "id": "t1", "name": "users", "comment": "", "x": 0.0, "y": 0.0,
                "fields": [
                    {"id": "f1", "name": "active", "type": "INT", "dict_code": "yes_no"},
                    {"id": "f2", "name": "age", "type": "INT", "dict_code": ""},
                ]
            }],
            "references": [],
            "dictionaries": [
                {"id": "dict-1", "name": "是否", "code": "yes_no", "comment": "",
                 "items": [{"id": "di-1", "value": "0", "label": "否", "sort": 0}]},
                {"id": "dict-2", "name": "性别", "code": "gender", "comment": "",
                 "items": []}
            ]
        }
    })
}

fn ok_put() -> Value {
    json!({"code": 0, "data": {"id": "d1", "revision": 6}})
}

// ── UT-MCP-33：create + 编码唯一校验 ─────────────────────────────────────

#[tokio::test]
async fn ut_mcp33_create_missing_name_or_code_zero_request() {
    record(&["UT-MCP-33"]);
    // 缺 name / 缺 code → 本地 VALIDATION_ERROR，零请求（连 GET 都不发）
    let svc = service("http://127.0.0.1:0".into());
    for args in [
        json!({"id": "d1", "action": "create", "code": "x"}),
        json!({"id": "d1", "action": "create", "name": "x"}),
    ] {
        let err = svc.call("update_dictionary", args).await.unwrap_err();
        assert_eq!(err.code, "VALIDATION_ERROR");
    }
}

#[tokio::test]
async fn ut_mcp33_create_success_appends_dict() {
    record(&["UT-MCP-33"]);
    let (base_url, captured, _h) = mock_server(base_diagram(), ok_put()).await;
    let svc = service(base_url);

    let resp = svc
        .call(
            "update_dictionary",
            json!({
                "id": "d1", "action": "create",
                "name": "状态", "code": "status", "comment": "通用状态",
                "items": [{"value": "1", "label": "启用"}, {"id": "di-x", "value": "0", "label": "停用", "sort": 9}]
            }),
        )
        .await
        .expect("create 应成功");

    // 瘦响应结构：{id, revision, dict_id, fields_cleared:0}
    assert_eq!(resp["id"], "d1");
    assert_eq!(resp["revision"], 6);
    assert_eq!(resp["fields_cleared"], 0);
    let dict_id = resp["dict_id"].as_str().unwrap();
    assert!(dict_id.starts_with("dict-"), "缺省 dict_id 应自动生成 dict- 前缀");

    // PUT body：dictionaries 追加一条，缺 id 项自动补 id，其余图元不变
    let body = put_body(&captured);
    let diagram = &body["diagram"];
    let dicts = diagram["dictionaries"].as_array().unwrap();
    assert_eq!(dicts.len(), 3);
    let new_dict = &dicts[2];
    assert_eq!(new_dict["id"].as_str().unwrap(), dict_id);
    assert_eq!(new_dict["code"], "status");
    assert_eq!(new_dict["name"], "状态");
    let items = new_dict["items"].as_array().unwrap();
    assert_eq!(items.len(), 2);
    assert!(items[0]["id"].as_str().unwrap().starts_with("di-"), "缺 id 项应自动补 di- 前缀");
    assert_eq!(items[0]["sort"], 0, "sort 缺省按下标");
    assert_eq!(items[1]["id"], "di-x", "已有 id 保留");
    assert_eq!(items[1]["sort"], 9);
    assert_eq!(diagram["tables"][0]["fields"].as_array().unwrap().len(), 2, "字段不得变动");
    assert_eq!(dicts[0]["code"], "yes_no", "既有字典不得变动");
}

#[tokio::test]
async fn ut_mcp33_create_code_conflict_no_put() {
    record(&["UT-MCP-33"]);
    let (base_url, captured, _h) = mock_server(base_diagram(), ok_put()).await;
    let svc = service(base_url);

    let err = svc
        .call(
            "update_dictionary",
            json!({"id": "d1", "action": "create", "name": "是否2", "code": "yes_no"}),
        )
        .await
        .unwrap_err();
    assert_eq!(err.code, "VALIDATION_ERROR");
    assert!(err.message.contains("已存在"), "错误应说明编码冲突: {}", err.message);
    assert_eq!(put_count(&captured), 0, "编码冲突不得发出 PUT（零写请求）");
}

// ── UT-MCP-34：update 改码同步引用 + delete 级联置空 ─────────────────────

#[tokio::test]
async fn ut_mcp34_update_code_syncs_field_refs() {
    record(&["UT-MCP-34"]);
    let (base_url, captured, _h) = mock_server(base_diagram(), ok_put()).await;
    let svc = service(base_url);

    let resp = svc
        .call(
            "update_dictionary",
            json!({"id": "d1", "action": "update", "dict_id": "dict-1", "code": "yn", "name": "是否标记"}),
        )
        .await
        .expect("update 应成功");
    assert_eq!(resp["dict_id"], "dict-1");
    assert_eq!(resp["fields_cleared"], 0);

    let body = put_body(&captured);
    let diagram = &body["diagram"];
    assert_eq!(diagram["dictionaries"][0]["code"], "yn");
    assert_eq!(diagram["dictionaries"][0]["name"], "是否标记");
    // 改码同步引用：f1 的 dict_code 从 yes_no 改写为 yn（无悬空绑定）
    assert_eq!(diagram["tables"][0]["fields"][0]["dict_code"], "yn");
    assert_eq!(diagram["tables"][0]["fields"][1]["dict_code"], "", "未绑定字段不受影响");
}

#[tokio::test]
async fn ut_mcp34_update_code_conflict_no_put() {
    record(&["UT-MCP-34"]);
    let (base_url, captured, _h) = mock_server(base_diagram(), ok_put()).await;
    let svc = service(base_url);

    let err = svc
        .call(
            "update_dictionary",
            json!({"id": "d1", "action": "update", "dict_id": "dict-1", "code": "gender"}),
        )
        .await
        .unwrap_err();
    assert_eq!(err.code, "VALIDATION_ERROR");
    assert_eq!(put_count(&captured), 0, "改码冲突不得发出 PUT");
}

#[tokio::test]
async fn ut_mcp34_delete_cascade_clears_fields() {
    record(&["UT-MCP-34"]);
    let (base_url, captured, _h) = mock_server(base_diagram(), ok_put()).await;
    let svc = service(base_url);

    let resp = svc
        .call(
            "update_dictionary",
            json!({"id": "d1", "action": "delete", "dict_id": "dict-1"}),
        )
        .await
        .expect("delete 应成功");
    assert_eq!(resp["dict_id"], "dict-1");
    assert_eq!(resp["fields_cleared"], 1, "f1 引用 yes_no，应级联置空 1 处");

    let body = put_body(&captured);
    let diagram = &body["diagram"];
    let dicts = diagram["dictionaries"].as_array().unwrap();
    assert_eq!(dicts.len(), 1, "字典应移除");
    assert_eq!(dicts[0]["code"], "gender");
    assert_eq!(diagram["tables"][0]["fields"][0]["dict_code"], "", "引用字段 dict_code 应置空");
}

#[tokio::test]
async fn ut_mcp34_dict_id_not_found_zero_request() {
    record(&["UT-MCP-34"]);
    // update/delete 缺 dict_id → 本地拒绝，零请求
    let svc = service("http://127.0.0.1:0".into());
    for args in [
        json!({"id": "d1", "action": "update"}),
        json!({"id": "d1", "action": "delete"}),
    ] {
        let err = svc.call("update_dictionary", args).await.unwrap_err();
        assert_eq!(err.code, "VALIDATION_ERROR");
    }
    // dict_id 不存在 → GET 后拒绝，零 PUT
    let (base_url, captured, _h) = mock_server(base_diagram(), ok_put()).await;
    let svc = service(base_url);
    let err = svc
        .call(
            "update_dictionary",
            json!({"id": "d1", "action": "delete", "dict_id": "dict-999"}),
        )
        .await
        .unwrap_err();
    assert_eq!(err.code, "VALIDATION_ERROR");
    assert!(err.message.contains("不存在"));
    assert_eq!(put_count(&captured), 0, "dict_id 不存在不得发出 PUT");
}

// ── UT-MCP-35：update_field dict_code 绑定/解绑/悬空拒绝 ─────────────────

#[tokio::test]
async fn ut_mcp35_bind_existing_dict_code() {
    record(&["UT-MCP-35"]);
    let (base_url, captured, _h) = mock_server(base_diagram(), ok_put()).await;
    let svc = service(base_url);

    let resp = svc
        .call(
            "update_field",
            json!({"id": "d1", "table_id": "t1", "field_id": "f2", "dict_code": "gender"}),
        )
        .await
        .expect("绑定已存在编码应成功");
    assert_eq!(resp["revision"], 6);

    let body = put_body(&captured);
    assert_eq!(body["diagram"]["tables"][0]["fields"][1]["dict_code"], "gender");
    assert_eq!(body["diagram"]["tables"][0]["fields"][0]["dict_code"], "yes_no", "其他字段不受影响");
}

#[tokio::test]
async fn ut_mcp35_unbind_with_empty_string() {
    record(&["UT-MCP-35"]);
    let (base_url, captured, _h) = mock_server(base_diagram(), ok_put()).await;
    let svc = service(base_url);

    svc.call(
        "update_field",
        json!({"id": "d1", "table_id": "t1", "field_id": "f1", "dict_code": ""}),
    )
    .await
    .expect("空串解绑应成功");

    let body = put_body(&captured);
    assert_eq!(body["diagram"]["tables"][0]["fields"][0]["dict_code"], "", "空串应解绑置空");
}

#[tokio::test]
async fn ut_mcp35_dangling_dict_code_rejected_no_put() {
    record(&["UT-MCP-35"]);
    let (base_url, captured, _h) = mock_server(base_diagram(), ok_put()).await;
    let svc = service(base_url);

    let err = svc
        .call(
            "update_field",
            json!({"id": "d1", "table_id": "t1", "field_id": "f2", "dict_code": "not_exist"}),
        )
        .await
        .unwrap_err();
    assert_eq!(err.code, "VALIDATION_ERROR");
    assert!(err.message.contains("不存在"), "错误应说明字典不存在: {}", err.message);
    assert_eq!(put_count(&captured), 0, "悬空编码不得发出 PUT（杜绝 S07 EX-7.4）");
}
