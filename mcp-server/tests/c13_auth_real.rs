use std::io::{BufRead, BufReader, Write};
use std::net::TcpListener as StdTcpListener;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};

use coldrawdb_mcp::reporter;
use serde_json::{json, Value};

struct ChildGuard(Child);

impl Drop for ChildGuard {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

fn backend_binary() -> PathBuf {
    if let Some(path) = std::env::var_os("COLDRAWDB_BACKEND_BIN") {
        return PathBuf::from(path);
    }
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
    let debug = root.join("backend/target/debug/backend");
    if debug.is_file() {
        debug
    } else {
        root.join("backend/target/release/backend")
    }
}

fn reserve_port() -> u16 {
    StdTcpListener::bind("127.0.0.1:0")
        .unwrap()
        .local_addr()
        .unwrap()
        .port()
}

async fn start_backend(
    db_path: &Path,
    static_dir: &Path,
    port: u16,
    jwt_secret: &str,
    diagrams_auth: bool,
) -> ChildGuard {
    let backend_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .join("backend");
    let child = Command::new(backend_binary())
        .current_dir(backend_root)
        .env("COLDRAWDB_BIND_ADDR", format!("127.0.0.1:{port}"))
        .env(
            "COLDRAWDB_DB_URL",
            format!("sqlite://{}?mode=rwc", db_path.display()),
        )
        .env(
            "COLDRAWDB_DIAGRAMS_AUTH",
            if diagrams_auth { "on" } else { "off" },
        )
        .env("COLDRAWDB_ACCESS_TTL_SECS", "300")
        .env("COLDRAWDB_JWT_SECRET", jwt_secret)
        .env("COLDRAWDB_STATIC_DIR", static_dir)
        .env("RUST_LOG", "warn")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .unwrap();
    let mut child = ChildGuard(child);
    let base_url = format!("http://127.0.0.1:{port}");
    let client = reqwest::Client::builder().no_proxy().build().unwrap();
    let deadline = Instant::now() + Duration::from_secs(30);
    loop {
        if let Some(status) = child.0.try_wait().unwrap() {
            panic!("真实 backend 提前退出: {status}");
        }
        if let Ok(response) = client
            .get(format!("{base_url}/api/v1/diagrams/health"))
            .send()
            .await
        {
            if response.status().is_success()
                || response.status() == reqwest::StatusCode::UNAUTHORIZED
            {
                return child;
            }
        }
        assert!(Instant::now() < deadline, "等待真实 backend 超时");
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
}

async fn restart_backend(
    child: &mut ChildGuard,
    db_path: &Path,
    static_dir: &Path,
    port: u16,
    jwt_secret: &str,
    diagrams_auth: bool,
) {
    let _ = child.0.kill();
    let _ = child.0.wait();
    *child = start_backend(db_path, static_dir, port, jwt_secret, diagrams_auth).await;
}

async fn setup_room(base_url: &str, case_name: &str) -> (String, Value) {
    let client = reqwest::Client::builder().no_proxy().build().unwrap();
    let email = format!("{case_name}@example.test");
    let password = "RealPass123";
    let response = client
        .post(format!("{base_url}/api/v1/auth/register"))
        .json(&json!({"email":email,"password":password,"displayName":"ST"}))
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), 201);
    let response = client
        .post(format!("{base_url}/api/v1/auth/login"))
        .json(&json!({"email":email,"password":password}))
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), 200);
    let login: Value = response.json().await.unwrap();
    let token = login["accessToken"].as_str().unwrap();

    let response = client
        .post(format!("{base_url}/api/v1/diagrams"))
        .bearer_auth(token)
        .json(&json!({"name":case_name,"database":"postgresql"}))
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), 200);
    let created: Value = response.json().await.unwrap();
    let diagram_id = created["data"]["id"].as_str().unwrap().to_string();
    let response = client
        .get(format!("{base_url}/api/v1/diagrams/{diagram_id}"))
        .bearer_auth(token)
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), 200);
    let current: Value = response.json().await.unwrap();
    let mut diagram = current["data"].clone();
    let revision = diagram["revision"].as_i64().unwrap();
    diagram["tables"] = json!([{
        "id":"table-real","name":"before","comment":"room fixture","x":20.0,"y":30.0,
        "fields":[{"id":"field-real","name":"id","type":"INT","primary":true,"not_null":true}]
    }]);
    diagram["references"] = json!([]);
    diagram["areas"] = json!([{"id":"area-real","name":"area","tables":["table-real"]}]);
    diagram["notes"] = json!([{"id":"note-real","title":"note","content":"keep"}]);
    let response = client
        .put(format!("{base_url}/api/v1/diagrams/{diagram_id}"))
        .bearer_auth(token)
        .json(&json!({"expected_revision":revision,"diagram":diagram}))
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), 200);
    let saved: Value = response.json().await.unwrap();
    let response = client
        .post(format!("{base_url}/api/v1/rooms"))
        .bearer_auth(token)
        .json(&json!({"name":case_name,"diagramId":diagram_id}))
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), 201);

    let response = client
        .get(format!("{base_url}/api/v1/diagrams/{diagram_id}"))
        .bearer_auth(token)
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), 200);
    let current: Value = response.json().await.unwrap();
    assert_eq!(saved["data"]["revision"], current["data"]["revision"]);
    (diagram_id, current["data"].clone())
}

async fn issue_access_token(base_url: &str, email: &str) -> String {
    let client = reqwest::Client::builder().no_proxy().build().unwrap();
    let response = client
        .post(format!("{base_url}/api/v1/auth/register"))
        .json(&json!({"email":email,"password":"RealPass123","displayName":"ST"}))
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), 201);
    let response = client
        .post(format!("{base_url}/api/v1/auth/login"))
        .json(&json!({"email":email,"password":"RealPass123"}))
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), 200);
    let login: Value = response.json().await.unwrap();
    login["accessToken"].as_str().unwrap().to_string()
}

async fn bind_room(base_url: &str, token: &str, diagram_id: &str, name: &str) {
    let client = reqwest::Client::builder().no_proxy().build().unwrap();
    let response = client
        .post(format!("{base_url}/api/v1/rooms"))
        .bearer_auth(token)
        .json(&json!({"name":name,"diagramId":diagram_id}))
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), 201);
}

struct StdioSession {
    next_id: i64,
    stdin: std::process::ChildStdin,
    reader: BufReader<std::process::ChildStdout>,
}

impl StdioSession {
    fn call(&mut self, method: &str, params: Value) -> Value {
        let id = self.next_id;
        self.next_id += 1;
        writeln!(
            self.stdin,
            "{}",
            json!({"jsonrpc":"2.0","id":id,"method":method,"params":params})
        )
        .unwrap();
        self.stdin.flush().unwrap();
        let mut line = String::new();
        assert!(self.reader.read_line(&mut line).unwrap() > 0);
        let response: Value = serde_json::from_str(&line).unwrap();
        assert_eq!(response["id"], id);
        assert!(response.get("error").is_none());
        response["result"].clone()
    }

    fn tool_result(&mut self, name: &str, arguments: Value) -> Value {
        self.call("tools/call", json!({"name":name,"arguments":arguments}))
    }

    fn tool(&mut self, name: &str, arguments: Value) -> Value {
        let result = self.tool_result(name, arguments);
        assert_eq!(result["isError"], false, "真实工具调用失败: {result}");
        result["structuredContent"].clone()
    }
}

struct RealMcp {
    session: StdioSession,
    _child: ChildGuard,
}

impl RealMcp {
    fn new(base_url: &str, dynamic: bool) -> Self {
        let mut command = Command::new(env!("CARGO_BIN_EXE_coldrawdb-mcp"));
        command
            .env("COLDRAWDB_BASE_URL", base_url)
            .env("COLDRAWDB_AUTH_PROBE", "off")
            .env_remove("COLDRAWDB_ACCESS_TOKEN")
            .env_remove("COLDRAWDB_AUTH_EMAIL")
            .env_remove("COLDRAWDB_AUTH_PASSWORD")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null());
        if dynamic {
            command
                .env("COLDRAWDB_AUTH_EMAIL", "st-mcp-20@example.test")
                .env("COLDRAWDB_AUTH_PASSWORD", "RealPass123");
        }
        let mut child = ChildGuard(command.spawn().unwrap());
        let mut session = StdioSession {
            next_id: 1,
            stdin: child.0.stdin.take().unwrap(),
            reader: BufReader::new(child.0.stdout.take().unwrap()),
        };
        session.call(
            "initialize",
            json!({"protocolVersion":"2025-06-18","capabilities":{},"clientInfo":{"name":"auth-real","version":"1"}}),
        );
        Self {
            session,
            _child: child,
        }
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
#[ignore = "需要真实 backend 二进制；verify 预跑显式执行"]
async fn st_mcp_19_real_stdio_room_write_without_credentials() {
    let started = Instant::now();
    let temp = tempfile::tempdir().unwrap();
    let db_path = temp.path().join("st19.sqlite");
    let port = reserve_port();
    let base_url = format!("http://127.0.0.1:{port}");
    let _backend = start_backend(
        &db_path,
        &temp.path().join("no-static"),
        port,
        "st19-secret",
        false,
    )
    .await;
    let (diagram_id, mut desired) = setup_room(&base_url, "st-mcp-19").await;
    let revision = desired["revision"].as_i64().unwrap();
    desired["tables"][0]["name"] = json!("anonymous-must-fail");
    let mut mcp = RealMcp::new(&base_url, false);
    let result = tokio::task::block_in_place(|| {
        mcp.session.tool_result(
            "update_diagram",
            json!({"id":diagram_id,"expected_revision":revision,"diagram":desired}),
        )
    });
    assert_eq!(result["isError"], true);
    assert!(result.get("structuredContent").is_none());
    let text = result["content"][0]["text"].as_str().unwrap();
    let error: Value = serde_json::from_str(text).unwrap();
    assert_eq!(error["code"], "ROOMS_AUTH_REQUIRED");
    assert_eq!(error["details"]["reason"], "TOKEN_MISSING");
    assert!(!text.contains("RealPass123"));
    assert!(!text.contains("refresh_token="));
    reporter::report("ST-MCP-19", Ok(()), started.elapsed().as_millis());
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
#[ignore = "需要真实 backend 二进制；verify 预跑显式执行"]
async fn st_mcp_20_real_login_refresh_room_ack_and_cookie_rotation() {
    let started = Instant::now();
    let temp = tempfile::tempdir().unwrap();
    let db_path = temp.path().join("st20.sqlite");
    let static_dir = temp.path().join("no-static");
    let port = reserve_port();
    let base_url = format!("http://127.0.0.1:{port}");
    let mut backend = start_backend(&db_path, &static_dir, port, "st20-secret-a", true).await;
    let (diagram_id, _) = setup_room(&base_url, "st-mcp-20").await;
    let mut mcp = RealMcp::new(&base_url, true);

    // 首次调用完成真实 login 并把 access token / refresh cookie 留在 MCP 进程内。
    let initial = tokio::task::block_in_place(|| {
        mcp.session
            .tool("get_diagram", json!({"id":diagram_id.clone()}))
    });
    let mut desired = initial["diagram"].clone();
    let revision = desired["revision"].as_i64().unwrap();
    desired["tables"][0]["name"] = json!("after-refresh");

    // JWT 是无状态的；更换 secret 可确定性使首个 access token 失效，
    // 同一 SQLite 中的 opaque refresh token 仍然有效，无需等待最小 300 秒 TTL。
    restart_backend(
        &mut backend,
        &db_path,
        &static_dir,
        port,
        "st20-secret-b",
        true,
    )
    .await;
    let saved = tokio::task::block_in_place(|| {
        mcp.session.tool(
            "update_diagram",
            json!({"id":diagram_id.clone(),"expected_revision":revision,"diagram":desired}),
        )
    });
    assert!(saved["revision"].as_i64().unwrap() >= 1);
    let materialized = tokio::task::block_in_place(|| {
        mcp.session
            .tool("get_diagram", json!({"id":diagram_id.clone()}))
    });
    assert_eq!(
        materialized["diagram"]["tables"][0]["name"],
        "after-refresh"
    );
    assert_eq!(materialized["diagram"]["areas"][0]["id"], "area-real");
    assert_eq!(materialized["diagram"]["notes"][0]["id"], "note-real");

    // 再次使 access token 失效：只有保存了轮换后的 cookie，第二次 refresh 才能成功。
    restart_backend(
        &mut backend,
        &db_path,
        &static_dir,
        port,
        "st20-secret-c",
        true,
    )
    .await;
    let after_rotation = tokio::task::block_in_place(|| {
        mcp.session
            .tool("get_diagram", json!({"id":diagram_id.clone()}))
    });
    assert_eq!(
        after_rotation["diagram"]["tables"][0]["name"],
        "after-refresh"
    );
    let output = serde_json::to_string(&after_rotation).unwrap();
    assert!(!output.contains("RealPass123"));
    assert!(!output.contains("accessToken"));
    assert!(!output.contains("refresh_token="));
    reporter::report("ST-MCP-20", Ok(()), started.elapsed().as_millis());
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
#[ignore = "需要真实 backend 二进制；verify 预跑显式执行"]
async fn st_mcp_21_import_actual_ids_reference_and_first_room_op() {
    let started = Instant::now();
    let temp = tempfile::tempdir().unwrap();
    let db_path = temp.path().join("st21.sqlite");
    let port = reserve_port();
    let base_url = format!("http://127.0.0.1:{port}");
    let _backend = start_backend(
        &db_path,
        &temp.path().join("no-static"),
        port,
        "st21-secret",
        false,
    )
    .await;
    let token = issue_access_token(&base_url, "st-mcp-20@example.test").await;
    let mut mcp = RealMcp::new(&base_url, true);
    let payload = json!({
        "name":"ST-MCP-21","database":"postgresql",
        "tables":[
            {"id":"legacy-users","name":"users","comment":"keep users","x":10.0,"y":20.0,
             "fields":[{"id":"legacy-user-id","name":"id","type":"INT","primary":true}]},
            {"id":"legacy-orders","name":"orders","comment":"keep orders","x":300.0,"y":20.0,
             "fields":[{"id":"legacy-order-user","name":"user_id","type":"INT"}]}
        ],
        "references":[],
        "areas":[{"id":"legacy-area","name":"keep area","x":1.0,"y":2.0,"width":300.0,"height":200.0,"color":"#abcdef"}],
        "notes":[{"id":"legacy-note","title":"keep note","content":"preserve me","x":3.0,"y":4.0,"color":"#123456"}]
    });
    let imported = tokio::task::block_in_place(|| {
        mcp.session.tool(
            "import_schema",
            json!({"format":"drawdb_json","payload":payload}),
        )
    });
    let diagram_id = imported["diagram_id"].as_str().unwrap().to_string();
    let initial = tokio::task::block_in_place(|| {
        mcp.session
            .tool("get_diagram", json!({"id":diagram_id.clone()}))
    });
    let before = initial["diagram"].clone();
    let start_table_id = before["tables"][0]["id"].as_str().unwrap().to_string();
    let start_field_id = before["tables"][0]["fields"][0]["id"]
        .as_str()
        .unwrap()
        .to_string();
    let end_table_id = before["tables"][1]["id"].as_str().unwrap().to_string();
    let end_field_id = before["tables"][1]["fields"][0]["id"]
        .as_str()
        .unwrap()
        .to_string();

    let old_id_error = tokio::task::block_in_place(|| {
        mcp.session.tool_result(
            "update_reference",
            json!({"id":diagram_id.clone(),"action":"create",
                "start_table_id":"pre-import-users","start_field_id":"pre-import-user-id",
                "end_table_id":"pre-import-orders","end_field_id":"pre-import-order-user"}),
        )
    });
    assert_eq!(old_id_error["isError"], true);
    let error: Value =
        serde_json::from_str(old_id_error["content"][0]["text"].as_str().unwrap()).unwrap();
    assert_eq!(error["code"], "VALIDATION_ERROR");
    assert!(error["details"].get("endpoint").is_some());

    // 后续写入口只接受 GET 返回的实际 ID。
    tokio::task::block_in_place(|| {
        mcp.session.tool(
            "update_reference",
            json!({"id":diagram_id.clone(),"action":"create","ref_id":"reference-real",
                "start_table_id":start_table_id.clone(),"start_field_id":start_field_id.clone(),
                "end_table_id":end_table_id.clone(),"end_field_id":end_field_id.clone(),
                "cardinality":"one_to_many","on_delete":"cascade","on_update":"restrict",
                "name":"orders_user"}),
        )
    });
    let actual_reference = tokio::task::block_in_place(|| {
        mcp.session
            .tool("get_diagram", json!({"id":diagram_id.clone()}))
    })["diagram"]
        .clone();
    assert_eq!(actual_reference["references"].as_array().unwrap().len(), 1);
    let reference_id = actual_reference["references"][0]["id"]
        .as_str()
        .unwrap()
        .to_string();

    bind_room(&base_url, &token, &diagram_id, "ST-MCP-21").await;
    tokio::task::block_in_place(|| {
        mcp.session.tool(
            "update_table",
            json!({"id":diagram_id.clone(),"table_id":start_table_id.clone(),"name":"users_after_room_op"}),
        )
    });
    let final_result = tokio::task::block_in_place(|| {
        mcp.session
            .tool("get_diagram", json!({"id":diagram_id.clone()}))
    });
    let final_diagram = &final_result["diagram"];
    assert_eq!(final_diagram["tables"][0]["name"], "users_after_room_op");
    assert_eq!(final_diagram["references"].as_array().unwrap().len(), 1);
    let reference = &final_diagram["references"][0];
    assert_eq!(reference["id"], reference_id);
    assert_eq!(reference["start_table_id"], start_table_id);
    assert_eq!(reference["start_field_id"], start_field_id);
    assert_eq!(reference["end_table_id"], end_table_id);
    assert_eq!(reference["end_field_id"], end_field_id);
    assert_eq!(final_diagram["areas"], before["areas"]);
    assert_eq!(final_diagram["notes"], before["notes"]);
    assert_eq!(final_diagram["tables"][0]["fields"][0]["id"], start_field_id);
    assert_eq!(final_diagram["tables"][1]["fields"][0]["id"], end_field_id);
    reporter::report("ST-MCP-21", Ok(()), started.elapsed().as_millis());
}
