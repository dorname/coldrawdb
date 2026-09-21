//! 协作写入的纯逻辑：房间查找、op 帧、入站帧解释、stderr 是否输出。
//! 落地目标：mcp-server（guard 释放后接入 ApiClient::update 与 protocol.rs）。

pub mod diff;
pub mod session;
pub mod transport;

use serde_json::{json, Value};

pub fn find_room_id(rooms: &Value, diagram_id: &str) -> Option<String> {
    let items = rooms.get("items").and_then(Value::as_array)?;
    items.iter().find_map(|room| {
        let bound = room.get("diagramId").and_then(Value::as_str)?;
        if bound == diagram_id && room.get("archivedAt").and_then(Value::as_str).is_none() {
            room.get("id").and_then(Value::as_str).map(str::to_string)
        } else {
            None
        }
    })
}

pub fn build_op_frame(client_rev: i64, op: &Value) -> Value {
    json!({"type":"op","clientRev":client_rev,"op":op})
}

#[derive(Debug, PartialEq)]
pub enum Inbound {
    Ignore,
    Ack { server_rev: i64 },
    Failed { code: String, message: String },
}

pub fn interpret_frame(frame: &Value) -> Inbound {
    match frame.get("type").and_then(Value::as_str) {
        Some("ack") => {
            let server_rev = frame.get("serverRev").and_then(Value::as_i64).unwrap_or(0);
            if server_rev >= 1 {
                Inbound::Ack { server_rev }
            } else {
                Inbound::Failed {
                    code: "UPSTREAM_ERROR".into(),
                    message: "ack 缺少有效 serverRev".into(),
                }
            }
        }
        Some("error") => Inbound::Failed {
            code: frame
                .get("code")
                .and_then(Value::as_str)
                .unwrap_or("UPSTREAM_ERROR")
                .to_string(),
            message: frame
                .get("message")
                .and_then(Value::as_str)
                .unwrap_or("协作写入失败")
                .to_string(),
        },
        _ => Inbound::Ignore,
    }
}

/// 按发送顺序消费入站帧：忽略 connected/remote_op，每条 op 必须等到 ack。
/// 返回最后一条 ack 的 serverRev。空 op 列表返回 `current_rev`（无变更）。
pub fn drive_ops(ops: &[Value], inbound: &[Value], current_rev: i64) -> Result<i64, (String, String)> {
    if ops.is_empty() {
        return Ok(current_rev);
    }
    let mut frames = inbound.iter();
    let mut last = current_rev;
    for (index, op) in ops.iter().enumerate() {
        let _frame = build_op_frame((index as i64) + 1, op);
        loop {
            let Some(frame) = frames.next() else {
                return Err(("UPSTREAM_ERROR".into(), "协作通道在 ack 前关闭".into()));
            };
            match interpret_frame(frame) {
                Inbound::Ignore => continue,
                Inbound::Ack { server_rev } => {
                    last = server_rev;
                    break;
                }
                Inbound::Failed { code, message } => return Err((code, message)),
            }
        }
    }
    Ok(last)
}

/// 由 HTTP base URL 得到房间协作 WebSocket 地址。token 只放在 query，错误信息不得回显 token。
pub fn collab_ws_url(base_url: &str, room_id: &str, token: &str) -> Result<String, String> {
    if token.is_empty() {
        return Err("UNAUTHENTICATED".into());
    }
    let mut url = url::Url::parse(base_url).map_err(|_| "CONFIG_INVALID".to_string())?;
    match url.scheme() {
        "http" => url.set_scheme("ws").map_err(|_| "CONFIG_INVALID".to_string())?,
        "https" => url.set_scheme("wss").map_err(|_| "CONFIG_INVALID".to_string())?,
        "ws" | "wss" => {}
        _ => return Err("CONFIG_INVALID".into()),
    }
    url.set_path(&format!("/ws/rooms/{room_id}"));
    url.query_pairs_mut().append_pair("token", token);
    Ok(url.to_string())
}

/// PUT 收到 USE_OP_CHANNEL 之后，决定要连接的房间。不回显 token。
pub fn prepare_room_write(token: Option<&str>, rooms: &Value, diagram_id: &str) -> Result<String, (String, String)> {
    let Some(token) = token.filter(|value| !value.is_empty()) else {
        return Err((
            "UNAUTHENTICATED".into(),
            "房间图写入需要 COLDRAWDB_ACCESS_TOKEN".into(),
        ));
    };
    let _ = token;
    find_room_id(rooms, diagram_id).ok_or_else(|| {
        (
            "NOT_FOUND".into(),
            format!("没有找到绑定 diagram {diagram_id} 的活跃房间"),
        )
    })
}

/// 房间写入计划：房间、要发送的 op、当前 revision。无差异时 ops 为空。
pub fn plan_room_save(
    base_url: &str,
    token: Option<&str>,
    rooms: &Value,
    diagram_id: &str,
    base_revision: i64,
    base_diagram: &Value,
    desired: &Value,
) -> Result<(String, Vec<Value>, i64), (String, String)> {
    let room_id = prepare_room_write(token, rooms, diagram_id)?;
    let token = token.unwrap_or("");
    let ws_url = collab_ws_url(base_url, &room_id, token).map_err(|code| {
        (code, "无法构造协作 WebSocket 地址".into())
    })?;
    let _ = ws_url;
    let ops = diff::diff_diagrams(base_diagram, desired);
    Ok((room_id, ops, base_revision))
}

/// 连接失败时去掉 URL 中的 token，避免 stderr 泄露凭据。
pub fn redact_transport_error(ws_url: &str, error: &str) -> String {
    let mut cleaned = error.to_string();
    if let Ok(url) = url::Url::parse(ws_url) {
        if let Some(token) = url.query_pairs().find(|(key, _)| key == "token").map(|(_, value)| value.into_owned()) {
            if !token.is_empty() {
                cleaned = cleaned.replace(&token, "[REDACTED]");
            }
        }
        cleaned = cleaned.replace(ws_url, &redact_url(&url));
    }
    cleaned
}

fn redact_url(url: &url::Url) -> String {
    let mut clone = url.clone();
    clone.set_query(None);
    format!("{}?[REDACTED]", clone)
}

/// 409 且 details.code=USE_OP_CHANNEL 时不要当成 revision 冲突。
pub fn conflict_code(status: u16, body: &Value) -> Option<&'static str> {
    if status != 409 {
        return None;
    }
    match body.pointer("/details/code").and_then(Value::as_str) {
        Some("USE_OP_CHANNEL") => Some("USE_OP_CHANNEL"),
        _ => Some("REVISION_CONFLICT"),
    }
}
pub fn tool_log_line(status: &str, tool: &str, duration_ms: u128, code: Option<&str>, message: Option<&str>) -> Option<Value> {
    if status == "ok" {
        return None;
    }
    Some(json!({
        "event": "tool_call",
        "tool": tool,
        "duration_ms": duration_ms,
        "status": "error",
        "code": code.unwrap_or("INTERNAL_ERROR"),
        "message": message.unwrap_or("工具调用失败"),
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_active_room_by_diagram_id() {
        let rooms = json!({"items":[
            {"id":"r-old","diagramId":"d1","archivedAt":"2026-01-01T00:00:00Z"},
            {"id":"r-live","diagramId":"d1"}
        ],"total":2});
        assert_eq!(find_room_id(&rooms, "d1").as_deref(), Some("r-live"));
        assert_eq!(find_room_id(&rooms, "missing"), None);
    }

    #[test]
    fn ack_and_error_frames() {
        assert_eq!(
            interpret_frame(&json!({"type":"ack","serverRev":4})),
            Inbound::Ack { server_rev: 4 }
        );
        assert_eq!(
            interpret_frame(&json!({"type":"connected","serverRev":3})),
            Inbound::Ignore
        );
        assert_eq!(
            interpret_frame(&json!({"type":"error","code":"READ_ONLY","message":"只读成员不能提交 op"})),
            Inbound::Failed { code: "READ_ONLY".into(), message: "只读成员不能提交 op".into() }
        );
    }

    #[test]
    fn ok_calls_produce_no_stderr_line() {
        assert!(tool_log_line("ok", "list_diagrams", 7, None, None).is_none());
        let line = tool_log_line("error", "update_diagram", 4, Some("USE_OP_CHANNEL"), Some("房间图表请通过协作 op 通道写入")).unwrap();
        assert_eq!(line["status"], "error");
        assert_eq!(line["code"], "USE_OP_CHANNEL");
        assert!(line.get("diagram").is_none());
    }

    #[test]
    fn op_frame_shape() {
        let frame = build_op_frame(1, &json!({"type":"table.update","targetId":"t1","changes":{"name":"a"}}));
        assert_eq!(frame["type"], "op");
        assert_eq!(frame["clientRev"], 1);
        assert_eq!(frame["op"]["type"], "table.update");
    }

    #[test]
    fn drive_ops_skips_connected_and_returns_last_ack() {
        let ops = vec![json!({"type":"table.update","targetId":"t1"})];
        let inbound = vec![
            json!({"type":"connected","serverRev":3}),
            json!({"type":"ack","serverRev":4}),
        ];
        assert_eq!(drive_ops(&ops, &inbound, 3).unwrap(), 4);
    }

    #[test]
    fn drive_ops_surfaces_read_only() {
        let ops = vec![json!({"type":"table.update","targetId":"t1"})];
        let inbound = vec![json!({"type":"error","code":"READ_ONLY","message":"只读成员不能提交 op"})];
        let err = drive_ops(&ops, &inbound, 1).unwrap_err();
        assert_eq!(err.0, "READ_ONLY");
    }

    #[test]
    fn empty_diff_keeps_current_revision() {
        assert_eq!(drive_ops(&[], &[], 6).unwrap(), 6);
    }

    #[test]
    fn use_op_channel_is_not_revision_conflict() {
        let body = json!({"code":409,"message":"USE_OP_CHANNEL: 房间图表请通过协作 op 通道写入","details":{"code":"USE_OP_CHANNEL"}});
        assert_eq!(conflict_code(409, &body), Some("USE_OP_CHANNEL"));
        let conflict = json!({"code":409,"message":"revision conflict","details":{"current_revision":3}});
        assert_eq!(conflict_code(409, &conflict), Some("REVISION_CONFLICT"));
    }

    #[test]
    fn ws_url_switches_scheme_and_hides_token_from_errors() {
        let url = collab_ws_url("http://127.0.0.1:3000/api", "room-1", "secret-token").unwrap();
        assert!(url.starts_with("ws://127.0.0.1:3000/ws/rooms/room-1?"));
        assert!(url.contains("token=secret-token"));
        let err = collab_ws_url("http://127.0.0.1:3000", "room-1", "").unwrap_err();
        assert_eq!(err, "UNAUTHENTICATED");
        assert!(!err.contains("secret"));
    }

    #[test]
    fn prepare_room_write_requires_token_and_active_room() {
        let rooms = json!({"items":[{"id":"r1","diagramId":"d1"}]});
        let missing = prepare_room_write(None, &rooms, "d1").unwrap_err();
        assert_eq!(missing.0, "UNAUTHENTICATED");
        assert!(!missing.1.contains("secret-token"));
        assert_eq!(prepare_room_write(Some("secret-token"), &rooms, "d1").unwrap(), "r1");
        let absent = prepare_room_write(Some("secret-token"), &rooms, "d2").unwrap_err();
        assert_eq!(absent.0, "NOT_FOUND");
    }

    #[test]
    fn plan_room_save_emits_table_update_only() {
        let rooms = json!({"items":[{"id":"r1","diagramId":"d1"}]});
        let base = json!({"revision":4,"tables":[{"id":"t1","name":"users","fields":[{"id":"f1","name":"id"}]}]});
        let desired = json!({"revision":4,"tables":[{"id":"t1","name":"accounts","fields":[{"id":"f1","name":"id"}]}]});
        let (room_id, ops, rev) = plan_room_save(
            "http://127.0.0.1:3000",
            Some("secret-token"),
            &rooms,
            "d1",
            4,
            &base,
            &desired,
        )
        .unwrap();
        assert_eq!(room_id, "r1");
        assert_eq!(rev, 4);
        assert_eq!(ops.len(), 1);
        assert_eq!(ops[0]["type"], "table.update");
        assert!(ops[0]["changes"].get("fields").is_none());
    }

    #[test]
    fn transport_errors_do_not_contain_the_token() {
        let url = collab_ws_url("http://127.0.0.1:3000", "room-1", "secret-token").unwrap();
        let raw = format!("connect failed: {url}");
        let cleaned = redact_transport_error(&url, &raw);
        assert!(!cleaned.contains("secret-token"));
        assert!(cleaned.contains("[REDACTED]"));
    }
}
