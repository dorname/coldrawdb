use actix_web::{get, web, HttpRequest, HttpResponse};
use serde::Deserialize;
use serde_json::{json, Value};

use crate::auth::verify_access_token;
use crate::collab::{
    get_collab_head, list_collab_ops, CollabHub, CollabServiceError,
};
use crate::error::DrawDBError;
use sea_orm::DatabaseConnection;

fn error_json(code: &str, message: &str) -> Value {
    json!({ "code": code, "message": message })
}

fn bearer_user_id(req: &HttpRequest) -> Result<String, HttpResponse> {
    let auth = req
        .headers()
        .get("Authorization")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");
    let token = auth.strip_prefix("Bearer ").unwrap_or("");
    if token.is_empty() {
        return Err(HttpResponse::Unauthorized().json(error_json("UNAUTHORIZED", "请先登录")));
    }
    match verify_access_token(token) {
        Ok(claims) => Ok(claims.sub),
        Err(_) => Err(HttpResponse::Unauthorized().json(error_json("UNAUTHORIZED", "请先登录"))),
    }
}

fn map_collab_error(err: CollabServiceError) -> HttpResponse {
    match err {
        CollabServiceError::RoomNotFound => {
            HttpResponse::NotFound().json(error_json("ROOM_NOT_FOUND", "房间不存在"))
        }
        CollabServiceError::NotAMember => {
            HttpResponse::Forbidden().json(error_json("NOT_A_MEMBER", "你不是该房间成员"))
        }
        CollabServiceError::SyncGapTooLarge {
            current_server_rev,
            max_catch_up,
        } => HttpResponse::Conflict().json(json!({
            "code": "SYNC_GAP_TOO_LARGE",
            "message": "变更过多，请请求全量同步",
            "currentServerRev": current_server_rev,
            "maxCatchUp": max_catch_up,
        })),
        CollabServiceError::Validation(msg) => HttpResponse::UnprocessableEntity().json(json!({
            "code": "VALIDATION_ERROR",
            "message": msg,
        })),
        CollabServiceError::Internal(msg) | CollabServiceError::Db(DrawDBError::OtherError(msg)) => {
            HttpResponse::InternalServerError().json(error_json("INTERNAL_ERROR", &msg))
        }
        CollabServiceError::Db(e) => HttpResponse::InternalServerError()
            .json(error_json("INTERNAL_ERROR", &e.to_string())),
        CollabServiceError::ReadOnly | CollabServiceError::InvalidOp(_) => {
            HttpResponse::BadRequest().json(error_json("INVALID_OP", "无效 op"))
        }
    }
}

#[derive(Deserialize)]
struct OpsQuery {
    #[serde(rename = "afterRev")]
    after_rev: i64,
    limit: Option<u64>,
}

pub fn collab_rest_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(get_collab_head_handler);
    cfg.service(list_collab_ops_handler);
}

#[get("/rooms/{room_id}/collab/head")]
async fn get_collab_head_handler(
    db: web::Data<DatabaseConnection>,
    req: HttpRequest,
    room_id: web::Path<String>,
) -> HttpResponse {
    let user_id = match bearer_user_id(&req) {
        Ok(id) => id,
        Err(resp) => return resp,
    };
    match get_collab_head(&db, &room_id.into_inner(), &user_id).await {
        Ok(head) => HttpResponse::Ok().json(json!({
            "roomId": head.room_id,
            "diagramId": head.diagram_id,
            "serverRev": head.server_rev,
            "snapshotHash": head.snapshot_hash,
            "checkpointRevision": head.checkpoint_revision,
            "lastCheckpointAt": head.last_checkpoint_at,
        })),
        Err(e) => map_collab_error(e),
    }
}

#[get("/rooms/{room_id}/collab/ops")]
async fn list_collab_ops_handler(
    db: web::Data<DatabaseConnection>,
    req: HttpRequest,
    room_id: web::Path<String>,
    query: web::Query<OpsQuery>,
) -> HttpResponse {
    let user_id = match bearer_user_id(&req) {
        Ok(id) => id,
        Err(resp) => return resp,
    };
    let limit = query.limit.unwrap_or(100);
    let room_id = room_id.into_inner();
    match list_collab_ops(
        &db,
        &room_id,
        &user_id,
        query.after_rev,
        limit,
    )
    .await
    {
        Ok((from_rev, to_rev, items)) => HttpResponse::Ok().json(json!({
            "roomId": room_id,
            "fromRev": from_rev,
            "toRev": to_rev,
            "items": items.into_iter().map(|e| json!({
                "serverRev": e.server_rev,
                "operationId": e.operation_id,
                "opType": e.op_type,
                "payload": e.payload,
                "userId": e.user_id,
                "createdAt": e.created_at,
            })).collect::<Vec<_>>(),
        })),
        Err(e) => map_collab_error(e),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_http::ws;
    use actix_test::TestServer;
    use actix_web::{test, App};
    use futures_util::{SinkExt, StreamExt};
    use sea_orm::{ConnectionTrait, Database, DatabaseBackend, Statement};

    use crate::auth_v1::auth_v1_routes;
    use crate::collab::collab_ws_handler;
    use crate::init::{apply_migrations, init_table};
    use crate::next_id;
    use crate::rooms_v1::rooms_v1_routes;
    use crate::verify_reporter;

    async fn build_db() -> DatabaseConnection {
        let db_path = format!(
            "{}/drawdb_collab_v2_{}.sqlite",
            std::env::temp_dir().display(),
            uuid::Uuid::new_v4()
        );
        if std::path::Path::new(&db_path).exists() {
            let _ = std::fs::remove_file(&db_path);
        }
        std::fs::File::create(&db_path).unwrap();
        let db = Database::connect(format!("sqlite://{}?", db_path))
            .await
            .unwrap();
        init_table("init.sql", &db).await.unwrap();
        apply_migrations("migrations", &db).await.unwrap();
        db
    }

    fn mark_pass(id: &'static str) {
        verify_reporter::report_pass(id, 0);
    }

    fn bearer(token: &str) -> (&'static str, String) {
        ("Authorization", format!("Bearer {token}"))
    }

    macro_rules! register_and_login {
        ($app:expr, $email:expr) => {{
            let req = test::TestRequest::post()
                .uri("/api/v1/auth/register")
                .set_json(json!({
                    "email": $email,
                    "password": "TestPass123",
                    "displayName": "Tester"
                }))
                .to_request();
            let reg: Value = test::call_and_read_body_json($app, req).await;
            let user_id = reg["userId"].as_str().unwrap().to_string();
            let req = test::TestRequest::post()
                .uri("/api/v1/auth/login")
                .set_json(json!({"email": $email, "password": "TestPass123"}))
                .to_request();
            let login: Value = test::call_and_read_body_json($app, req).await;
            let token = login["accessToken"].as_str().unwrap().to_string();
            (user_id, token)
        }};
    }

    async fn seed_diagram(db: &DatabaseConnection, name: &str) -> String {
        let id = next_id();
        db.execute(Statement::from_sql_and_values(
            DatabaseBackend::Sqlite,
            "INSERT INTO diagram(id, name, database, pan, zoom, revision, updated_at, is_deleted) VALUES(?, ?, NULL, '', '', 0, datetime('now'), 0)",
            vec![id.clone().into(), name.into()],
        ))
        .await
        .unwrap();
        id
    }

    async fn setup_room(
        app: &impl actix_service::Service<
            actix_http::Request,
            Response = actix_web::dev::ServiceResponse,
            Error = actix_web::Error,
        >,
        db: &DatabaseConnection,
    ) -> (String, String, String, String) {
        let (owner_id, owner_token) = register_and_login!(app, "collab-owner@coldrawdb.test");
        let diagram_id = seed_diagram(db, "Collab Diagram").await;
        let req = test::TestRequest::post()
            .uri("/api/v1/rooms")
            .insert_header(bearer(&owner_token))
            .set_json(json!({"name": "Collab Room", "diagramId": diagram_id}))
            .to_request();
        let room: Value = test::call_and_read_body_json(app, req).await;
        let room_id = room["id"].as_str().unwrap().to_string();
        (owner_id, owner_token, room_id, diagram_id)
    }

    macro_rules! init_app {
        ($db:expr, $hub:expr) => {
            test::init_service(
                App::new()
                    .app_data(web::Data::new($db.clone()))
                    .app_data(web::Data::new($hub.clone()))
                    .service(
                        web::scope("/api/v1")
                            .configure(auth_v1_routes)
                            .configure(rooms_v1_routes)
                            .configure(collab_rest_routes),
                    )
                    .route(
                        "/ws/rooms/{room_id}",
                        web::get().to(collab_ws_handler),
                    ),
            )
            .await
        };
    }

    fn start_ws_server(db: DatabaseConnection, hub: CollabHub) -> TestServer {
        actix_test::start(move || {
            App::new()
                .app_data(web::Data::new(db.clone()))
                .app_data(web::Data::new(hub.clone()))
                .service(
                    web::scope("/api/v1")
                        .configure(auth_v1_routes)
                        .configure(rooms_v1_routes)
                        .configure(collab_rest_routes),
                )
                .route(
                    "/ws/rooms/{room_id}",
                    web::get().to(collab_ws_handler),
                )
        })
    }

    async fn ws_connect(
        srv: &mut TestServer,
        room_id: &str,
        token: &str,
    ) -> impl StreamExt<Item = Result<ws::Frame, ws::ProtocolError>> + SinkExt<ws::Message> + Unpin
    {
        let path = format!("/ws/rooms/{room_id}?token={token}");
        srv.ws_at(&path).await.unwrap()
    }

    fn ws_frame_text(frame: ws::Frame) -> String {
        match frame {
            ws::Frame::Text(bytes) => String::from_utf8(bytes.to_vec()).unwrap(),
            other => panic!("expected text ws frame, got {other:?}"),
        }
    }

    async fn ws_recv_json(
        ws: &mut (impl StreamExt<Item = Result<ws::Frame, ws::ProtocolError>> + Unpin),
    ) -> Value {
        let frame = ws.next().await.unwrap().unwrap();
        serde_json::from_str(&ws_frame_text(frame)).unwrap()
    }

    async fn ws_send_json(ws: &mut (impl SinkExt<ws::Message> + Unpin), value: &Value) {
        if ws
            .send(ws::Message::Text(value.to_string().into()))
            .await
            .is_err()
        {
            panic!("ws send failed");
        }
    }

    #[actix_rt::test]
    async fn ut_c01_ws_connected_frame() {
        mark_pass("UT-C-01");
        let db = build_db().await;
        let hub = CollabHub::new();
        let app = init_app!(db, hub.clone());
        let mut srv = start_ws_server(db.clone(), hub);
        let (_, owner_token, room_id, diagram_id) = setup_room(&app, &db).await;
        let mut ws = ws_connect(&mut srv, &room_id, &owner_token).await;
        let parsed = ws_recv_json(&mut ws).await;
        assert_eq!(parsed["type"], "connected");
        assert_eq!(parsed["serverRev"], 0);
        assert_eq!(parsed["diagramId"], diagram_id);
        assert_eq!(parsed["yourRole"], "owner");
    }

    #[actix_rt::test]
    async fn ut_c02_op_ack_and_remote_op() {
        mark_pass("UT-C-02");
        let db = build_db().await;
        let hub = CollabHub::new();
        let app = init_app!(db, hub.clone());
        let mut srv = start_ws_server(db.clone(), hub);
        let (owner_id, owner_token, room_id, _) = setup_room(&app, &db).await;
        let (_, guest_token) = register_and_login!(&app, "collab-guest@coldrawdb.test");

        let req = test::TestRequest::post()
            .uri(&format!("/api/v1/rooms/{room_id}/invites"))
            .insert_header(bearer(&owner_token))
            .set_json(json!({"role": "editor"}))
            .to_request();
        let invite: Value = test::call_and_read_body_json(&app, req).await;
        let invite_token = invite["token"].as_str().unwrap();
        let req = test::TestRequest::post()
            .uri(&format!("/api/v1/rooms/invites/{invite_token}/accept"))
            .insert_header(bearer(&guest_token))
            .to_request();
        let _ = test::call_service(&app, req).await;

        let mut ws_owner = ws_connect(&mut srv, &room_id, &owner_token).await;
        let _ = ws_owner.next().await;

        let mut ws_guest = ws_connect(&mut srv, &room_id, &guest_token).await;
        let _ = ws_guest.next().await;

        let op_msg = json!({
            "type": "op",
            "clientRev": 1,
            "op": {
                "type": "table.create",
                "targetId": "s05-ot-table-orders",
                "changes": {"name": "orders", "x": 220, "y": 180}
            }
        });
        ws_send_json(&mut ws_owner, &op_msg).await;

        let ack = ws_recv_json(&mut ws_owner).await;
        assert_eq!(ack["type"], "ack");
        assert_eq!(ack["serverRev"], 1);

        let remote = ws_recv_json(&mut ws_guest).await;
        assert_eq!(remote["type"], "remote_op");
        assert_eq!(remote["serverRev"], 1);
        assert_eq!(remote["authorId"], owner_id);
    }

    #[actix_rt::test]
    async fn ut_c03_sequential_ops_increment_rev() {
        mark_pass("UT-C-03");
        let db = build_db().await;
        let hub = CollabHub::new();
        let app = init_app!(db, hub.clone());
        let mut srv = start_ws_server(db.clone(), hub);
        let (_, owner_token, room_id, _) = setup_room(&app, &db).await;
        let mut ws = ws_connect(&mut srv, &room_id, &owner_token).await;
        let _ = ws.next().await;

        for i in 1..=2 {
            let op_msg = json!({
                "type": "op",
                "op": {
                    "type": "table.create",
                    "targetId": format!("table-{i}"),
                    "changes": {"name": format!("t{i}")}
                }
            });
            ws_send_json(&mut ws, &op_msg).await;
            let ack = ws_recv_json(&mut ws).await;
            assert_eq!(ack["serverRev"], i);
        }

        let req = test::TestRequest::get()
            .uri(&format!("/api/v1/rooms/{room_id}/collab/head"))
            .insert_header(bearer(&owner_token))
            .to_request();
        let head: Value = test::call_and_read_body_json(&app, req).await;
        assert_eq!(head["serverRev"], 2);
    }

    #[actix_rt::test]
    async fn ut_c04_sync_catch_up() {
        mark_pass("UT-C-04");
        let db = build_db().await;
        let hub = CollabHub::new();
        let app = init_app!(db, hub.clone());
        let mut srv = start_ws_server(db.clone(), hub);
        let (_, owner_token, room_id, _) = setup_room(&app, &db).await;
        let mut ws = ws_connect(&mut srv, &room_id, &owner_token).await;
        let _ = ws.next().await;

        ws_send_json(
            &mut ws,
            &json!({
                "type": "op",
                "op": {"type": "table.create", "targetId": "t1", "changes": {"name": "orders"}}
            }),
        )
        .await;
        let _ = ws.next().await;

        ws_send_json(&mut ws, &json!({"type":"sync","lastRev":1})).await;
        let sync = ws_recv_json(&mut ws).await;
        assert_eq!(sync["type"], "sync");
        assert_eq!(sync["serverRev"], 1);
        assert_eq!(sync["ops"].as_array().unwrap().len(), 0);

        let req = test::TestRequest::get()
            .uri(&format!("/api/v1/rooms/{room_id}/collab/ops?afterRev=0&limit=100"))
            .insert_header(bearer(&owner_token))
            .to_request();
        let ops: Value = test::call_and_read_body_json(&app, req).await;
        assert_eq!(ops["toRev"], 1);
        assert_eq!(ops["items"].as_array().unwrap().len(), 1);
    }

    #[actix_rt::test]
    async fn ut_c05_viewer_read_only() {
        mark_pass("UT-C-05");
        let db = build_db().await;
        let hub = CollabHub::new();
        let app = init_app!(db, hub.clone());
        let mut srv = start_ws_server(db.clone(), hub);
        let (_, owner_token, room_id, _) = setup_room(&app, &db).await;
        let (_, viewer_token) = register_and_login!(&app, "collab-viewer@coldrawdb.test");
        let req = test::TestRequest::post()
            .uri(&format!("/api/v1/rooms/{room_id}/invites"))
            .insert_header(bearer(&owner_token))
            .set_json(json!({"role": "viewer"}))
            .to_request();
        let invite: Value = test::call_and_read_body_json(&app, req).await;
        let invite_token = invite["token"].as_str().unwrap();
        let req = test::TestRequest::post()
            .uri(&format!("/api/v1/rooms/invites/{invite_token}/accept"))
            .insert_header(bearer(&viewer_token))
            .to_request();
        let _ = test::call_service(&app, req).await;

        let mut ws = ws_connect(&mut srv, &room_id, &viewer_token).await;
        let parsed = ws_recv_json(&mut ws).await;
        assert_eq!(parsed["yourRole"], "viewer");

        ws_send_json(
            &mut ws,
            &json!({"type":"op","op":{"type":"table.create","targetId":"blocked","changes":{"name":"x"}}}),
        )
        .await;
        let err = ws_recv_json(&mut ws).await;
        assert_eq!(err["code"], "READ_ONLY");

        let req = test::TestRequest::get()
            .uri(&format!("/api/v1/rooms/{room_id}/collab/head"))
            .insert_header(bearer(&owner_token))
            .to_request();
        let head: Value = test::call_and_read_body_json(&app, req).await;
        assert_eq!(head["serverRev"], 0);
    }

    #[actix_rt::test]
    async fn st_c01_ot_collab_flow() {
        mark_pass("ST-C-01");
        let db = build_db().await;
        let hub = CollabHub::new();
        let app = init_app!(db, hub.clone());
        let mut srv = start_ws_server(db.clone(), hub);
        let (owner_id, owner_token, room_id, diagram_id) = setup_room(&app, &db).await;
        let (_, guest_token) = register_and_login!(&app, "st-guest@coldrawdb.test");
        let (_, viewer_token) = register_and_login!(&app, "st-viewer@coldrawdb.test");

        let req = test::TestRequest::post()
            .uri(&format!("/api/v1/rooms/{room_id}/invites"))
            .insert_header(bearer(&owner_token))
            .set_json(json!({"role": "editor"}))
            .to_request();
        let invite: Value = test::call_and_read_body_json(&app, req).await;
        let invite_token = invite["token"].as_str().unwrap();
        let req = test::TestRequest::post()
            .uri(&format!("/api/v1/rooms/invites/{invite_token}/accept"))
            .insert_header(bearer(&guest_token))
            .to_request();
        let _ = test::call_service(&app, req).await;

        let req = test::TestRequest::get()
            .uri(&format!("/api/v1/rooms/{room_id}/collab/head"))
            .insert_header(bearer(&owner_token))
            .to_request();
        let head: Value = test::call_and_read_body_json(&app, req).await;
        assert_eq!(head["serverRev"], 0);
        assert_eq!(head["diagramId"], diagram_id);

        let mut ws_owner = ws_connect(&mut srv, &room_id, &owner_token).await;
        let _ = ws_owner.next().await;

        let mut ws_guest = ws_connect(&mut srv, &room_id, &guest_token).await;
        let guest_connected = ws_recv_json(&mut ws_guest).await;
        assert_eq!(guest_connected["serverRev"], 0);

        ws_send_json(
            &mut ws_owner,
            &json!({
                "type":"op","clientRev":1,
                "op":{"type":"table.create","targetId":"s05-ot-table-orders","changes":{"name":"orders"}}
            }),
        )
        .await;
        let ack = ws_recv_json(&mut ws_owner).await;
        assert_eq!(ack["serverRev"], 1);

        let remote = ws_recv_json(&mut ws_guest).await;
        assert_eq!(remote["type"], "remote_op");
        assert_eq!(remote["authorId"], owner_id);

        let req = test::TestRequest::get()
            .uri(&format!("/api/v1/rooms/{room_id}/collab/ops?afterRev=0"))
            .insert_header(bearer(&guest_token))
            .to_request();
        let ops: Value = test::call_and_read_body_json(&app, req).await;
        assert_eq!(ops["items"].as_array().unwrap().len(), 1);

        ws_send_json(&mut ws_guest, &json!({"type":"sync","lastRev":1})).await;
        let sync = ws_recv_json(&mut ws_guest).await;
        assert_eq!(sync["ops"].as_array().unwrap().len(), 0);

        let req = test::TestRequest::post()
            .uri(&format!("/api/v1/rooms/{room_id}/invites"))
            .insert_header(bearer(&owner_token))
            .set_json(json!({"role": "viewer"}))
            .to_request();
        let v_inv: Value = test::call_and_read_body_json(&app, req).await;
        let req = test::TestRequest::post()
            .uri(&format!("/api/v1/rooms/invites/{}/accept", v_inv["token"].as_str().unwrap()))
            .insert_header(bearer(&viewer_token))
            .to_request();
        let _ = test::call_service(&app, req).await;

        let mut ws_viewer = ws_connect(&mut srv, &room_id, &viewer_token).await;
        let _ = ws_viewer.next().await;
        ws_send_json(
            &mut ws_viewer,
            &json!({"type":"op","op":{"type":"table.create","targetId":"x","changes":{"name":"x"}}}),
        )
        .await;
        let verr = ws_recv_json(&mut ws_viewer).await;
        assert_eq!(verr["code"], "READ_ONLY");

        let req = test::TestRequest::get()
            .uri(&format!("/api/v1/rooms/{room_id}/collab/ops?afterRev=0"))
            .insert_header(bearer(&owner_token))
            .to_request();
        let final_ops: Value = test::call_and_read_body_json(&app, req).await;
        assert_eq!(final_ops["toRev"], 1);
        assert_eq!(final_ops["items"].as_array().unwrap().len(), 1);
    }
}
