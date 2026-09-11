use actix_web::{delete, get, post, put, web, HttpRequest, HttpResponse};
use sea_orm::{ConnectionTrait, DatabaseBackend, DatabaseConnection, Statement, TransactionTrait};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::collab::{get_room_document, room_id_for_diagram};
use crate::diagram_persistence::{load_diagram, persist_import_payload, save_diagram, DiagramFull, SaveDiagramError};
use crate::error::DrawDBError;
use crate::next_id;

#[derive(Serialize)]
struct ApiResp<T: Serialize> {
    code: i32,
    data: T,
    request_id: String,
}

#[derive(Serialize)]
struct ApiErr {
    code: i32,
    message: String,
    request_id: String,
    details: Option<Value>,
}

#[derive(Deserialize)]
struct CreateReq {
    name: String,
    database: Option<String>,
}

#[derive(Deserialize)]
struct SaveReq {
    expected_revision: i64,
    diagram: DiagramFull,
}

#[derive(Deserialize)]
struct ImportReq {
    source: Option<String>,
    payload: Value,
}

#[derive(Serialize)]
struct ImportResult {
    diagram_id: String,
    imported_tables: i64,
    imported_fields: i64,
    warnings: Vec<String>,
    source: Option<String>,
}

fn esc(s: &str) -> String { s.replace('\'', "''") }

pub fn diagrams_v1_routes(config: &mut web::ServiceConfig) {
    config.service(health_v1);
    config.service(create_diagram_v1);
    config.service(get_diagram_v1);
    config.service(save_diagram_v1);
    config.service(delete_diagram_v1);
    config.service(import_diagram_v1);
}

/// feat-docker-compose-deploy（SMOKE-core-01 / 部署方案 §6）：健康检查端点。
/// 静态段 `health` 在 actix-web 路由中优先于动态段 `{id}`，无 DB 依赖。
#[get("/diagrams/health")]
async fn health_v1() -> HttpResponse {
    HttpResponse::Ok().json(serde_json::json!({"status": "ok"}))
}

#[post("/diagrams")]
async fn create_diagram_v1(db: web::Data<DatabaseConnection>, req: web::Json<CreateReq>) -> Result<HttpResponse, DrawDBError> {
    let request_id = next_id();
    let id = next_id();
    let tx = db.begin().await?;
    let sql = format!(
        "INSERT INTO diagram(id, name, database, pan, zoom, revision, updated_at, is_deleted) VALUES('{}','{}',{},'', '',0,datetime('now'),0)",
        esc(&id),
        esc(&req.name),
        req.database.as_ref().map(|v| format!("'{}'", esc(v))).unwrap_or("NULL".to_string())
    );
    tx.execute(Statement::from_sql_and_values(DatabaseBackend::Sqlite, sql, vec![])).await?;
    tx.commit().await?;
    Ok(HttpResponse::Ok().json(ApiResp { code: 0, data: serde_json::json!({"id": id}), request_id }))
}

#[get("/diagrams/{id}")]
async fn get_diagram_v1(db: web::Data<DatabaseConnection>, id: web::Path<String>) -> Result<HttpResponse, DrawDBError> {
    let request_id = next_id();
    // fix-collab-autosave-race（方案 B 单写者）：room 绑定且已物化 → 直接返回物化文档
    // （与 op log 恒一致；revision 注入当前 head server_rev）。首连/刷新/全量重载恒正确。
    if let Some((mut doc, rev)) = get_room_document(db.get_ref(), &id).await? {
        doc["revision"] = serde_json::json!(rev);
        return Ok(HttpResponse::Ok().json(ApiResp {
            code: 0,
            data: doc,
            request_id,
        }));
    }
    match load_diagram(db.get_ref(), &id).await? {
        Some(diagram) => Ok(HttpResponse::Ok().json(ApiResp {
            code: 0,
            data: diagram,
            request_id,
        })),
        None => Ok(HttpResponse::NotFound().json(ApiErr {
            code: 404,
            message: "not found".into(),
            request_id,
            details: None,
        })),
    }
}

#[put("/diagrams/{id}")]
async fn save_diagram_v1(db: web::Data<DatabaseConnection>, id: web::Path<String>, _http: HttpRequest, req: web::Json<SaveReq>) -> Result<HttpResponse, DrawDBError> {
    let request_id = next_id();
    let id = id.into_inner();

    // fix-collab-autosave-race（方案 B 单写者）：room 绑定 diagram 写收编——
    // 写只走 WS op 通道（op 落库即物化），全量快照 PUT 一律 409 USE_OP_CHANNEL。
    // 原 S05.5 checkpoint 头协议（X-Room-Id / X-Collab-Server-Rev）废弃；
    // 非房间 diagram 的 legacy 无鉴权路径不变（S01/S02 兼容）。
    if room_id_for_diagram(db.get_ref(), &id).await?.is_some() {
        return Ok(HttpResponse::Conflict().json(ApiErr {
            code: 409,
            message: "USE_OP_CHANNEL: 房间图表请通过协作 op 通道写入".into(),
            request_id,
            details: Some(serde_json::json!({"code": "USE_OP_CHANNEL"})),
        }));
    }

    match save_diagram(db.get_ref(), &id, req.expected_revision, &req.diagram).await {
        Ok(revision) => Ok(HttpResponse::Ok().json(ApiResp {
            code: 0,
            data: serde_json::json!({"id": id, "revision": revision}),
            request_id,
        })),
        Err(SaveDiagramError::NotFound) => Ok(HttpResponse::NotFound().json(ApiErr {
            code: 404,
            message: "not found".into(),
            request_id,
            details: None,
        })),
        Err(SaveDiagramError::BadRequest(msg)) => Ok(HttpResponse::BadRequest().json(ApiErr {
            code: 400,
            message: msg,
            request_id,
            details: None,
        })),
        Err(SaveDiagramError::Conflict { current_revision }) => Ok(HttpResponse::Conflict().json(ApiErr {
            code: 409,
            message: "revision conflict".into(),
            request_id,
            details: Some(serde_json::json!({"current_revision": current_revision})),
        })),
        Err(SaveDiagramError::Db(e)) => Err(DrawDBError::DatabaseError(e)),
    }
}

#[delete("/diagrams/{id}")]
async fn delete_diagram_v1(db: web::Data<DatabaseConnection>, id: web::Path<String>) -> Result<HttpResponse, DrawDBError> {
    let request_id = next_id();
    let tx = db.begin().await?;
    let sql = format!("UPDATE diagram SET is_deleted=1, updated_at=datetime('now') WHERE id='{}'", esc(&id));
    tx.execute(Statement::from_sql_and_values(DatabaseBackend::Sqlite, sql, vec![])).await?;
    tx.commit().await?;
    Ok(HttpResponse::Ok().json(ApiResp { code: 0, data: serde_json::json!({"id": id.into_inner()}), request_id }))
}

#[post("/diagrams/import")]
async fn import_diagram_v1(db: web::Data<DatabaseConnection>, req: web::Json<ImportReq>) -> Result<HttpResponse, DrawDBError> {
    let request_id = next_id();
    if !req.payload.is_object() {
        return Ok(HttpResponse::BadRequest().json(ApiErr {
            code: 400,
            message: "payload must be object".into(),
            request_id,
            details: Some(serde_json::json!({"field": "payload"})),
        }));
    }

    let id = next_id();
    let name = req.payload.get("name").and_then(|v| v.as_str()).unwrap_or("imported_diagram");
    let imported_tables = req
        .payload
        .get("tables")
        .and_then(|v| v.as_array())
        .map(|arr| arr.len() as i64)
        .unwrap_or(0);

    let imported_fields = req
        .payload
        .get("tables")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .map(|t| {
                    t.get("fields")
                        .and_then(|f| f.as_array())
                        .map(|f| f.len() as i64)
                        .unwrap_or(0)
                })
                .sum()
        })
        .unwrap_or(0);

    let mut warnings = vec![];
    if req.payload.get("tables").is_none() {
        warnings.push("tables missing, imported as empty".to_string());
    }

    let tx = db.begin().await?;
    let sql = format!(
        "INSERT INTO diagram(id, name, database, pan, zoom, revision, updated_at, is_deleted) VALUES('{}','{}',NULL,'','',0,datetime('now'),0)",
        esc(&id), esc(name)
    );
    tx.execute(Statement::from_sql_and_values(DatabaseBackend::Sqlite, sql, vec![])).await?;
    tx.commit().await?;

    let _ = persist_import_payload(db.get_ref(), &id, &req.payload).await;

    Ok(HttpResponse::Ok().json(ApiResp {
        code: 0,
        data: ImportResult {
            diagram_id: id,
            imported_tables,
            imported_fields,
            warnings,
            source: req.source.clone(),
        },
        request_id,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::{test, App};
    use sea_orm::Database;
    use crate::init::{init_table, apply_migrations};
    use crate::verify_reporter;

    async fn build_db() -> DatabaseConnection {
        let db_path = format!(
            "{}/drawdb_api_v1_{}.sqlite",
            std::env::temp_dir().display(),
            uuid::Uuid::new_v4()
        );
        if std::path::Path::new(&db_path).exists() { let _ = std::fs::remove_file(&db_path); }
        std::fs::File::create(&db_path).unwrap();
        let db = Database::connect(format!("sqlite://{}?", db_path)).await.unwrap();
        init_table("init.sql", &db).await.unwrap();
        apply_migrations("migrations", &db).await.unwrap();
        db
    }

    fn mark_pass(id: &'static str) {
        verify_reporter::report_pass(id, 0);
    }

    /// UT-DP-03：health 端点（feat-docker-compose-deploy，core-PV §5）
    ///
    /// 无 DB 依赖；同时验证静态段 `health` 优先于动态段 `{id}`（否则 handler 会因缺
    /// `web::Data<DatabaseConnection>` 而 500）。
    #[actix_web::test]
    async fn ut_dp_03_health_endpoint() {
        let app = test::init_service(
            App::new().service(web::scope("/api/v1").configure(diagrams_v1_routes)),
        )
        .await;
        let req = test::TestRequest::get()
            .uri("/api/v1/diagrams/health")
            .to_request();
        let res = test::call_service(&app, req).await;
        assert_eq!(res.status(), 200);
        let body: Value = test::read_body_json(res).await;
        assert_eq!(body["status"], "ok");
        mark_pass("UT-DP-03");
    }

    #[actix_web::test]
    async fn st_s01_01_diagram_crud_and_conflict() {
        mark_pass("ST-S01-01");
        let db = build_db().await;
        let app = test::init_service(
            App::new().app_data(web::Data::new(db)).service(web::scope("/api/v1").configure(diagrams_v1_routes))
        ).await;

        let req = test::TestRequest::post().uri("/api/v1/diagrams")
            .set_json(serde_json::json!({"name":"d1"})).to_request();
        let resp: Value = test::call_and_read_body_json(&app, req).await;
        let id = resp["data"]["id"].as_str().unwrap().to_string();

        let req = test::TestRequest::put().uri(&format!("/api/v1/diagrams/{}", id))
            .set_json(serde_json::json!({"expected_revision":0,"diagram":{"id":id,"name":"d2"}})).to_request();
        let ok = test::call_service(&app, req).await;
        assert!(ok.status().is_success());

        let req = test::TestRequest::put().uri(&format!("/api/v1/diagrams/{}", id))
            .set_json(serde_json::json!({"expected_revision":0,"diagram":{"id":id,"name":"d3"}})).to_request();
        let conflict = test::call_service(&app, req).await;
        assert_eq!(conflict.status(), 409);

        let req = test::TestRequest::delete().uri(&format!("/api/v1/diagrams/{}", id)).to_request();
        let del = test::call_service(&app, req).await;
        assert!(del.status().is_success());

        let req = test::TestRequest::get().uri(&format!("/api/v1/diagrams/{}", id)).to_request();
        let not_found = test::call_service(&app, req).await;
        assert_eq!(not_found.status(), 404);
    }

    #[actix_web::test]
    async fn st_s01_02_import_success_and_invalid_payload() {
        mark_pass("ST-S01-02");
        let db = build_db().await;
        let app = test::init_service(
            App::new().app_data(web::Data::new(db)).service(web::scope("/api/v1").configure(diagrams_v1_routes))
        ).await;
        let req = test::TestRequest::post().uri("/api/v1/diagrams/import")
            .set_json(serde_json::json!({
                "source": "localStorage",
                "payload": {"name":"import-1", "tables":[{"fields":[{"name":"id"},{"name":"n"}]}]}
            }))
            .to_request();
        let ok: Value = test::call_and_read_body_json(&app, req).await;
        assert_eq!(ok["code"], 0);
        assert_eq!(ok["data"]["imported_tables"], 1);
        assert_eq!(ok["data"]["imported_fields"], 2);
        let req = test::TestRequest::post().uri("/api/v1/diagrams/import")
            .set_json(serde_json::json!({"payload": "invalid"}))
            .to_request();
        let bad = test::call_service(&app, req).await;
        assert_eq!(bad.status(), 400);
    }

    #[actix_web::test]
    async fn ut_s01_01_create_empty_diagram() {
        mark_pass("UT-S01-01");
        let db = build_db().await;
        let app = test::init_service(
            App::new().app_data(web::Data::new(db)).service(web::scope("/api/v1").configure(diagrams_v1_routes))
        ).await;
        let req = test::TestRequest::post().uri("/api/v1/diagrams")
            .set_json(serde_json::json!({"name":"empty"})).to_request();
        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());
    }

    #[actix_web::test]
    async fn ut_s01_03_put_with_correct_revision() {
        mark_pass("UT-S01-03");
        let db = build_db().await;
        let app = test::init_service(
            App::new().app_data(web::Data::new(db)).service(web::scope("/api/v1").configure(diagrams_v1_routes))
        ).await;
        let req = test::TestRequest::post().uri("/api/v1/diagrams")
            .set_json(serde_json::json!({"name":"d"})).to_request();
        let resp: Value = test::call_and_read_body_json(&app, req).await;
        let did = resp["data"]["id"].as_str().unwrap().to_string();
        let req = test::TestRequest::put().uri(&format!("/api/v1/diagrams/{}", did))
            .set_json(serde_json::json!({"expected_revision":0,"diagram":{"id":did,"name":"d2"}})).to_request();
        let ok = test::call_service(&app, req).await;
        assert_eq!(ok.status(), 200);
    }

    #[actix_web::test]
    async fn ut_s01_04_put_with_stale_revision() {
        mark_pass("UT-S01-04");
        let db = build_db().await;
        let app = test::init_service(
            App::new().app_data(web::Data::new(db)).service(web::scope("/api/v1").configure(diagrams_v1_routes))
        ).await;
        let req = test::TestRequest::post().uri("/api/v1/diagrams")
            .set_json(serde_json::json!({"name":"d"})).to_request();
        let resp: Value = test::call_and_read_body_json(&app, req).await;
        let did = resp["data"]["id"].as_str().unwrap().to_string();
        let req = test::TestRequest::put().uri(&format!("/api/v1/diagrams/{}", did))
            .set_json(serde_json::json!({"expected_revision":0,"diagram":{"id":did,"name":"d2"}})).to_request();
        let _ = test::call_service(&app, req).await;
        let req = test::TestRequest::put().uri(&format!("/api/v1/diagrams/{}", did))
            .set_json(serde_json::json!({"expected_revision":0,"diagram":{"id":did,"name":"d3"}})).to_request();
        let conflict = test::call_service(&app, req).await;
        assert_eq!(conflict.status(), 409);
    }

    #[actix_web::test]
    async fn ut_s01_05_delete_cascades() {
        mark_pass("UT-S01-05");
        let db = build_db().await;
        let app = test::init_service(
            App::new().app_data(web::Data::new(db)).service(web::scope("/api/v1").configure(diagrams_v1_routes))
        ).await;
        let req = test::TestRequest::post().uri("/api/v1/diagrams")
            .set_json(serde_json::json!({"name":"to-delete"})).to_request();
        let resp: Value = test::call_and_read_body_json(&app, req).await;
        let did = resp["data"]["id"].as_str().unwrap().to_string();
        let req = test::TestRequest::delete().uri(&format!("/api/v1/diagrams/{}", did)).to_request();
        let del = test::call_service(&app, req).await;
        assert!(del.status().is_success());
        let req = test::TestRequest::get().uri(&format!("/api/v1/diagrams/{}", did)).to_request();
        let nf = test::call_service(&app, req).await;
        assert_eq!(nf.status(), 404);
    }

    #[actix_web::test]
    async fn ut_s02_01_get_existing_diagram() {
        mark_pass("UT-S02-01");
        let db = build_db().await;
        let app = test::init_service(
            App::new().app_data(web::Data::new(db)).service(web::scope("/api/v1").configure(diagrams_v1_routes))
        ).await;
        let req = test::TestRequest::post().uri("/api/v1/diagrams")
            .set_json(serde_json::json!({"name":"shared","database":"mysql"})).to_request();
        let resp: Value = test::call_and_read_body_json(&app, req).await;
        let did = resp["data"]["id"].as_str().unwrap().to_string();
        let req = test::TestRequest::get().uri(&format!("/api/v1/diagrams/{}", did)).to_request();
        let got: Value = test::call_and_read_body_json(&app, req).await;
        assert_eq!(got["code"], 0);
        assert_eq!(got["data"]["id"], did);
        assert_eq!(got["data"]["name"], "shared");
        assert_eq!(got["data"]["database"], "mysql");
        assert_eq!(got["data"]["revision"], 0);
    }

    #[actix_web::test]
    async fn ut_s02_02_get_missing_diagram_returns_404() {
        mark_pass("UT-S02-02");
        let db = build_db().await;
        let app = test::init_service(
            App::new().app_data(web::Data::new(db)).service(web::scope("/api/v1").configure(diagrams_v1_routes))
        ).await;
        let req = test::TestRequest::get().uri("/api/v1/diagrams/999999999999").to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 404);
    }

    // -------------------------------------------------------------------
    // fix-collab-autosave-race（方案 B 服务端物化单写者）
    // -------------------------------------------------------------------

    async fn seed_user(db: &DatabaseConnection) -> String {
        let id = crate::next_id();
        db.execute(Statement::from_sql_and_values(
            DatabaseBackend::Sqlite,
            "INSERT INTO user(id, email, password_hash, display_name) VALUES(?, ?, 'x', 'T')",
            vec![id.clone().into(), format!("{id}@t.dev").into()],
        ))
        .await
        .unwrap();
        id
    }

    async fn seed_diagram(db: &DatabaseConnection, name: &str) -> String {
        let id = crate::next_id();
        db.execute(Statement::from_sql_and_values(
            DatabaseBackend::Sqlite,
            "INSERT INTO diagram(id, name, database, pan, zoom, revision, updated_at, is_deleted) VALUES(?, ?, NULL, '', '', 0, datetime('now'), 0)",
            vec![id.clone().into(), name.into()],
        ))
        .await
        .unwrap();
        id
    }

    /// UT-C-06 — op 落库即物化：doc_json 初始化 + table.create（含字段）落文档，与 head 一致
    #[actix_web::test]
    async fn ut_c06_append_op_materializes_doc() {
        mark_pass("UT-C-06");
        let db = build_db().await;
        let user_id = seed_user(&db).await;
        let diagram_id = seed_diagram(&db, "d").await;
        let room = crate::rooms::create_room(&db, &user_id, "r", &diagram_id).await.unwrap();

        // doc_json NULL 时首个 op：先从关系存储初始化，再 apply
        let op = serde_json::json!({"type":"table.create","targetId":"t1","changes":{"id":"t1","name":"users","x":1.0,"y":2.0,"color":"#fff","comment":"","fields":[{"id":"f1","name":"id","type_":"INT"}]}});
        let res = crate::collab::append_op(&db, &room.id, &user_id, "table.create", op).await.unwrap();
        assert_eq!(res.server_rev, 1);

        let (doc, rev) = crate::collab::get_room_document(&db, &diagram_id)
            .await
            .unwrap()
            .expect("物化文档应存在");
        assert_eq!(rev, 1, "doc 与 head 恒一致");
        let tables = doc["tables"].as_array().unwrap();
        assert_eq!(tables.len(), 1);
        assert_eq!(tables[0]["name"], "users");
        assert_eq!(tables[0]["fields"].as_array().unwrap().len(), 1, "table.create 字段随表物化");

        // 第二条 op 继续物化（非重复初始化）
        let op2 = serde_json::json!({"type":"note.create","targetId":"n1","changes":{"content":"hi"}});
        crate::collab::append_op(&db, &room.id, &user_id, "note.create", op2).await.unwrap();
        let (doc2, rev2) = crate::collab::get_room_document(&db, &diagram_id).await.unwrap().unwrap();
        assert_eq!(rev2, 2);
        assert_eq!(doc2["tables"].as_array().unwrap().len(), 1);
        assert_eq!(doc2["notes"].as_array().unwrap().len(), 1);
    }

    /// UT-C-08 — room 绑定 diagram 的全量 PUT 写收编（409 USE_OP_CHANNEL）；非 room legacy PUT 不变
    #[actix_web::test]
    async fn ut_c08_room_bound_put_rejected() {
        mark_pass("UT-C-08");
        let db = build_db().await;
        let user_id = seed_user(&db).await;
        let room_diagram = seed_diagram(&db, "room-d").await;
        crate::rooms::create_room(&db, &user_id, "r", &room_diagram).await.unwrap();
        let app = test::init_service(
            App::new().app_data(web::Data::new(db.clone())).service(web::scope("/api/v1").configure(diagrams_v1_routes))
        ).await;

        // 携带旧 checkpoint 头也一样拒绝（协议废弃）
        let req = test::TestRequest::put()
            .uri(&format!("/api/v1/diagrams/{room_diagram}"))
            .insert_header(("X-Room-Id", "whatever"))
            .insert_header(("X-Collab-Server-Rev", "0"))
            .set_json(serde_json::json!({"expected_revision":0,"diagram":{"id":room_diagram,"name":"x"}}))
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 409);

        // legacy 非 room diagram：全量 PUT 不受影响（S01 回归）
        let plain = seed_diagram(&db, "plain").await;
        let req = test::TestRequest::put()
            .uri(&format!("/api/v1/diagrams/{plain}"))
            .set_json(serde_json::json!({"expected_revision":0,"diagram":{"id":plain,"name":"p2"}}))
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());
    }

    /// UT-C-09 — GET 物化读：room 图 append op 后 GET 返回物化文档（revision=head）；未物化回退关系存储
    #[actix_web::test]
    async fn ut_c09_get_returns_materialized_doc() {
        mark_pass("UT-C-09");
        let db = build_db().await;
        let user_id = seed_user(&db).await;
        let diagram_id = seed_diagram(&db, "d").await;
        let room = crate::rooms::create_room(&db, &user_id, "r", &diagram_id).await.unwrap();
        crate::collab::append_op(&db, &room.id, &user_id, "table.create",
            serde_json::json!({"type":"table.create","targetId":"t1","changes":{"id":"t1","name":"users","fields":[]}}))
            .await.unwrap();
        crate::collab::append_op(&db, &room.id, &user_id, "note.create",
            serde_json::json!({"type":"note.create","targetId":"n1","changes":{"content":"hi"}}))
            .await.unwrap();

        let app = test::init_service(
            App::new().app_data(web::Data::new(db.clone())).service(web::scope("/api/v1").configure(diagrams_v1_routes))
        ).await;
        let req = test::TestRequest::get().uri(&format!("/api/v1/diagrams/{diagram_id}")).to_request();
        let got: Value = test::call_and_read_body_json(&app, req).await;
        assert_eq!(got["code"], 0);
        assert_eq!(got["data"]["tables"].as_array().unwrap().len(), 1, "GET 返回物化文档（含 op 效果）");
        assert_eq!(got["data"]["notes"].as_array().unwrap().len(), 1);
        assert_eq!(got["data"]["revision"], 2, "revision 注入当前 head server_rev");

        // 未物化（无 op）的 room 图：回退关系存储
        let diagram2 = seed_diagram(&db, "d2").await;
        crate::rooms::create_room(&db, &user_id, "r2", &diagram2).await.unwrap();
        let req = test::TestRequest::get().uri(&format!("/api/v1/diagrams/{diagram2}")).to_request();
        let got2: Value = test::call_and_read_body_json(&app, req).await;
        assert_eq!(got2["code"], 0);
        assert_eq!(got2["data"]["name"], "d2");
        assert_eq!(got2["data"]["revision"], 0);
    }
}
