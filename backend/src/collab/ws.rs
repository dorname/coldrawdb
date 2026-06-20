use actix::{Actor, ActorContext, AsyncContext, Handler, Message, StreamHandler};
use actix_web::{web, HttpRequest, HttpResponse};
use actix_web_actors::ws;
use sea_orm::DatabaseConnection;
use serde::Deserialize;
use serde_json::{json, Value};

use crate::auth::verify_access_token;
use crate::collab::{
    append_op, authorize_ws, get_collab_head, list_collab_ops, list_member_presence, CollabHub,
    CollabServiceError,
};

struct PushText(String);

impl Message for PushText {
    type Result = ();
}

struct CollabWsSession {
    room_id: String,
    user_id: String,
    role: String,
    db: DatabaseConnection,
    hub: CollabHub,
}

impl Handler<PushText> for CollabWsSession {
    type Result = ();

    fn handle(&mut self, msg: PushText, ctx: &mut Self::Context) {
        ctx.text(msg.0);
    }
}

impl CollabWsSession {
    fn send_json(&self, ctx: &mut ws::WebsocketContext<Self>, value: Value) {
        if let Ok(text) = serde_json::to_string(&value) {
            ctx.text(text);
        }
    }
}

impl Actor for CollabWsSession {
    type Context = ws::WebsocketContext<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        let room_id = self.room_id.clone();
        let user_id = self.user_id.clone();
        let role = self.role.clone();
        let db = self.db.clone();
        let hub = self.hub.clone();
        let addr = ctx.address();

        let room_id_sub = room_id.clone();
        let user_id_sub = user_id.clone();
        let addr_sub = addr.clone();
        actix_rt::spawn(async move {
            let mut rx = hub.subscribe(&room_id_sub);
            while let Ok(msg) = rx.recv().await {
                if msg
                    .except_user_id
                    .as_deref()
                    .is_some_and(|ex| ex == user_id_sub)
                {
                    continue;
                }
                addr_sub.do_send(PushText(msg.json));
            }
        });

        actix_rt::spawn(async move {
            match get_collab_head(&db, &room_id, &user_id).await {
                Ok(head) => {
                    let members = list_member_presence(&db, &room_id)
                        .await
                        .unwrap_or_default()
                        .into_iter()
                        .map(|(uid, display_name, member_role)| {
                            json!({
                                "userId": uid,
                                "displayName": display_name,
                                "role": member_role,
                                "online": uid == user_id,
                            })
                        })
                        .collect::<Vec<_>>();
                    let frame = json!({
                        "type": "connected",
                        "serverRev": head.server_rev,
                        "diagramId": head.diagram_id,
                        "snapshotHash": head.snapshot_hash,
                        "members": members,
                        "yourRole": role,
                    });
                    if let Ok(text) = serde_json::to_string(&frame) {
                        addr.do_send(PushText(text));
                    }
                }
                Err(_) => addr.do_send(PushText(
                    json!({"type":"error","code":"INTERNAL_ERROR","message":"无法加载协作头"}).to_string(),
                )),
            }
        });
    }
}

impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for CollabWsSession {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        match msg {
            Ok(ws::Message::Ping(bytes)) => ctx.pong(&bytes),
            Ok(ws::Message::Pong(_)) => {}
            Ok(ws::Message::Text(text)) => {
                let db = self.db.clone();
                let room_id = self.room_id.clone();
                let user_id = self.user_id.clone();
                let role = self.role.clone();
                let hub = self.hub.clone();
                let addr = ctx.address();
                actix_rt::spawn(async move {
                    let parsed: Result<Value, _> = serde_json::from_str(&text);
                    let Ok(frame) = parsed else {
                        addr.do_send(PushText(
                            json!({"type":"error","code":"INVALID_OP","message":"无效 JSON"})
                                .to_string(),
                        ));
                        return;
                    };
                    let Some(msg_type) = frame.get("type").and_then(|v| v.as_str()) else {
                        return;
                    };
                    match msg_type {
                        "op" => {
                            if role == "viewer" {
                                addr.do_send(PushText(
                                    json!({"type":"error","code":"READ_ONLY","message":"只读成员不能提交 op"})
                                        .to_string(),
                                ));
                                return;
                            }
                            let Some(op_obj) = frame.get("op") else {
                                addr.do_send(PushText(
                                    json!({"type":"error","code":"INVALID_OP","message":"缺少 op 字段"})
                                        .to_string(),
                                ));
                                return;
                            };
                            let op_type = op_obj
                                .get("type")
                                .and_then(|v| v.as_str())
                                .unwrap_or("")
                                .to_string();
                            if op_type.is_empty() {
                                addr.do_send(PushText(
                                    json!({"type":"error","code":"INVALID_OP","message":"缺少 op.type"})
                                        .to_string(),
                                ));
                                return;
                            }
                            match append_op(&db, &room_id, &user_id, &op_type, op_obj.clone()).await
                            {
                                Ok(result) => {
                                    let client_rev = frame.get("clientRev").cloned();
                                    let ack = json!({
                                        "type": "ack",
                                        "serverRev": result.server_rev,
                                        "clientRev": client_rev,
                                        "appliedOp": op_obj,
                                    });
                                    if let Ok(text) = serde_json::to_string(&ack) {
                                        addr.do_send(PushText(text));
                                    }
                                    let remote = json!({
                                        "type": "remote_op",
                                        "serverRev": result.server_rev,
                                        "authorId": user_id,
                                        "op": op_obj,
                                    });
                                    if let Ok(text) = serde_json::to_string(&remote) {
                                        hub.broadcast(&room_id, text, Some(user_id));
                                    }
                                }
                                Err(CollabServiceError::ReadOnly) => {
                                    addr.do_send(PushText(
                                        json!({"type":"error","code":"READ_ONLY","message":"只读成员不能提交 op"})
                                            .to_string(),
                                    ));
                                }
                                Err(e) => {
                                    addr.do_send(PushText(
                                        json!({"type":"error","code":"INTERNAL_ERROR","message": format!("{e:?}")})
                                            .to_string(),
                                    ));
                                }
                            }
                        }
                        "sync" => {
                            let last_rev = frame
                                .get("lastRev")
                                .and_then(|v| v.as_i64())
                                .unwrap_or(0);
                            match list_collab_ops(&db, &room_id, &user_id, last_rev, 500).await {
                                Ok((_, server_rev, items)) if items.is_empty() => {
                                    let head = get_collab_head(&db, &room_id, &user_id).await;
                                    let rev = head.map(|h| h.server_rev).unwrap_or(last_rev);
                                    addr.do_send(PushText(
                                        json!({"type":"sync","serverRev": rev, "ops": []}).to_string(),
                                    ));
                                }
                                Ok((_, server_rev, items)) => {
                                    let ops: Vec<Value> = items
                                        .into_iter()
                                        .map(|e| {
                                            json!({
                                                "serverRev": e.server_rev,
                                                "operationId": e.operation_id,
                                                "opType": e.op_type,
                                                "payload": e.payload,
                                                "userId": e.user_id,
                                                "createdAt": e.created_at,
                                            })
                                        })
                                        .collect();
                                    addr.do_send(PushText(
                                        json!({"type":"sync","serverRev": server_rev, "ops": ops})
                                            .to_string(),
                                    ));
                                }
                                Err(CollabServiceError::SyncGapTooLarge { .. }) => {
                                    addr.do_send(PushText(
                                        json!({"type":"error","code":"SYNC_GAP_TOO_LARGE","message":"变更过多，请请求全量同步"})
                                            .to_string(),
                                    ));
                                }
                                Err(_) => {
                                    addr.do_send(PushText(
                                        json!({"type":"error","code":"INTERNAL_ERROR","message":"sync 失败"})
                                            .to_string(),
                                    ));
                                }
                            }
                        }
                        "presence" => {}
                        _ => {}
                    }
                });
            }
            Ok(ws::Message::Close(reason)) => {
                ctx.close(reason);
                ctx.stop();
            }
            _ => {}
        }
    }
}

#[derive(Deserialize)]
pub struct WsQuery {
    token: String,
}

fn map_ws_auth_error(err: CollabServiceError) -> HttpResponse {
    match err {
        CollabServiceError::RoomNotFound => HttpResponse::NotFound().json(json!({
            "code": "ROOM_NOT_FOUND",
            "message": "房间不存在"
        })),
        CollabServiceError::NotAMember => HttpResponse::Forbidden().json(json!({
            "code": "NOT_A_MEMBER",
            "message": "你不是该房间成员"
        })),
        _ => HttpResponse::InternalServerError().json(json!({
            "code": "INTERNAL_ERROR",
            "message": "服务器内部错误"
        })),
    }
}

pub async fn collab_ws_handler(
    req: HttpRequest,
    stream: web::Payload,
    db: web::Data<DatabaseConnection>,
    hub: web::Data<CollabHub>,
    path: web::Path<String>,
    query: web::Query<WsQuery>,
) -> Result<HttpResponse, actix_web::Error> {
    let room_id = path.into_inner();
    let claims = match verify_access_token(&query.token) {
        Ok(c) => c,
        Err(_) => {
            return Ok(HttpResponse::Unauthorized().json(json!({
                "code": "UNAUTHORIZED",
                "message": "请先登录"
            })));
        }
    };
    let role = match authorize_ws(&db, &room_id, &claims.sub).await {
        Ok(r) => r,
        Err(e) => return Ok(map_ws_auth_error(e)),
    };

    let session = CollabWsSession {
        room_id,
        user_id: claims.sub,
        role,
        db: db.get_ref().clone(),
        hub: hub.get_ref().clone(),
    };
    ws::start(session, &req, stream)
}
