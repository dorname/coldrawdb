use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Instant;

use coldrawdb_mcp::api::ApiClient;
use coldrawdb_mcp::reporter;
use coldrawdb_mcp::Config;
use futures_util::{SinkExt, StreamExt};
use serde_json::{json, Value};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio_tungstenite::tungstenite::Message;

#[derive(Clone)]
struct Reply {
    status: &'static str,
    body: Value,
    cookie: Option<&'static str>,
}

impl Reply {
    fn new(status: &'static str, body: Value) -> Self {
        Self {
            status,
            body,
            cookie: None,
        }
    }

    fn with_cookie(mut self, cookie: &'static str) -> Self {
        self.cookie = Some(cookie);
        self
    }
}

enum Step {
    Http(Reply),
    DropConnection,
    WsReject401,
    Ws { op_error: bool },
}

#[derive(Clone)]
struct RecordedRequest {
    method: String,
    path: String,
    raw: String,
    body: Value,
}

async fn read_http(stream: &mut TcpStream) -> RecordedRequest {
    let mut bytes = Vec::new();
    let (header_end, content_length) = loop {
        let mut chunk = [0_u8; 4096];
        let count = stream.read(&mut chunk).await.unwrap();
        assert!(count > 0, "请求提前结束");
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
    let raw = String::from_utf8_lossy(&bytes).to_string();
    let headers = String::from_utf8_lossy(&bytes[..header_end]);
    let mut parts = headers.lines().next().unwrap().split_whitespace();
    let method = parts.next().unwrap().to_string();
    let path = parts.next().unwrap().to_string();
    let body = if content_length == 0 {
        json!({})
    } else {
        serde_json::from_slice(&bytes[header_end..header_end + content_length]).unwrap()
    };
    RecordedRequest {
        method,
        path,
        raw,
        body,
    }
}

fn http_response(reply: &Reply) -> String {
    let payload = reply.body.to_string();
    let cookie = reply
        .cookie
        .map(|cookie| format!("Set-Cookie: {cookie}\r\n"))
        .unwrap_or_default();
    format!(
        "HTTP/1.1 {}\r\nContent-Type: application/json\r\n{}Content-Length: {}\r\nConnection: close\r\n\r\n{payload}",
        reply.status,
        cookie,
        payload.len()
    )
}

async fn script_server(
    steps: Vec<Step>,
) -> (String, Arc<Mutex<Vec<RecordedRequest>>>, Arc<AtomicUsize>) {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let requests = Arc::new(Mutex::new(Vec::new()));
    let op_sends = Arc::new(AtomicUsize::new(0));
    let recorded = requests.clone();
    let sent = op_sends.clone();
    tokio::spawn(async move {
        for step in steps {
            let (mut stream, _) = listener.accept().await.unwrap();
            match step {
                Step::Http(reply) => {
                    let request = read_http(&mut stream).await;
                    recorded.lock().unwrap().push(request);
                    stream
                        .write_all(http_response(&reply).as_bytes())
                        .await
                        .unwrap();
                }
                Step::DropConnection => {
                    let request = read_http(&mut stream).await;
                    recorded.lock().unwrap().push(request);
                    drop(stream);
                }
                Step::WsReject401 => {
                    let request = read_http(&mut stream).await;
                    recorded.lock().unwrap().push(request);
                    let body = json!({"message":"unauthorized"}).to_string();
                    let response = format!(
                        "HTTP/1.1 401 Unauthorized\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
                        body.len()
                    );
                    stream.write_all(response.as_bytes()).await.unwrap();
                }
                Step::Ws { op_error } => {
                    let mut head = vec![0_u8; 8192];
                    let count = stream.peek(&mut head).await.unwrap();
                    let raw = String::from_utf8_lossy(&head[..count]).to_string();
                    let path = raw
                        .lines()
                        .next()
                        .and_then(|line| line.split_whitespace().nth(1))
                        .unwrap_or("")
                        .to_string();
                    recorded.lock().unwrap().push(RecordedRequest {
                        method: "WS".into(),
                        path,
                        raw,
                        body: json!({}),
                    });
                    let mut socket = tokio_tungstenite::accept_async(stream).await.unwrap();
                    let message = socket.next().await.unwrap().unwrap();
                    let Message::Text(frame) = message else {
                        panic!("WS 首帧不是 op 文本")
                    };
                    let frame: Value = serde_json::from_str(&frame).unwrap();
                    assert_eq!(frame["type"], "op");
                    sent.fetch_add(1, Ordering::SeqCst);
                    let response = if op_error {
                        json!({"type":"error","code":"READ_ONLY","message":"只读成员不能提交 op"})
                    } else {
                        json!({"type":"ack","serverRev":5})
                    };
                    socket
                        .send(Message::Text(response.to_string().into()))
                        .await
                        .unwrap();
                }
            }
        }
    });
    (format!("http://{address}"), requests, op_sends)
}

fn static_config(base_url: String) -> Config {
    Config::from_values(HashMap::from([
        ("COLDRAWDB_BASE_URL".into(), base_url),
        ("COLDRAWDB_ACCESS_TOKEN".into(), "secret-token".into()),
    ]))
    .unwrap()
}

fn dynamic_config(base_url: String) -> Config {
    Config::from_values(HashMap::from([
        ("COLDRAWDB_BASE_URL".into(), base_url),
        ("COLDRAWDB_AUTH_EMAIL".into(), "mcp@example.test".into()),
        ("COLDRAWDB_AUTH_PASSWORD".into(), "room-password".into()),
    ]))
    .unwrap()
}

fn base_diagram() -> Value {
    json!({
        "id":"d1","revision":4,
        "tables":[{"id":"t1","name":"users","fields":[{"id":"f1","name":"id"}]}],
        "references":[],"areas":[],"notes":[]
    })
}

fn desired_diagram() -> Value {
    let mut desired = base_diagram();
    desired["tables"][0]["name"] = json!("accounts");
    desired
}

fn login_reply() -> Reply {
    Reply::new(
        "200 OK",
        json!({"accessToken":"token-old","expiresIn":300,"tokenType":"Bearer"}),
    )
    .with_cookie("refresh_token=cookie-old; Path=/api/v1/auth; HttpOnly")
}

fn refresh_reply() -> Reply {
    Reply::new(
        "200 OK",
        json!({"accessToken":"token-new","expiresIn":300,"tokenType":"Bearer"}),
    )
    .with_cookie("refresh_token=cookie-new; Path=/api/v1/auth; HttpOnly")
}

fn use_op_channel() -> Reply {
    Reply::new(
        "409 Conflict",
        json!({"message":"USE_OP_CHANNEL","details":{"code":"USE_OP_CHANNEL"}}),
    )
}

fn rooms_reply() -> Reply {
    Reply::new(
        "200 OK",
        json!({"items":[{"id":"r1","diagramId":"d1"}],"total":1}),
    )
}

fn diagram_reply() -> Reply {
    Reply::new("200 OK", json!({"code":0,"data":base_diagram()}))
}

#[tokio::test]
async fn ut_mcp_45_missing_room_credentials_fail_before_rooms() {
    let started = Instant::now();
    let (base_url, requests, _) = script_server(vec![
        Step::Http(Reply::new(
            "200 OK",
            json!({"data":{"id":"d1","revision":2}}),
        )),
        Step::Http(use_op_channel()),
        Step::Http(Reply::new(
            "401 Unauthorized",
            json!({"message":"Access token expired"}),
        )),
    ])
    .await;
    let api = ApiClient::new(
        Config::from_values(HashMap::from([("COLDRAWDB_BASE_URL".into(), base_url)])).unwrap(),
    )
    .unwrap();

    let personal = api.update("d1", 1, base_diagram()).await.unwrap();
    assert_eq!(personal["revision"], 2);
    let missing = api.update("d1", 4, desired_diagram()).await.unwrap_err();
    assert_eq!(missing.code, "ROOMS_AUTH_REQUIRED");
    assert_eq!(missing.details.unwrap()["reason"], "TOKEN_MISSING");
    let ordinary = api.get("d1").await.unwrap_err();
    assert_eq!(ordinary.code, "UNAUTHENTICATED");

    let requests = requests.lock().unwrap();
    assert_eq!(requests.len(), 3);
    assert!(!requests
        .iter()
        .any(|request| request.path == "/api/v1/rooms"));
    reporter::report("UT-MCP-45", Ok(()), started.elapsed().as_millis());
}

#[tokio::test]
async fn ut_mcp_46_rooms_401_is_contextual_and_redacted() {
    let started = Instant::now();
    let unauthorized = json!({
        "code":"UNAUTHORIZED",
        "message":"401 secret-token",
        "request_id":"req-rooms-safe",
        "details":{
            "password":"room-password",
            "cookie":"refresh_token=room-cookie",
            "authorization":"Bearer secret-token"
        }
    });
    let (base_url, requests, _) = script_server(vec![
        Step::Http(use_op_channel()),
        Step::Http(Reply::new("401 Unauthorized", unauthorized)),
        Step::Http(use_op_channel()),
        Step::Http(Reply::new("403 Forbidden", json!({"message":"forbidden"}))),
        Step::Http(use_op_channel()),
        Step::Http(Reply::new(
            "500 Internal Server Error",
            json!({"message":"database unavailable"}),
        )),
    ])
    .await;
    let api = ApiClient::new(static_config(base_url)).unwrap();

    let error = api.update("d1", 4, desired_diagram()).await.unwrap_err();
    assert_eq!(error.code, "ROOMS_AUTH_REQUIRED");
    assert_eq!(error.request_id.as_deref(), Some("req-rooms-safe"));
    let details = error.details.clone().unwrap();
    assert_eq!(details["reason"], "TOKEN_INVALID_OR_EXPIRED");
    assert_eq!(details["upstream_code"], "UNAUTHENTICATED");
    assert_eq!(details["http_status"], 401);
    let serialized = serde_json::to_string(&error).unwrap();
    for secret in ["secret-token", "room-password", "room-cookie"] {
        assert!(!serialized.contains(secret), "错误结果泄漏敏感值");
    }

    let forbidden = api.update("d1", 4, desired_diagram()).await.unwrap_err();
    assert_eq!(forbidden.code, "PERMISSION_DENIED");
    let upstream = api.update("d1", 4, desired_diagram()).await.unwrap_err();
    assert_eq!(upstream.code, "UPSTREAM_ERROR");
    assert_eq!(requests.lock().unwrap().len(), 6);
    reporter::report("UT-MCP-46", Ok(()), started.elapsed().as_millis());
}

#[tokio::test]
async fn ut_mcp_47_login_refresh_retry_and_latest_ws_token() {
    let started = Instant::now();
    for values in [
        HashMap::from([
            ("COLDRAWDB_BASE_URL".into(), "http://127.0.0.1:1".into()),
            ("COLDRAWDB_ACCESS_TOKEN".into(), "static".into()),
            ("COLDRAWDB_AUTH_EMAIL".into(), "mcp@example.test".into()),
            ("COLDRAWDB_AUTH_PASSWORD".into(), "room-password".into()),
        ]),
        HashMap::from([
            ("COLDRAWDB_BASE_URL".into(), "http://127.0.0.1:1".into()),
            ("COLDRAWDB_AUTH_EMAIL".into(), "mcp@example.test".into()),
        ]),
    ] {
        assert_eq!(
            Config::from_values(values).unwrap_err().code,
            "CONFIG_INVALID"
        );
    }
    let config = dynamic_config("http://127.0.0.1:1".into());
    let debug = format!("{config:?}");
    assert!(!debug.contains("room-password"));
    assert!(!debug.contains("mcp@example.test"));

    let (base_url, requests, op_sends) = script_server(vec![
        Step::Http(login_reply()),
        Step::Http(Reply::new("401 Unauthorized", json!({"message":"expired"}))),
        Step::Http(refresh_reply()),
        Step::Http(use_op_channel()),
        Step::Http(rooms_reply()),
        Step::Http(diagram_reply()),
        Step::Ws { op_error: false },
    ])
    .await;
    let api = ApiClient::new(dynamic_config(base_url)).unwrap();
    let saved = api.update("d1", 4, desired_diagram()).await.unwrap();
    assert_eq!(saved["revision"], 5);
    assert_eq!(op_sends.load(Ordering::SeqCst), 1);

    let requests = requests.lock().unwrap();
    assert_eq!(requests.len(), 7);
    assert_eq!(requests[0].path, "/api/v1/auth/login");
    assert_eq!(requests[0].body["password"], "room-password");
    assert_eq!(
        requests
            .iter()
            .filter(|request| request.path == "/api/v1/auth/refresh")
            .count(),
        1
    );
    assert!(requests[2]
        .raw
        .to_ascii_lowercase()
        .contains("cookie: refresh_token=cookie-old"));
    assert_eq!(
        requests
            .iter()
            .filter(|request| request.method == "PUT")
            .count(),
        2
    );
    for index in [3, 4, 5] {
        assert!(requests[index]
            .raw
            .to_ascii_lowercase()
            .contains("authorization: bearer token-new"));
    }
    assert_eq!(requests[6].method, "WS");
    assert!(requests[6].path.contains("token=token-new"));
    assert!(!requests[6].path.contains("token-old"));
    reporter::report("UT-MCP-47", Ok(()), started.elapsed().as_millis());
}

#[tokio::test]
async fn ut_mcp_48_refresh_failure_and_ws_no_replay_boundaries() {
    let started = Instant::now();
    let (base_url, requests, _) = script_server(vec![
        Step::Http(login_reply()),
        Step::Http(Reply::new("401 Unauthorized", json!({"message":"expired"}))),
        Step::Http(Reply::new(
            "401 Unauthorized",
            json!({"message":"refresh invalid","password":"room-password"}),
        )),
    ])
    .await;
    let api = ApiClient::new(dynamic_config(base_url)).unwrap();
    let error = api.update("d1", 4, desired_diagram()).await.unwrap_err();
    assert_eq!(error.code, "UNAUTHENTICATED");
    assert!(!serde_json::to_string(&error)
        .unwrap()
        .contains("room-password"));
    {
        let requests_seen = requests.lock().unwrap();
        assert_eq!(requests_seen.len(), 3);
        assert_eq!(
            requests_seen
                .iter()
                .filter(|request| request.method == "PUT")
                .count(),
            1
        );
    }

    let (base_url, network_requests, _) = script_server(vec![
        Step::Http(login_reply()),
        Step::Http(Reply::new("401 Unauthorized", json!({"message":"expired"}))),
        Step::DropConnection,
    ])
    .await;
    let api = ApiClient::new(dynamic_config(base_url)).unwrap();
    let network_error = api.update("d1", 4, desired_diagram()).await.unwrap_err();
    assert_eq!(network_error.code, "UPSTREAM_UNREACHABLE");
    assert_eq!(network_requests.lock().unwrap().len(), 3);

    let (base_url, requests, op_sends) = script_server(vec![
        Step::Http(login_reply()),
        Step::Http(use_op_channel()),
        Step::Http(rooms_reply()),
        Step::Http(diagram_reply()),
        Step::WsReject401,
        Step::Http(refresh_reply()),
        Step::Ws { op_error: true },
    ])
    .await;
    let api = ApiClient::new(dynamic_config(base_url)).unwrap();
    let error = api.update("d1", 4, desired_diagram()).await.unwrap_err();
    assert_eq!(error.code, "READ_ONLY");
    assert_eq!(op_sends.load(Ordering::SeqCst), 1, "op 失败后不得重放");
    let requests = requests.lock().unwrap();
    assert_eq!(requests.len(), 7);
    assert_eq!(requests[4].method, "GET");
    assert!(requests[4].path.starts_with("/ws/rooms/r1"));
    assert_eq!(requests[5].path, "/api/v1/auth/refresh");
    assert_eq!(requests[6].method, "WS");
    assert!(requests[6].path.contains("token=token-new"));
    assert_eq!(
        requests
            .iter()
            .filter(|request| request.path == "/api/v1/auth/refresh")
            .count(),
        1
    );
    reporter::report("UT-MCP-48", Ok(()), started.elapsed().as_millis());
}
