use actix_web::{delete, get, patch, post, web, HttpRequest, HttpResponse};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::auth::verify_access_token;
use crate::error::DrawDBError;
use crate::rooms::{
    accept_invite, archive_room, create_invite, create_room, get_room_detail, leave_room,
    list_archived_rooms, list_members, list_rooms, permanent_delete_room, preview_invite,
    public_base_url, remove_member, restore_room, update_member_role, RoomsServiceError,
};
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

fn map_rooms_error(err: RoomsServiceError) -> HttpResponse {
    match err {
        RoomsServiceError::DiagramNotFound => {
            HttpResponse::NotFound().json(error_json("DIAGRAM_NOT_FOUND", "图表不存在"))
        }
        RoomsServiceError::DiagramTaken { existing_room_id } => HttpResponse::Conflict().json(json!({
            "code": "ROOM_DIAGRAM_TAKEN",
            "message": "该 diagram 已在其他协作房间中",
            "existingRoomId": existing_room_id
        })),
        RoomsServiceError::RoomNotFound => {
            HttpResponse::NotFound().json(error_json("ROOM_NOT_FOUND", "房间不存在"))
        }
        RoomsServiceError::NotAMember => {
            HttpResponse::Forbidden().json(error_json("NOT_A_MEMBER", "你不是该房间成员"))
        }
        RoomsServiceError::Forbidden(msg) => {
            HttpResponse::Forbidden().json(error_json("FORBIDDEN", &msg))
        }
        RoomsServiceError::InviteNotFound => {
            HttpResponse::NotFound().json(error_json("INVITE_NOT_FOUND", "邀请链接无效"))
        }
        RoomsServiceError::InviteExpired => {
            HttpResponse::Gone().json(error_json("INVITE_EXPIRED", "邀请链接已过期"))
        }
        RoomsServiceError::OwnerCannotLeave => HttpResponse::Conflict()
            .json(error_json("OWNER_CANNOT_LEAVE", "房间 owner 不能离开，请先删除房间或转让")),
        RoomsServiceError::CannotRemoveOwner => HttpResponse::Conflict()
            .json(error_json("CANNOT_REMOVE_OWNER", "不能移除房间 owner")),
        RoomsServiceError::MemberNotFound => {
            HttpResponse::NotFound().json(error_json("MEMBER_NOT_FOUND", "成员不存在"))
        }
        RoomsServiceError::Validation { fields } => {
            let map: serde_json::Map<String, Value> = fields
                .into_iter()
                .map(|(k, v)| (k, Value::String(v)))
                .collect();
            HttpResponse::UnprocessableEntity().json(json!({
                "code": "VALIDATION_ERROR",
                "message": "请求参数无效",
                "fields": map
            }))
        }
        RoomsServiceError::Internal(msg) | RoomsServiceError::Db(DrawDBError::OtherError(msg)) => {
            HttpResponse::InternalServerError().json(error_json("INTERNAL_ERROR", &msg))
        }
        RoomsServiceError::Db(e) => HttpResponse::InternalServerError()
            .json(error_json("INTERNAL_ERROR", &e.to_string())),
    }
}

#[derive(Deserialize)]
struct CreateRoomBody {
    name: String,
    #[serde(rename = "diagramId")]
    diagram_id: String,
}

#[derive(Serialize)]
struct RoomResponse {
    id: String,
    name: String,
    #[serde(rename = "diagramId")]
    diagram_id: String,
    #[serde(rename = "ownerId")]
    owner_id: String,
    #[serde(rename = "createdAt")]
    created_at: String,
    #[serde(rename = "updatedAt")]
    updated_at: String,
}

#[derive(Serialize)]
struct RoomDetailResponse {
    id: String,
    name: String,
    #[serde(rename = "diagramId")]
    diagram_id: String,
    #[serde(rename = "ownerId")]
    owner_id: String,
    #[serde(rename = "createdAt")]
    created_at: String,
    #[serde(rename = "updatedAt")]
    updated_at: String,
    #[serde(rename = "diagramTitle")]
    diagram_title: String,
    #[serde(rename = "myRole")]
    my_role: String,
    #[serde(rename = "memberCount")]
    member_count: i64,
}

#[derive(Serialize)]
struct RoomSummaryResponse {
    id: String,
    name: String,
    #[serde(rename = "diagramId")]
    diagram_id: String,
    #[serde(rename = "diagramTitle")]
    diagram_title: String,
    #[serde(rename = "myRole")]
    my_role: String,
    #[serde(rename = "memberCount")]
    member_count: i64,
    #[serde(rename = "updatedAt")]
    updated_at: String,
    // feat-room-recycle-bin-dropdown-and-db-export: recycle bin shows archive time
    #[serde(rename = "archivedAt")]
    archived_at: Option<String>,
}

#[derive(Deserialize)]
struct CreateInviteBody {
    role: String,
}

#[derive(Serialize)]
struct InviteCreatedResponse {
    #[serde(rename = "inviteUrl")]
    invite_url: String,
    token: String,
    role: String,
    #[serde(rename = "expiresAt")]
    expires_at: String,
}

#[derive(Serialize)]
struct InvitePreviewResponse {
    #[serde(rename = "roomName")]
    room_name: String,
    #[serde(rename = "diagramTitle")]
    diagram_title: String,
    #[serde(rename = "diagramId")]
    diagram_id: String,
    role: String,
    #[serde(rename = "invitedBy", skip_serializing_if = "Option::is_none")]
    invited_by: Option<String>,
    #[serde(rename = "expiresAt")]
    expires_at: String,
}

#[derive(Serialize)]
struct AcceptInviteResponse {
    #[serde(rename = "roomId")]
    room_id: String,
    #[serde(rename = "diagramId")]
    diagram_id: String,
    role: String,
    #[serde(rename = "alreadyMember", skip_serializing_if = "is_false")]
    already_member: bool,
}

fn is_false(v: &bool) -> bool {
    !*v
}

#[derive(Serialize)]
struct RoomMemberResponse {
    #[serde(rename = "userId")]
    user_id: String,
    email: String,
    #[serde(rename = "displayName", skip_serializing_if = "Option::is_none")]
    display_name: Option<String>,
    role: String,
    #[serde(rename = "joinedAt")]
    joined_at: String,
}

#[derive(Deserialize)]
struct UpdateMemberRoleBody {
    role: String,
}

#[derive(Deserialize)]
struct ListQuery {
    limit: Option<u64>,
    offset: Option<u64>,
    // feat-room-recycle-bin-dropdown-and-db-export: true => recycle bin (owner's archived rooms)
    archived: Option<bool>,
}

pub fn rooms_v1_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(list_rooms_handler);
    cfg.service(create_room_handler);
    cfg.service(preview_invite_handler);
    cfg.service(accept_invite_handler);
    cfg.service(get_room_handler);
    cfg.service(delete_room_handler);
    cfg.service(restore_room_handler);
    cfg.service(permanent_delete_room_handler);
    cfg.service(create_room_invite_handler);
    cfg.service(list_room_members_handler);
    cfg.service(leave_room_handler);
    cfg.service(update_room_member_handler);
    cfg.service(remove_room_member_handler);
}

#[get("/rooms")]
async fn list_rooms_handler(
    db: web::Data<DatabaseConnection>,
    req: HttpRequest,
    query: web::Query<ListQuery>,
) -> HttpResponse {
    let user_id = match bearer_user_id(&req) {
        Ok(id) => id,
        Err(resp) => return resp,
    };
    let limit = query.limit.unwrap_or(20);
    let offset = query.offset.unwrap_or(0);
    let result = if query.archived.unwrap_or(false) {
        list_archived_rooms(&db, &user_id, limit, offset).await
    } else {
        list_rooms(&db, &user_id, limit, offset).await
    };
    match result {
        Ok((items, total)) => HttpResponse::Ok().json(json!({
            "items": items.into_iter().map(|r| RoomSummaryResponse {
                id: r.id,
                name: r.name,
                diagram_id: r.diagram_id,
                diagram_title: r.diagram_title,
                my_role: r.my_role,
                member_count: r.member_count,
                updated_at: r.updated_at,
                archived_at: r.archived_at,
            }).collect::<Vec<_>>(),
            "total": total
        })),
        Err(e) => map_rooms_error(e),
    }
}

#[post("/rooms")]
async fn create_room_handler(
    db: web::Data<DatabaseConnection>,
    req: HttpRequest,
    body: web::Json<CreateRoomBody>,
) -> HttpResponse {
    let user_id = match bearer_user_id(&req) {
        Ok(id) => id,
        Err(resp) => return resp,
    };
    match create_room(&db, &user_id, &body.name, &body.diagram_id).await {
        Ok(room) => HttpResponse::Created().json(RoomResponse {
            id: room.id,
            name: room.name,
            diagram_id: room.diagram_id,
            owner_id: room.owner_id,
            created_at: room.created_at,
            updated_at: room.updated_at,
        }),
        Err(e) => map_rooms_error(e),
    }
}

#[get("/rooms/invites/{token}")]
async fn preview_invite_handler(
    db: web::Data<DatabaseConnection>,
    token: web::Path<String>,
) -> HttpResponse {
    match preview_invite(&db, &token.into_inner()).await {
        Ok(preview) => HttpResponse::Ok().json(InvitePreviewResponse {
            room_name: preview.room_name,
            diagram_title: preview.diagram_title,
            diagram_id: preview.diagram_id,
            role: preview.role,
            invited_by: preview.invited_by,
            expires_at: preview.expires_at,
        }),
        Err(e) => map_rooms_error(e),
    }
}

#[post("/rooms/invites/{token}/accept")]
async fn accept_invite_handler(
    db: web::Data<DatabaseConnection>,
    req: HttpRequest,
    token: web::Path<String>,
) -> HttpResponse {
    let user_id = match bearer_user_id(&req) {
        Ok(id) => id,
        Err(resp) => return resp,
    };
    match accept_invite(&db, &token.into_inner(), &user_id).await {
        Ok(result) => HttpResponse::Ok().json(AcceptInviteResponse {
            room_id: result.room_id,
            diagram_id: result.diagram_id,
            role: result.role,
            already_member: result.already_member,
        }),
        Err(e) => map_rooms_error(e),
    }
}

#[get("/rooms/{room_id}")]
async fn get_room_handler(
    db: web::Data<DatabaseConnection>,
    req: HttpRequest,
    room_id: web::Path<String>,
) -> HttpResponse {
    let user_id = match bearer_user_id(&req) {
        Ok(id) => id,
        Err(resp) => return resp,
    };
    match get_room_detail(&db, &room_id.into_inner(), &user_id).await {
        Ok(detail) => HttpResponse::Ok().json(RoomDetailResponse {
            id: detail.room.id,
            name: detail.room.name,
            diagram_id: detail.room.diagram_id,
            owner_id: detail.room.owner_id,
            created_at: detail.room.created_at,
            updated_at: detail.room.updated_at,
            diagram_title: detail.diagram_title,
            my_role: detail.my_role,
            member_count: detail.member_count,
        }),
        Err(e) => map_rooms_error(e),
    }
}

#[delete("/rooms/{room_id}")]
async fn delete_room_handler(
    db: web::Data<DatabaseConnection>,
    req: HttpRequest,
    room_id: web::Path<String>,
) -> HttpResponse {
    let user_id = match bearer_user_id(&req) {
        Ok(id) => id,
        Err(resp) => return resp,
    };
    match archive_room(&db, &room_id.into_inner(), &user_id).await {
        Ok(()) => HttpResponse::NoContent().finish(),
        Err(e) => map_rooms_error(e),
    }
}

// feat-room-recycle-bin-dropdown-and-db-export: recycle bin restore.
#[post("/rooms/{room_id}/restore")]
async fn restore_room_handler(
    db: web::Data<DatabaseConnection>,
    req: HttpRequest,
    room_id: web::Path<String>,
) -> HttpResponse {
    let user_id = match bearer_user_id(&req) {
        Ok(id) => id,
        Err(resp) => return resp,
    };
    match restore_room(&db, &room_id.into_inner(), &user_id).await {
        Ok(room) => HttpResponse::Ok().json(RoomResponse {
            id: room.id,
            name: room.name,
            diagram_id: room.diagram_id,
            owner_id: room.owner_id,
            created_at: room.created_at,
            updated_at: room.updated_at,
        }),
        Err(RoomsServiceError::DiagramTaken { existing_room_id }) => {
            HttpResponse::Conflict().json(json!({
                "code": "ROOM_DIAGRAM_TAKEN",
                "message": "该 diagram 已在其他协作房间中，无法恢复",
                "existingRoomId": existing_room_id
            }))
        }
        Err(e) => map_rooms_error(e),
    }
}

// feat-room-recycle-bin-dropdown-and-db-export: recycle bin hard delete.
#[delete("/rooms/{room_id}/permanent")]
async fn permanent_delete_room_handler(
    db: web::Data<DatabaseConnection>,
    req: HttpRequest,
    room_id: web::Path<String>,
) -> HttpResponse {
    let user_id = match bearer_user_id(&req) {
        Ok(id) => id,
        Err(resp) => return resp,
    };
    match permanent_delete_room(&db, &room_id.into_inner(), &user_id).await {
        Ok(()) => HttpResponse::NoContent().finish(),
        Err(e) => map_rooms_error(e),
    }
}

#[post("/rooms/{room_id}/invites")]
async fn create_room_invite_handler(
    db: web::Data<DatabaseConnection>,
    req: HttpRequest,
    room_id: web::Path<String>,
    body: web::Json<CreateInviteBody>,
) -> HttpResponse {
    let user_id = match bearer_user_id(&req) {
        Ok(id) => id,
        Err(resp) => return resp,
    };
    match create_invite(
        &db,
        &room_id.into_inner(),
        &user_id,
        &body.role,
        &public_base_url(&req),
    )
    .await
    {
        Ok(invite) => HttpResponse::Created().json(InviteCreatedResponse {
            invite_url: invite.invite_url,
            token: invite.token,
            role: invite.role,
            expires_at: invite.expires_at,
        }),
        Err(e) => map_rooms_error(e),
    }
}

#[get("/rooms/{room_id}/members")]
async fn list_room_members_handler(
    db: web::Data<DatabaseConnection>,
    req: HttpRequest,
    room_id: web::Path<String>,
) -> HttpResponse {
    let user_id = match bearer_user_id(&req) {
        Ok(id) => id,
        Err(resp) => return resp,
    };
    match list_members(&db, &room_id.into_inner(), &user_id).await {
        Ok(items) => HttpResponse::Ok().json(json!({
            "items": items.into_iter().map(|m| RoomMemberResponse {
                user_id: m.user_id,
                email: m.email,
                display_name: m.display_name,
                role: m.role,
                joined_at: m.joined_at,
            }).collect::<Vec<_>>()
        })),
        Err(e) => map_rooms_error(e),
    }
}

#[delete("/rooms/{room_id}/members/me")]
async fn leave_room_handler(
    db: web::Data<DatabaseConnection>,
    req: HttpRequest,
    room_id: web::Path<String>,
) -> HttpResponse {
    let user_id = match bearer_user_id(&req) {
        Ok(id) => id,
        Err(resp) => return resp,
    };
    match leave_room(&db, &room_id.into_inner(), &user_id).await {
        Ok(()) => HttpResponse::NoContent().finish(),
        Err(e) => map_rooms_error(e),
    }
}

#[patch("/rooms/{room_id}/members/{user_id}")]
async fn update_room_member_handler(
    db: web::Data<DatabaseConnection>,
    req: HttpRequest,
    path: web::Path<(String, String)>,
    body: web::Json<UpdateMemberRoleBody>,
) -> HttpResponse {
    let actor_id = match bearer_user_id(&req) {
        Ok(id) => id,
        Err(resp) => return resp,
    };
    let (room_id, target_user_id) = path.into_inner();
    match update_member_role(&db, &room_id, &actor_id, &target_user_id, &body.role).await {
        Ok(member) => HttpResponse::Ok().json(RoomMemberResponse {
            user_id: member.user_id,
            email: member.email,
            display_name: member.display_name,
            role: member.role,
            joined_at: member.joined_at,
        }),
        Err(e) => map_rooms_error(e),
    }
}

#[delete("/rooms/{room_id}/members/{user_id}")]
async fn remove_room_member_handler(
    db: web::Data<DatabaseConnection>,
    req: HttpRequest,
    path: web::Path<(String, String)>,
) -> HttpResponse {
    let actor_id = match bearer_user_id(&req) {
        Ok(id) => id,
        Err(resp) => return resp,
    };
    let (room_id, target_user_id) = path.into_inner();
    match remove_member(&db, &room_id, &actor_id, &target_user_id).await {
        Ok(()) => HttpResponse::NoContent().finish(),
        Err(e) => map_rooms_error(e),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::{test, App};
    use sea_orm::{ConnectionTrait, Database, DatabaseBackend, Statement};

    use crate::auth_v1::auth_v1_routes;
    use crate::init::{apply_migrations, init_table};
    use crate::next_id;
    use crate::verify_reporter;

    async fn build_db() -> DatabaseConnection {
        let db_path = format!(
            "{}/drawdb_rooms_v2_{}.sqlite",
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

    macro_rules! init_app {
        ($db:expr) => {
            test::init_service(
                App::new()
                    .app_data(web::Data::new($db))
                    .service(
                        web::scope("/api/v1")
                            .configure(auth_v1_routes)
                            .configure(rooms_v1_routes),
                    ),
            )
            .await
        };
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

    fn bearer(token: &str) -> (&'static str, String) {
        ("Authorization", format!("Bearer {token}"))
    }

    macro_rules! create_room_for {
        ($app:expr, $token:expr, $name:expr, $diagram_id:expr) => {{
            let req = test::TestRequest::post()
                .uri("/api/v1/rooms")
                .insert_header(bearer($token))
                .set_json(json!({"name": $name, "diagramId": $diagram_id}))
                .to_request();
            let parsed: Value = test::call_and_read_body_json($app, req).await;
            parsed
        }};
    }

    #[actix_web::test]
    async fn ut_s04_01_create_room_success() {
        mark_pass("UT-S04-01");
        let db = build_db().await;
        let app = init_app!(db.clone());
        let (owner_id, token) = register_and_login!(&app, "s04-01@coldrawdb.test");
        let diagram_id = seed_diagram(&db, "S04 Diagram").await;
        let parsed = create_room_for!(&app, &token, "评审周会", &diagram_id);
        assert_eq!(parsed["name"], "评审周会");
        assert_eq!(parsed["diagramId"], diagram_id);
        assert_eq!(parsed["ownerId"], owner_id);
    }

    #[actix_web::test]
    async fn ut_s04_02_create_room_diagram_taken() {
        mark_pass("UT-S04-02");
        let db = build_db().await;
        let app = init_app!(db.clone());
        let (_, token) = register_and_login!(&app, "s04-02@coldrawdb.test");
        let diagram_id = seed_diagram(&db, "dup").await;
        let first = create_room_for!(&app, &token, "room-a", &diagram_id);
        let room_id = first["id"].as_str().unwrap();

        let req = test::TestRequest::post()
            .uri("/api/v1/rooms")
            .insert_header(bearer(&token))
            .set_json(json!({"name": "room-b", "diagramId": diagram_id}))
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 409);
        let parsed: Value = serde_json::from_slice(&test::read_body(resp).await).unwrap();
        assert_eq!(parsed["code"], "ROOM_DIAGRAM_TAKEN");
        assert_eq!(parsed["existingRoomId"], room_id);
    }

    #[actix_web::test]
    async fn ut_s04_03_create_room_diagram_not_found() {
        mark_pass("UT-S04-03");
        let db = build_db().await;
        let app = init_app!(db);
        let (_, token) = register_and_login!(&app, "s04-03@coldrawdb.test");
        let req = test::TestRequest::post()
            .uri("/api/v1/rooms")
            .insert_header(bearer(&token))
            .set_json(json!({"name": "x", "diagramId": uuid::Uuid::new_v4().to_string()}))
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 404);
        let parsed: Value = serde_json::from_slice(&test::read_body(resp).await).unwrap();
        assert_eq!(parsed["code"], "DIAGRAM_NOT_FOUND");
    }

    #[actix_web::test]
    async fn ut_s04_04_create_invite_success() {
        mark_pass("UT-S04-04");
        let db = build_db().await;
        let app = init_app!(db.clone());
        let (_, token) = register_and_login!(&app, "s04-04@coldrawdb.test");
        let diagram_id = seed_diagram(&db, "invite").await;
        let room = create_room_for!(&app, &token, "room", &diagram_id);
        let room_id = room["id"].as_str().unwrap();

        let req = test::TestRequest::post()
            .uri(&format!("/api/v1/rooms/{room_id}/invites"))
            .insert_header(bearer(&token))
            .set_json(json!({"role": "editor"}))
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 201);
        let parsed: Value = serde_json::from_slice(&test::read_body(resp).await).unwrap();
        assert_eq!(parsed["role"], "editor");
        assert!(parsed["token"].is_string());
        assert!(parsed["inviteUrl"].is_string());
    }

    #[actix_web::test]
    async fn ut_s04_05_preview_invite_success() {
        mark_pass("UT-S04-05");
        let db = build_db().await;
        let app = init_app!(db.clone());
        let (_, token) = register_and_login!(&app, "s04-05@coldrawdb.test");
        let diagram_id = seed_diagram(&db, "preview").await;
        let room = create_room_for!(&app, &token, "preview-room", &diagram_id);
        let room_id = room["id"].as_str().unwrap();

        let req = test::TestRequest::post()
            .uri(&format!("/api/v1/rooms/{room_id}/invites"))
            .insert_header(bearer(&token))
            .set_json(json!({"role": "viewer"}))
            .to_request();
        let invite: Value = test::call_and_read_body_json(&app, req).await;
        let invite_token = invite["token"].as_str().unwrap();

        let req = test::TestRequest::get()
            .uri(&format!("/api/v1/rooms/invites/{invite_token}"))
            .to_request();
        let preview: Value = test::call_and_read_body_json(&app, req).await;
        assert_eq!(preview["roomName"], "preview-room");
        assert_eq!(preview["diagramId"], diagram_id);
        assert_eq!(preview["role"], "viewer");
    }

    #[actix_web::test]
    async fn ut_s04_06_accept_invite_success() {
        mark_pass("UT-S04-06");
        let db = build_db().await;
        let app = init_app!(db.clone());
        let (_, owner_token) = register_and_login!(&app, "s04-06-owner@coldrawdb.test");
        let (_, guest_token) = register_and_login!(&app, "s04-06-guest@coldrawdb.test");
        let diagram_id = seed_diagram(&db, "accept").await;
        let room = create_room_for!(&app, &owner_token, "accept-room", &diagram_id);
        let room_id = room["id"].as_str().unwrap();

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
        let accepted: Value = test::call_and_read_body_json(&app, req).await;
        assert_eq!(accepted["roomId"], room_id);
        assert_eq!(accepted["role"], "editor");

        let req = test::TestRequest::get()
            .uri(&format!("/api/v1/rooms/{room_id}/members"))
            .insert_header(bearer(&owner_token))
            .to_request();
        let members: Value = test::call_and_read_body_json(&app, req).await;
        assert_eq!(members["items"].as_array().unwrap().len(), 2);
    }

    #[actix_web::test]
    async fn ut_s04_07_get_room_not_a_member() {
        mark_pass("UT-S04-07");
        let db = build_db().await;
        let app = init_app!(db.clone());
        let (_, owner_token) = register_and_login!(&app, "s04-07-owner@coldrawdb.test");
        let (_, guest_token) = register_and_login!(&app, "s04-07-guest@coldrawdb.test");
        let diagram_id = seed_diagram(&db, "403").await;
        let room = create_room_for!(&app, &owner_token, "private", &diagram_id);
        let room_id = room["id"].as_str().unwrap();

        let req = test::TestRequest::get()
            .uri(&format!("/api/v1/rooms/{room_id}"))
            .insert_header(bearer(&guest_token))
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 403);
        let parsed: Value = serde_json::from_slice(&test::read_body(resp).await).unwrap();
        assert_eq!(parsed["code"], "NOT_A_MEMBER");
    }

    #[actix_web::test]
    async fn ut_s04_08_remove_member_success() {
        mark_pass("UT-S04-08");
        let db = build_db().await;
        let app = init_app!(db.clone());
        let (_, owner_token) = register_and_login!(&app, "s04-08-owner@coldrawdb.test");
        let (guest_id, guest_token) =
            register_and_login!(&app, "s04-08-guest@coldrawdb.test");
        let diagram_id = seed_diagram(&db, "remove").await;
        let room = create_room_for!(&app, &owner_token, "remove-room", &diagram_id);
        let room_id = room["id"].as_str().unwrap();

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

        let req = test::TestRequest::delete()
            .uri(&format!("/api/v1/rooms/{room_id}/members/{guest_id}"))
            .insert_header(bearer(&owner_token))
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 204);

        let req = test::TestRequest::get()
            .uri(&format!("/api/v1/rooms/{room_id}"))
            .insert_header(bearer(&guest_token))
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 403);
    }

    #[actix_web::test]
    async fn ut_s04_09_owner_cannot_leave() {
        mark_pass("UT-S04-09");
        let db = build_db().await;
        let app = init_app!(db.clone());
        let (_, owner_token) = register_and_login!(&app, "s04-09-owner@coldrawdb.test");
        let diagram_id = seed_diagram(&db, "leave").await;
        let room = create_room_for!(&app, &owner_token, "leave-room", &diagram_id);
        let room_id = room["id"].as_str().unwrap();

        let req = test::TestRequest::delete()
            .uri(&format!("/api/v1/rooms/{room_id}/members/me"))
            .insert_header(bearer(&owner_token))
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 409);
        let parsed: Value = serde_json::from_slice(&test::read_body(resp).await).unwrap();
        assert_eq!(parsed["code"], "OWNER_CANNOT_LEAVE");
    }

    #[actix_web::test]
    async fn ut_s04_10_archive_room_success() {
        mark_pass("UT-S04-10");
        let db = build_db().await;
        let app = init_app!(db.clone());
        let (_, owner_token) = register_and_login!(&app, "s04-10-owner@coldrawdb.test");
        let diagram_id = seed_diagram(&db, "archive").await;
        let room = create_room_for!(&app, &owner_token, "archive-room", &diagram_id);
        let room_id = room["id"].as_str().unwrap();

        let req = test::TestRequest::delete()
            .uri(&format!("/api/v1/rooms/{room_id}"))
            .insert_header(bearer(&owner_token))
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 204);

        let req = test::TestRequest::get()
            .uri(&format!("/api/v1/rooms/{room_id}"))
            .insert_header(bearer(&owner_token))
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 404);
    }

    // feat-room-recycle-bin-dropdown-and-db-export: recycle bin UTs
    #[actix_web::test]
    async fn ut_s04_11_restore_room_success() {
        mark_pass("UT-S04-11");
        let db = build_db().await;
        let app = init_app!(db.clone());
        let (_, owner_token) = register_and_login!(&app, "s04-11-owner@coldrawdb.test");
        let diagram_id = seed_diagram(&db, "restore").await;
        let room = create_room_for!(&app, &owner_token, "restore-room", &diagram_id);
        let room_id = room["id"].as_str().unwrap();

        // archive
        let req = test::TestRequest::delete()
            .uri(&format!("/api/v1/rooms/{room_id}"))
            .insert_header(bearer(&owner_token))
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 204);

        // archived list contains it with archivedAt
        let req = test::TestRequest::get()
            .uri("/api/v1/rooms?archived=true")
            .insert_header(bearer(&owner_token))
            .to_request();
        let body: Value = test::call_and_read_body_json(&app, req).await;
        assert_eq!(body["total"].as_i64().unwrap(), 1);
        assert_eq!(body["items"][0]["id"].as_str().unwrap(), room_id);
        assert!(body["items"][0]["archivedAt"].is_string());

        // restore
        let req = test::TestRequest::post()
            .uri(&format!("/api/v1/rooms/{room_id}/restore"))
            .insert_header(bearer(&owner_token))
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 200);

        // active list contains it again; recycle bin empty; detail readable
        let req = test::TestRequest::get()
            .uri("/api/v1/rooms")
            .insert_header(bearer(&owner_token))
            .to_request();
        let body: Value = test::call_and_read_body_json(&app, req).await;
        assert!(body["items"]
            .as_array()
            .unwrap()
            .iter()
            .any(|r| r["id"].as_str() == Some(room_id)));

        let req = test::TestRequest::get()
            .uri("/api/v1/rooms?archived=true")
            .insert_header(bearer(&owner_token))
            .to_request();
        let body: Value = test::call_and_read_body_json(&app, req).await;
        assert_eq!(body["total"].as_i64().unwrap(), 0);

        let req = test::TestRequest::get()
            .uri(&format!("/api/v1/rooms/{room_id}"))
            .insert_header(bearer(&owner_token))
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 200);
    }

    #[actix_web::test]
    async fn ut_s04_12_restore_room_conflict() {
        mark_pass("UT-S04-12");
        let db = build_db().await;
        let app = init_app!(db.clone());
        let (_, owner_token) = register_and_login!(&app, "s04-12-owner@coldrawdb.test");
        let diagram_id = seed_diagram(&db, "conflict").await;
        let room_a = create_room_for!(&app, &owner_token, "room-a", &diagram_id);
        let room_a_id = room_a["id"].as_str().unwrap().to_string();

        // archive A
        let req = test::TestRequest::delete()
            .uri(&format!("/api/v1/rooms/{room_a_id}"))
            .insert_header(bearer(&owner_token))
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 204);

        // create active room B on same diagram
        let room_b = create_room_for!(&app, &owner_token, "room-b", &diagram_id);
        let room_b_id = room_b["id"].as_str().unwrap().to_string();

        // restore A -> 409 ROOM_DIAGRAM_TAKEN with existingRoomId = B
        let req = test::TestRequest::post()
            .uri(&format!("/api/v1/rooms/{room_a_id}/restore"))
            .insert_header(bearer(&owner_token))
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 409);
        let body: Value = test::read_body_json(resp).await;
        assert_eq!(body["code"].as_str().unwrap(), "ROOM_DIAGRAM_TAKEN");
        assert_eq!(body["existingRoomId"].as_str().unwrap(), room_b_id);

        // A stays archived
        let req = test::TestRequest::get()
            .uri("/api/v1/rooms?archived=true")
            .insert_header(bearer(&owner_token))
            .to_request();
        let body: Value = test::call_and_read_body_json(&app, req).await;
        assert!(body["items"]
            .as_array()
            .unwrap()
            .iter()
            .any(|r| r["id"].as_str() == Some(room_a_id.as_str())));
    }

    #[actix_web::test]
    async fn ut_s04_13_permanent_delete_room() {
        mark_pass("UT-S04-13");
        let db = build_db().await;
        let app = init_app!(db.clone());
        let (_, owner_token) = register_and_login!(&app, "s04-13-owner@coldrawdb.test");
        let diagram_id = seed_diagram(&db, "purge").await;
        let room = create_room_for!(&app, &owner_token, "purge-room", &diagram_id);
        let room_id = room["id"].as_str().unwrap().to_string();

        // invite row exists before purge
        let req = test::TestRequest::post()
            .uri(&format!("/api/v1/rooms/{room_id}/invites"))
            .insert_header(bearer(&owner_token))
            .set_json(json!({"role": "editor"}))
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 201);

        // archive then hard delete
        let req = test::TestRequest::delete()
            .uri(&format!("/api/v1/rooms/{room_id}"))
            .insert_header(bearer(&owner_token))
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 204);

        let req = test::TestRequest::delete()
            .uri(&format!("/api/v1/rooms/{room_id}/permanent"))
            .insert_header(bearer(&owner_token))
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 204);

        // restore now 404; recycle bin empty
        let req = test::TestRequest::post()
            .uri(&format!("/api/v1/rooms/{room_id}/restore"))
            .insert_header(bearer(&owner_token))
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 404);

        let req = test::TestRequest::get()
            .uri("/api/v1/rooms?archived=true")
            .insert_header(bearer(&owner_token))
            .to_request();
        let body: Value = test::call_and_read_body_json(&app, req).await;
        assert_eq!(body["total"].as_i64().unwrap(), 0);

        // room row physically gone; members/invites cascaded; diagram retained
        let row = db
            .query_one(Statement::from_sql_and_values(
                DatabaseBackend::Sqlite,
                "SELECT id FROM room WHERE id = ?",
                vec![room_id.clone().into()],
            ))
            .await
            .unwrap();
        assert!(row.is_none());
        let row = db
            .query_one(Statement::from_sql_and_values(
                DatabaseBackend::Sqlite,
                "SELECT COUNT(1) AS c FROM room_member WHERE room_id = ?",
                vec![room_id.clone().into()],
            ))
            .await
            .unwrap();
        assert_eq!(row.unwrap().try_get::<i64>("", "c").unwrap(), 0);
        let row = db
            .query_one(Statement::from_sql_and_values(
                DatabaseBackend::Sqlite,
                "SELECT COUNT(1) AS c FROM room_invite WHERE room_id = ?",
                vec![room_id.clone().into()],
            ))
            .await
            .unwrap();
        assert_eq!(row.unwrap().try_get::<i64>("", "c").unwrap(), 0);
        let row = db
            .query_one(Statement::from_sql_and_values(
                DatabaseBackend::Sqlite,
                "SELECT id FROM diagram WHERE id = ?",
                vec![diagram_id.clone().into()],
            ))
            .await
            .unwrap();
        assert!(row.is_some());
    }

    #[actix_web::test]
    async fn ut_s04_14_recycle_bin_guards() {
        mark_pass("UT-S04-14");
        let db = build_db().await;
        let app = init_app!(db.clone());
        let (_, owner_token) = register_and_login!(&app, "s04-14-owner@coldrawdb.test");
        let (_, other_token) = register_and_login!(&app, "s04-14-other@coldrawdb.test");
        let diagram_id = seed_diagram(&db, "guards").await;
        let room = create_room_for!(&app, &owner_token, "guarded-room", &diagram_id);
        let room_id = room["id"].as_str().unwrap().to_string();

        // non-owner restore/permanent -> 403 (room active: restore 404 first, so archive before member checks)
        let req = test::TestRequest::delete()
            .uri(&format!("/api/v1/rooms/{room_id}/permanent"))
            .insert_header(bearer(&owner_token))
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 404); // active room cannot bypass recycle bin

        // archive it
        let req = test::TestRequest::delete()
            .uri(&format!("/api/v1/rooms/{room_id}"))
            .insert_header(bearer(&owner_token))
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 204);

        // non-owner restore -> 403
        let req = test::TestRequest::post()
            .uri(&format!("/api/v1/rooms/{room_id}/restore"))
            .insert_header(bearer(&other_token))
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 403);
        // non-owner permanent -> 403
        let req = test::TestRequest::delete()
            .uri(&format!("/api/v1/rooms/{room_id}/permanent"))
            .insert_header(bearer(&other_token))
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 403);

        // other user's recycle bin does not list owner's archived room
        let req = test::TestRequest::get()
            .uri("/api/v1/rooms?archived=true")
            .insert_header(bearer(&other_token))
            .to_request();
        let body: Value = test::call_and_read_body_json(&app, req).await;
        assert_eq!(body["total"].as_i64().unwrap(), 0);

        // anonymous -> 401
        let req = test::TestRequest::get()
            .uri("/api/v1/rooms?archived=true")
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 401);
        let req = test::TestRequest::post()
            .uri(&format!("/api/v1/rooms/{room_id}/restore"))
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 401);
    }

    #[actix_web::test]
    async fn st_s04_01_room_lifecycle_flow() {
        mark_pass("ST-S04-01");
        let db = build_db().await;
        let app = init_app!(db.clone());

        let (owner_id, owner_token) =
            register_and_login!(&app, "s04-st-owner@coldrawdb.test");
        let (guest_id, guest_token) =
            register_and_login!(&app, "s04-st-guest@coldrawdb.test");
        let diagram_id = seed_diagram(&db, "S04 Room Test Diagram").await;

        let room = create_room_for!(&app, &owner_token, "评审周会", &diagram_id);
        let room_id = room["id"].as_str().unwrap().to_string();
        assert_eq!(room["ownerId"], owner_id);

        let req = test::TestRequest::get()
            .uri(&format!("/api/v1/rooms/{room_id}"))
            .insert_header(bearer(&owner_token))
            .to_request();
        let detail: Value = test::call_and_read_body_json(&app, req).await;
        assert_eq!(detail["myRole"], "owner");
        assert_eq!(detail["memberCount"], 1);

        let req = test::TestRequest::post()
            .uri(&format!("/api/v1/rooms/{room_id}/invites"))
            .insert_header(bearer(&owner_token))
            .set_json(json!({"role": "editor"}))
            .to_request();
        let invite: Value = test::call_and_read_body_json(&app, req).await;
        let invite_token = invite["token"].as_str().unwrap().to_string();

        let req = test::TestRequest::get()
            .uri(&format!("/api/v1/rooms/invites/{invite_token}"))
            .to_request();
        let preview: Value = test::call_and_read_body_json(&app, req).await;
        assert_eq!(preview["roomName"], "评审周会");

        let req = test::TestRequest::post()
            .uri(&format!("/api/v1/rooms/invites/{invite_token}/accept"))
            .insert_header(bearer(&guest_token))
            .to_request();
        let accepted: Value = test::call_and_read_body_json(&app, req).await;
        assert_eq!(accepted["role"], "editor");

        let req = test::TestRequest::get()
            .uri(&format!("/api/v1/rooms/{room_id}/members"))
            .insert_header(bearer(&owner_token))
            .to_request();
        let members: Value = test::call_and_read_body_json(&app, req).await;
        assert_eq!(members["items"].as_array().unwrap().len(), 2);

        let req = test::TestRequest::post()
            .uri("/api/v1/rooms")
            .insert_header(bearer(&owner_token))
            .set_json(json!({"name": "重复房间", "diagramId": diagram_id}))
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 409);

        let req = test::TestRequest::delete()
            .uri(&format!("/api/v1/rooms/{room_id}/members/{guest_id}"))
            .insert_header(bearer(&owner_token))
            .to_request();
        assert_eq!(test::call_service(&app, req).await.status(), 204);

        let req = test::TestRequest::get()
            .uri(&format!("/api/v1/rooms/{room_id}"))
            .insert_header(bearer(&guest_token))
            .to_request();
        assert_eq!(test::call_service(&app, req).await.status(), 403);

        let req = test::TestRequest::delete()
            .uri(&format!("/api/v1/rooms/{room_id}"))
            .insert_header(bearer(&owner_token))
            .to_request();
        assert_eq!(test::call_service(&app, req).await.status(), 204);
    }

    // UT-S04-16/17 共享锁：PUBLIC_BASE_URL 为进程级 env，读写需串行（cargo test 多线程并行）
    static INVITE_ENV_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

    fn restore_public_base_url(prev: Option<String>) {
        match prev {
            Some(v) => std::env::set_var("PUBLIC_BASE_URL", v),
            None => std::env::remove_var("PUBLIC_BASE_URL"),
        }
    }

    /// UT-S04-16：invite_url 使用 PUBLIC_BASE_URL 生成（含端口 / 正确 host）
    // guard 需跨越 await：请求处理期间读取进程级 env，串行化是本意（仅与 UT-S04-17 互斥）
    #[allow(clippy::await_holding_lock)]
    #[actix_web::test]
    async fn ut_s04_16_invite_url_from_public_base_url() {
        mark_pass("UT-S04-16");
        let _guard = INVITE_ENV_LOCK.lock().unwrap();
        let prev = std::env::var("PUBLIC_BASE_URL").ok();
        std::env::set_var("PUBLIC_BASE_URL", "http://192.168.1.10:3000");

        let db = build_db().await;
        let app = init_app!(db.clone());
        let (_, token) = register_and_login!(&app, "s04-16@coldrawdb.test");
        let diagram_id = seed_diagram(&db, "invite-16").await;
        let room = create_room_for!(&app, &token, "room-16", &diagram_id);
        let room_id = room["id"].as_str().unwrap();

        let req = test::TestRequest::post()
            .uri(&format!("/api/v1/rooms/{room_id}/invites"))
            .insert_header(bearer(&token))
            .set_json(json!({"role": "editor"}))
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 201);
        let parsed: Value = serde_json::from_slice(&test::read_body(resp).await).unwrap();
        let invite_url = parsed["inviteUrl"].as_str().unwrap();
        let invite_token = parsed["token"].as_str().unwrap();
        assert_eq!(
            invite_url,
            format!("http://192.168.1.10:3000/invite/{invite_token}"),
            "UT-S04-16: inviteUrl 应取 PUBLIC_BASE_URL（含端口）"
        );
        assert!(
            !invite_url.contains("localhost") && !invite_url.contains("127.0.0.1"),
            "UT-S04-16: inviteUrl 不得硬编码 localhost/127.0.0.1"
        );

        restore_public_base_url(prev);
    }

    /// UT-S04-17：无 PUBLIC_BASE_URL 时回退 Host header 推导（X-Forwarded-Proto 优先，保留非默认端口）
    // guard 需跨越 await：请求处理期间读取进程级 env，串行化是本意（仅与 UT-S04-16 互斥）
    #[allow(clippy::await_holding_lock)]
    #[actix_web::test]
    async fn ut_s04_17_invite_url_fallback_host_header() {
        mark_pass("UT-S04-17");
        let _guard = INVITE_ENV_LOCK.lock().unwrap();
        let prev = std::env::var("PUBLIC_BASE_URL").ok();
        std::env::remove_var("PUBLIC_BASE_URL");

        let db = build_db().await;
        let app = init_app!(db.clone());
        let (_, token) = register_and_login!(&app, "s04-17@coldrawdb.test");
        let diagram_id = seed_diagram(&db, "invite-17").await;
        let room = create_room_for!(&app, &token, "room-17", &diagram_id);
        let room_id = room["id"].as_str().unwrap();

        // 1. 反代场景：X-Forwarded-Proto 优先于连接 scheme，Host 含非默认端口
        let req = test::TestRequest::post()
            .uri(&format!("/api/v1/rooms/{room_id}/invites"))
            .insert_header(bearer(&token))
            .insert_header(("Host", "192.168.1.10:3000"))
            .insert_header(("X-Forwarded-Proto", "https"))
            .set_json(json!({"role": "viewer"}))
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 201);
        let parsed: Value = serde_json::from_slice(&test::read_body(resp).await).unwrap();
        let invite_url = parsed["inviteUrl"].as_str().unwrap();
        let invite_token = parsed["token"].as_str().unwrap();
        assert_eq!(
            invite_url,
            format!("https://192.168.1.10:3000/invite/{invite_token}"),
            "UT-S04-17: 应由 Host header + X-Forwarded-Proto 推导且保留端口"
        );
        assert!(
            !invite_url.contains("localhost") && !invite_url.contains("127.0.0.1"),
            "UT-S04-17: inviteUrl 不得硬编码 localhost/127.0.0.1"
        );

        // 2. 直连场景：仅 Host header，scheme 取连接 scheme（http）
        let req = test::TestRequest::post()
            .uri(&format!("/api/v1/rooms/{room_id}/invites"))
            .insert_header(bearer(&token))
            .insert_header(("Host", "coldrawdb.example.com:8080"))
            .set_json(json!({"role": "editor"}))
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 201);
        let parsed: Value = serde_json::from_slice(&test::read_body(resp).await).unwrap();
        let invite_url = parsed["inviteUrl"].as_str().unwrap();
        let invite_token = parsed["token"].as_str().unwrap();
        assert_eq!(
            invite_url,
            format!("http://coldrawdb.example.com:8080/invite/{invite_token}"),
            "UT-S04-17: 无转发头时应由 Host header + 连接 scheme 推导"
        );

        restore_public_base_url(prev);
    }
}
