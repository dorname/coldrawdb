use actix::prelude::*;
use actix_web::{web, Error, HttpRequest, HttpResponse};
use actix_web_actors::ws;
use sea_orm::{ActiveModelTrait, DatabaseConnection, EntityTrait, TransactionTrait};

use crate::entity::prelude::Diagram;
use crate::entity::vo::DiagramVo;

use chrono::Utc;
use std::collections::HashMap;

#[derive(Message)]
#[rtype(result = "()")]
pub struct WsText(pub String);

#[derive(Message)]
#[rtype(result = "usize")]
struct Connect {
    pub diagram_id: String,
    pub addr: Recipient<WsText>,
}

#[derive(Message)]
#[rtype(result = "()")]
struct Disconnect {
    pub diagram_id: String,
    pub id: usize,
}

#[derive(Message)]
#[rtype(result = "()")]
struct ClientText {
    pub diagram_id: String,
    pub id: usize,
    pub text: String,
}

#[derive(Message)]
#[rtype(result = "()")]
struct Broadcast {
    pub diagram_id: String,
    pub text: String,
    pub skip: Option<usize>,
}

#[derive(Message)]
#[rtype(result = "()")]
struct SendOne {
    pub diagram_id: String,
    pub id: usize,
    pub text: String,
}

pub struct RoomHub {
    sessions: HashMap<String, HashMap<usize, Recipient<WsText>>>,
    next_id: usize,
    db: DatabaseConnection,
}

impl RoomHub {
    pub fn new(db: DatabaseConnection) -> Self {
        Self {
            sessions: HashMap::new(),
            next_id: 1,
            db,
        }
    }

    fn broadcast(&self, diagram_id: &str, message: &str, skip: Option<usize>) {
        if let Some(room) = self.sessions.get(diagram_id) {
            for (id, addr) in room {
                if skip.is_some_and(|s| s == *id) {
                    continue;
                }
                let _ = addr.do_send(WsText(message.to_string()));
            }
        }
    }

    fn send_one(&self, diagram_id: &str, id: usize, message: &str) {
        if let Some(room) = self.sessions.get(diagram_id) {
            if let Some(addr) = room.get(&id) {
                let _ = addr.do_send(WsText(message.to_string()));
            }
        }
    }
}

impl Actor for RoomHub {
    type Context = Context<Self>;
}

impl Handler<Connect> for RoomHub {
    type Result = usize;

    fn handle(&mut self, msg: Connect, _: &mut Context<Self>) -> Self::Result {
        let id = self.next_id;
        self.next_id += 1;

        self.sessions
            .entry(msg.diagram_id)
            .or_default()
            .insert(id, msg.addr);
        id
    }
}

impl Handler<Disconnect> for RoomHub {
    type Result = ();

    fn handle(&mut self, msg: Disconnect, _: &mut Context<Self>) {
        if let Some(room) = self.sessions.get_mut(&msg.diagram_id) {
            room.remove(&msg.id);
            if room.is_empty() {
                self.sessions.remove(&msg.diagram_id);
            }
        }
    }
}

impl Handler<ClientText> for RoomHub {
    type Result = ResponseFuture<()>;

    fn handle(&mut self, msg: ClientText, ctx: &mut Context<Self>) -> Self::Result {
        let db = self.db.clone();
        let diagram_id = msg.diagram_id.clone();
        let sender_id = msg.id;
        let text = msg.text.clone();
        let hub_addr = ctx.address();

        Box::pin(async move {
            let parsed: serde_json::Value = match serde_json::from_str(&text) {
                Ok(v) => v,
                Err(_) => return,
            };

            let msg_type = parsed.get("type").and_then(|v| v.as_str()).unwrap_or("");
            if msg_type == "diagram_snapshot_broadcast" {
                // Broadcast-only message: assumes snapshot already persisted via REST.
                let diagram_val = match parsed.get("payload").and_then(|p| p.get("diagram")) {
                    Some(v) => v.clone(),
                    None => return,
                };
                let vo: DiagramVo = match serde_json::from_value(diagram_val) {
                    Ok(v) => v,
                    Err(_) => return,
                };
                if vo.id != diagram_id {
                    return;
                }
                let out = serde_json::json!({
                    "type": "diagram_snapshot",
                    "payload": {
                        "diagramId": diagram_id,
                        "senderSessionId": sender_id,
                        "diagram": vo,
                    }
                });
                hub_addr.do_send(Broadcast {
                    diagram_id,
                    text: out.to_string(),
                    skip: Some(sender_id),
                });
                return;
            }

            if msg_type != "diagram_snapshot" {
                return;
            }

            // Expect payload.diagram to be a DiagramVo-like JSON (same as REST /diagrams/update).
            let diagram_val = match parsed.get("payload").and_then(|p| p.get("diagram")) {
                Some(v) => v.clone(),
                None => return,
            };
            let mut vo: DiagramVo = match serde_json::from_value(diagram_val) {
                Ok(v) => v,
                Err(_) => return,
            };

            if vo.id != diagram_id {
                return;
            }

            let expected_revision = vo.revision.unwrap_or(0);
            let tx = match db.begin().await {
                Ok(t) => t,
                Err(_) => return,
            };

            let existing = match Diagram::find_by_id(vo.id.clone()).one(&tx).await {
                Ok(v) => v,
                Err(_) => {
                    let _ = tx.rollback().await;
                    return;
                }
            };
            let Some(existing) = existing else {
                let _ = tx.rollback().await;
                return;
            };

            if expected_revision != existing.revision {
                let _ = tx.rollback().await;
                let conflict = serde_json::json!({
                    "type": "conflict",
                    "payload": {
                        "diagramId": diagram_id,
                        "expectedRevision": expected_revision,
                        "currentRevision": existing.revision
                    }
                });
                hub_addr.do_send(SendOne {
                    diagram_id,
                    id: sender_id,
                    text: conflict.to_string(),
                });
                return;
            }

            let now = Utc::now().to_rfc3339();
            vo.revision = Some(existing.revision + 1);
            vo.updated_at = Some(now.clone());
            vo.last_modified = Some(now);

            let am = vo.convert_to_active_model();
            let updated = match am.update(&tx).await {
                Ok(u) => u,
                Err(_) => {
                    let _ = tx.rollback().await;
                    return;
                }
            };
            if tx.commit().await.is_err() {
                return;
            }

            // Broadcast updated snapshot to everyone else in room.
            let out = serde_json::json!({
                "type": "diagram_snapshot",
                "payload": {
                    "diagramId": diagram_id,
                    "senderSessionId": sender_id,
                    "diagram": DiagramVo::from(&updated),
                }
            });
            hub_addr.do_send(Broadcast {
                diagram_id,
                text: out.to_string(),
                skip: Some(sender_id),
            });
        })
    }
}

impl Handler<Broadcast> for RoomHub {
    type Result = ();

    fn handle(&mut self, msg: Broadcast, _: &mut Context<Self>) {
        self.broadcast(&msg.diagram_id, &msg.text, msg.skip);
    }
}

impl Handler<SendOne> for RoomHub {
    type Result = ();

    fn handle(&mut self, msg: SendOne, _: &mut Context<Self>) {
        self.send_one(&msg.diagram_id, msg.id, &msg.text);
    }
}

pub struct WsSession {
    diagram_id: String,
    id: usize,
    hub: Addr<RoomHub>,
}

impl WsSession {
    pub fn new(diagram_id: String, hub: Addr<RoomHub>) -> Self {
        Self {
            diagram_id,
            id: 0,
            hub,
        }
    }
}

impl Actor for WsSession {
    type Context = ws::WebsocketContext<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        let addr = ctx.address().recipient();
        let diagram_id = self.diagram_id.clone();
        self.hub
            .send(Connect { diagram_id, addr })
            .into_actor(self)
            .map(|res, act, _| {
                if let Ok(id) = res {
                    act.id = id;
                }
            })
            .wait(ctx);
    }

    fn stopped(&mut self, _: &mut Self::Context) {
        self.hub.do_send(Disconnect {
            diagram_id: self.diagram_id.clone(),
            id: self.id,
        });
    }
}

impl Handler<WsText> for WsSession {
    type Result = ();

    fn handle(&mut self, msg: WsText, ctx: &mut Self::Context) {
        ctx.text(msg.0);
    }
}

impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for WsSession {
    fn handle(&mut self, item: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        match item {
            Ok(ws::Message::Text(text)) => {
                self.hub.do_send(ClientText {
                    diagram_id: self.diagram_id.clone(),
                    id: self.id,
                    text: text.to_string(),
                });
            }
            Ok(ws::Message::Ping(msg)) => ctx.pong(&msg),
            Ok(ws::Message::Pong(_)) => {}
            Ok(ws::Message::Close(reason)) => {
                ctx.close(reason);
                ctx.stop();
            }
            _ => {}
        }
    }
}

/// GET /diagrams/ws/{diagram_id}
pub async fn ws_diagram(
    req: HttpRequest,
    stream: web::Payload,
    diagram_id: web::Path<String>,
    hub: web::Data<Addr<RoomHub>>,
) -> Result<HttpResponse, Error> {
    let session = WsSession::new(diagram_id.into_inner(), hub.get_ref().clone());
    ws::start(session, &req, stream)
}

/// 在 App 启动时创建并注入 Hub（需要 DatabaseConnection）。
pub fn create_hub(db: DatabaseConnection) -> Addr<RoomHub> {
    RoomHub::new(db).start()
}

