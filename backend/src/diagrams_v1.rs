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

/// diagram-api-auth：强制鉴权开关的 app_data 覆盖点。
/// 生产不注册该 app_data → 回落读 `COLDRAWDB_DIAGRAMS_AUTH` 环境变量；
/// 测试经 `Option<web::Data<DiagramsAuthFlag>>` 精确控制，避免进程级 env 竞态。
pub struct DiagramsAuthFlag(pub bool);

/// diagram-api-auth：flag=on 时要求有效 Bearer token（auth_v1 access_token），缺失/非法 401；
/// flag=off（默认过渡态）匿名直通。Err 分支由 handler 直接作为响应返回。
pub fn guard_diagrams_auth(
    flag: Option<&DiagramsAuthFlag>,
    http: &HttpRequest,
    request_id: &str,
) -> Result<(), HttpResponse> {
    let required = flag
        .map(|f| f.0)
        .unwrap_or_else(crate::auth::diagrams_auth_required);
    if !required {
        return Ok(());
    }
    let token = http
        .headers()
        .get("Authorization")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("")
        .strip_prefix("Bearer ")
        .unwrap_or("");
    if token.is_empty() || crate::auth::verify_access_token(token).is_err() {
        return Err(HttpResponse::Unauthorized().json(ApiErr {
            code: 401,
            message: "请先登录".into(),
            request_id: request_id.to_string(),
            details: None,
        }));
    }
    Ok(())
}

/// diagram-api-auth：GET /diagrams/{id} 的匿名分享读凭证（S02 豁免）。
#[derive(Deserialize)]
struct ShareTokenQuery {
    share_token: Option<String>,
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
    config.service(share_diagram_v1);
}

/// feat-docker-compose-deploy（SMOKE-core-01 / 部署方案 §6）：健康检查端点。
/// 静态段 `health` 在 actix-web 路由中优先于动态段 `{id}`，无 DB 依赖。
#[get("/diagrams/health")]
async fn health_v1() -> HttpResponse {
    HttpResponse::Ok().json(serde_json::json!({"status": "ok"}))
}

#[post("/diagrams")]
async fn create_diagram_v1(
    db: web::Data<DatabaseConnection>,
    http: HttpRequest,
    flag: Option<web::Data<DiagramsAuthFlag>>,
    req: web::Json<CreateReq>,
) -> Result<HttpResponse, DrawDBError> {
    let request_id = next_id();
    // diagram-api-auth：flag=on 时无/非法 token → 401（写端点无匿名豁免）
    if let Err(resp) = guard_diagrams_auth(flag.as_ref().map(|d| d.get_ref()), &http, &request_id) {
        return Ok(resp);
    }
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
async fn get_diagram_v1(
    db: web::Data<DatabaseConnection>,
    id: web::Path<String>,
    http: HttpRequest,
    flag: Option<web::Data<DiagramsAuthFlag>>,
    query: web::Query<ShareTokenQuery>,
) -> Result<HttpResponse, DrawDBError> {
    let request_id = next_id();
    // diagram-api-auth：flag=on 时优先 Bearer token；无/非法 token 的唯一匿名豁免是
    // ?share_token= 匹配本图（S02 分享加载）；share_token 不匹配或图未开分享 → 404
    // （不暴露图存在性）；无 token 且无 share_token → 401。flag=off 全部跳过（同 v1.1）。
    if guard_diagrams_auth(flag.as_ref().map(|d| d.get_ref()), &http, &request_id).is_err() {
        let Some(share_token) = query.share_token.as_deref().filter(|s| !s.is_empty()) else {
            return Ok(HttpResponse::Unauthorized().json(ApiErr {
                code: 401,
                message: "请先登录".into(),
                request_id,
                details: None,
            }));
        };
        let row = db
            .query_one(Statement::from_sql_and_values(
                DatabaseBackend::Sqlite,
                format!(
                    "SELECT id FROM diagram WHERE id='{}' AND share_token='{}' AND is_deleted=0 LIMIT 1",
                    esc(&id),
                    esc(share_token)
                ),
                vec![],
            ))
            .await?;
        if row.is_none() {
            return Ok(HttpResponse::NotFound().json(ApiErr {
                code: 404,
                message: "not found".into(),
                request_id,
                details: None,
            }));
        }
    }
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
async fn save_diagram_v1(db: web::Data<DatabaseConnection>, id: web::Path<String>, http: HttpRequest, flag: Option<web::Data<DiagramsAuthFlag>>, req: web::Json<SaveReq>) -> Result<HttpResponse, DrawDBError> {
    let request_id = next_id();
    // diagram-api-auth：flag=on 时无/非法 token → 401（写端点无匿名豁免）
    if let Err(resp) = guard_diagrams_auth(flag.as_ref().map(|d| d.get_ref()), &http, &request_id) {
        return Ok(resp);
    }
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
async fn delete_diagram_v1(db: web::Data<DatabaseConnection>, id: web::Path<String>, http: HttpRequest, flag: Option<web::Data<DiagramsAuthFlag>>) -> Result<HttpResponse, DrawDBError> {
    let request_id = next_id();
    // diagram-api-auth：flag=on 时无/非法 token → 401（写端点无匿名豁免）
    if let Err(resp) = guard_diagrams_auth(flag.as_ref().map(|d| d.get_ref()), &http, &request_id) {
        return Ok(resp);
    }
    let tx = db.begin().await?;
    let sql = format!("UPDATE diagram SET is_deleted=1, updated_at=datetime('now') WHERE id='{}'", esc(&id));
    tx.execute(Statement::from_sql_and_values(DatabaseBackend::Sqlite, sql, vec![])).await?;
    tx.commit().await?;
    Ok(HttpResponse::Ok().json(ApiResp { code: 0, data: serde_json::json!({"id": id.into_inner()}), request_id }))
}

/// diagram-api-auth：铸造/轮换分享令牌（重复调用即轮换，旧分享链接立即失效）。
/// flag=on 时需 Bearer token（401）；图不存在或已软删 → 404。
/// 返回 data.{id, share_token}；share_token 为随机 URL-safe 串（uuid v4 simple）。
#[post("/diagrams/{id}/share")]
async fn share_diagram_v1(
    db: web::Data<DatabaseConnection>,
    id: web::Path<String>,
    http: HttpRequest,
    flag: Option<web::Data<DiagramsAuthFlag>>,
) -> Result<HttpResponse, DrawDBError> {
    let request_id = next_id();
    if let Err(resp) = guard_diagrams_auth(flag.as_ref().map(|d| d.get_ref()), &http, &request_id) {
        return Ok(resp);
    }
    let id = id.into_inner();
    let share_token = uuid::Uuid::new_v4().simple().to_string();
    let res = db
        .execute(Statement::from_sql_and_values(
            DatabaseBackend::Sqlite,
            format!(
                "UPDATE diagram SET share_token='{}', updated_at=datetime('now') WHERE id='{}' AND is_deleted=0",
                share_token,
                esc(&id)
            ),
            vec![],
        ))
        .await?;
    if res.rows_affected() == 0 {
        return Ok(HttpResponse::NotFound().json(ApiErr {
            code: 404,
            message: "not found".into(),
            request_id,
            details: None,
        }));
    }
    Ok(HttpResponse::Ok().json(ApiResp {
        code: 0,
        data: serde_json::json!({"id": id, "share_token": share_token}),
        request_id,
    }))
}

#[post("/diagrams/import")]
async fn import_diagram_v1(db: web::Data<DatabaseConnection>, http: HttpRequest, flag: Option<web::Data<DiagramsAuthFlag>>, req: web::Json<ImportReq>) -> Result<HttpResponse, DrawDBError> {
    let request_id = next_id();
    // diagram-api-auth：flag=on 时无/非法 token → 401（写端点无匿名豁免）
    if let Err(resp) = guard_diagrams_auth(flag.as_ref().map(|d| d.get_ref()), &http, &request_id) {
        return Ok(resp);
    }
    if !req.payload.is_object() {
        return Ok(HttpResponse::BadRequest().json(ApiErr {
            code: 400,
            message: "payload must be object".into(),
            request_id,
            details: Some(serde_json::json!({"field": "payload"})),
        }));
    }

    let id = next_id();
    // fix-diagram-import-persistence：payload.name 缺省由持久化层统一兜底
    // （normalize_import_payload 补 "imported_diagram"，与下方 INSERT 默认一致），
    // 避免 save_diagram 用 NULL 覆盖默认名。

    let tx = db.begin().await?;
    let name = req.payload.get("name").and_then(|v| v.as_str()).unwrap_or("imported_diagram");
    let sql = format!(
        "INSERT INTO diagram(id, name, database, pan, zoom, revision, updated_at, is_deleted) VALUES('{}','{}',NULL,'','',0,datetime('now'),0)",
        esc(&id), esc(name)
    );
    tx.execute(Statement::from_sql_and_values(DatabaseBackend::Sqlite, sql, vec![])).await?;
    tx.commit().await?;

    match persist_import_payload(db.get_ref(), &id, &req.payload).await {
        Ok(stats) => {
            // fix-diagram-import-persistence：数量按实际持久化上报，warnings 携带
            // 丢弃/补全明细（修复虚报 imported_tables 的假成功缺陷）。
            let mut warnings = stats.warnings;
            if req.payload.get("tables").is_none() {
                warnings.push("tables missing, imported as empty".to_string());
            }
            Ok(HttpResponse::Ok().json(ApiResp {
                code: 0,
                data: ImportResult {
                    diagram_id: id,
                    imported_tables: stats.tables,
                    imported_fields: stats.fields,
                    warnings,
                    source: req.source.clone(),
                },
                request_id,
            }))
        }
        Err(_) => {
            // 不留下半成品图：持久化失败时回收刚插入的空图行，并如实返回 5xx。
            let _ = db
                .execute(Statement::from_sql_and_values(
                    DatabaseBackend::Sqlite,
                    format!("UPDATE diagram SET is_deleted=1 WHERE id='{}'", esc(&id)),
                    vec![],
                ))
                .await;
            Ok(HttpResponse::InternalServerError().json(ApiErr {
                code: 500,
                message: "import persistence failed".into(),
                request_id,
                details: None,
            }))
        }
    }
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
        // fix-diagram-import-persistence：表必须带 name（缺 name 会被逐表容错丢弃）；
        // 表/字段缺 id 由服务端自动补全并如实持久化。
        let req = test::TestRequest::post().uri("/api/v1/diagrams/import")
            .set_json(serde_json::json!({
                "source": "localStorage",
                "payload": {"name":"import-1", "tables":[{"name":"t1","fields":[{"name":"id"},{"name":"n"}]}]}
            }))
            .to_request();
        let ok: Value = test::call_and_read_body_json(&app, req).await;
        assert_eq!(ok["code"], 0);
        assert_eq!(ok["data"]["imported_tables"], 1);
        assert_eq!(ok["data"]["imported_fields"], 2);
        let imported_id = ok["data"]["diagram_id"].as_str().unwrap().to_string();
        // 数量按实际持久化上报：GET 必须能读到补全 id 后的表与字段
        let req = test::TestRequest::get()
            .uri(&format!("/api/v1/diagrams/{}", imported_id))
            .to_request();
        let loaded: Value = test::call_and_read_body_json(&app, req).await;
        let table = &loaded["data"]["tables"][0];
        assert!(table["id"].as_str().map(|s| s.starts_with("auto-")).unwrap_or(false));
        assert_eq!(table["fields"].as_array().map(Vec::len), Some(2));
        assert!(table["fields"][0]["id"].as_str().map(|s| !s.is_empty()).unwrap_or(false));
        let req = test::TestRequest::post().uri("/api/v1/diagrams/import")
            .set_json(serde_json::json!({"payload": "invalid"}))
            .to_request();
        let bad = test::call_service(&app, req).await;
        assert_eq!(bad.status(), 400);
    }

    /// ST-S01-04（fix-diagram-import-persistence）：payload 无 name、表/字段无 id
    /// → name 兜底 imported_diagram、id 自动补全，GET 结构与契约一致。
    #[actix_web::test]
    async fn st_s01_04_import_missing_ids_and_name() {
        mark_pass("ST-S01-04");
        let db = build_db().await;
        let app = test::init_service(
            App::new().app_data(web::Data::new(db)).service(web::scope("/api/v1").configure(diagrams_v1_routes))
        ).await;
        let req = test::TestRequest::post().uri("/api/v1/diagrams/import")
            .set_json(serde_json::json!({
                "source": "mcp",
                "payload": {"tables":[{"name":"t_no_id","x":0.0,"y":0.0,
                                      "fields":[{"name":"f1","type":"INT"}]}]}
            }))
            .to_request();
        let ok: Value = test::call_and_read_body_json(&app, req).await;
        assert_eq!(ok["code"], 0);
        assert_eq!(ok["data"]["imported_tables"], 1);
        assert_eq!(ok["data"]["imported_fields"], 1);
        let id = ok["data"]["diagram_id"].as_str().unwrap().to_string();
        let req = test::TestRequest::get()
            .uri(&format!("/api/v1/diagrams/{}", id))
            .to_request();
        let loaded: Value = test::call_and_read_body_json(&app, req).await;
        assert_eq!(loaded["data"]["name"], "imported_diagram");
        let table = &loaded["data"]["tables"][0];
        assert!(table["id"].as_str().map(|s| s.starts_with("auto-")).unwrap_or(false));
        assert!(table["fields"][0]["id"].as_str().map(|s| !s.is_empty()).unwrap_or(false));
    }

    /// ST-S01-05（fix-diagram-import-persistence）：坏表（缺 name）单独丢弃并计入
    /// warnings，好表正常持久化；imported_tables 按实际落库数量上报。
    #[actix_web::test]
    async fn st_s01_05_import_bad_table_dropped_with_warning() {
        mark_pass("ST-S01-05");
        let db = build_db().await;
        let app = test::init_service(
            App::new().app_data(web::Data::new(db)).service(web::scope("/api/v1").configure(diagrams_v1_routes))
        ).await;
        let req = test::TestRequest::post().uri("/api/v1/diagrams/import")
            .set_json(serde_json::json!({
                "payload": {"name":"mix", "tables":[
                    {"name":"good","fields":[]},
                    {"fields":[{"name":"orphan"}]}
                ]}
            }))
            .to_request();
        let ok: Value = test::call_and_read_body_json(&app, req).await;
        assert_eq!(ok["code"], 0);
        assert_eq!(ok["data"]["imported_tables"], 1);
        let warnings = ok["data"]["warnings"].as_array().cloned().unwrap_or_default();
        assert!(
            warnings.iter().any(|w| w.as_str().map(|s| s.contains("dropped")).unwrap_or(false)),
            "warnings 必须包含丢弃明细: {warnings:?}"
        );
        let id = ok["data"]["diagram_id"].as_str().unwrap().to_string();
        let req = test::TestRequest::get()
            .uri(&format!("/api/v1/diagrams/{}", id))
            .to_request();
        let loaded: Value = test::call_and_read_body_json(&app, req).await;
        assert_eq!(loaded["data"]["tables"].as_array().map(Vec::len), Some(1));
        assert_eq!(loaded["data"]["tables"][0]["name"], "good");
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

    // ─── diagram-api-auth：鉴权开关 / share_token 豁免 UT/ST ─────────────────

    fn ut_token() -> String {
        crate::auth::sign_access_token("ut-user").unwrap().0
    }

    /// UT-S01-AUTH-01：flag=on 时 POST/PUT/DELETE/import 无 token → 401
    #[actix_web::test]
    async fn ut_s01_auth_01_flag_on_write_requires_token() {
        let db = build_db().await;
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(db))
                .app_data(web::Data::new(DiagramsAuthFlag(true)))
                .service(web::scope("/api/v1").configure(diagrams_v1_routes)),
        )
        .await;

        let resp = test::call_service(&app, test::TestRequest::post().uri("/api/v1/diagrams")
            .set_json(serde_json::json!({"name":"x"})).to_request()).await;
        assert_eq!(resp.status(), 401, "POST /diagrams 无 token 应 401");

        let resp = test::call_service(&app, test::TestRequest::put().uri("/api/v1/diagrams/some-id")
            .set_json(serde_json::json!({"expected_revision":0,"diagram":{"id":"some-id","name":"x"}})).to_request()).await;
        assert_eq!(resp.status(), 401, "PUT /diagrams 无 token 应 401");

        let resp = test::call_service(&app, test::TestRequest::delete().uri("/api/v1/diagrams/some-id").to_request()).await;
        assert_eq!(resp.status(), 401, "DELETE /diagrams 无 token 应 401");

        let resp = test::call_service(&app, test::TestRequest::post().uri("/api/v1/diagrams/import")
            .set_json(serde_json::json!({"payload":{"tables":[]}})).to_request()).await;
        assert_eq!(resp.status(), 401, "POST /diagrams/import 无 token 应 401");

        mark_pass("UT-S01-AUTH-01");
    }

    /// UT-S01-AUTH-02：flag=on 时非法/过期/伪造签名 token → 401；合法 token → 200
    #[actix_web::test]
    async fn ut_s01_auth_02_flag_on_invalid_token_401() {
        let db = build_db().await;
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(db))
                .app_data(web::Data::new(DiagramsAuthFlag(true)))
                .service(web::scope("/api/v1").configure(diagrams_v1_routes)),
        )
        .await;

        for bad in [
            "garbage-token".to_string(),
            crate::auth::sign_access_token_with_ttl("ut-user", -120).unwrap().0, // 已过期（超出 jwt 校验 60s leeway）
            format!("{}x", ut_token()),                                        // 伪造签名
        ] {
            let resp = test::call_service(&app, test::TestRequest::post().uri("/api/v1/diagrams")
                .insert_header(("Authorization", format!("Bearer {bad}")))
                .set_json(serde_json::json!({"name":"x"})).to_request()).await;
            assert_eq!(resp.status(), 401, "非法 token 应 401: {bad}");
        }

        let resp = test::call_service(&app, test::TestRequest::post().uri("/api/v1/diagrams")
            .insert_header(("Authorization", format!("Bearer {}", ut_token())))
            .set_json(serde_json::json!({"name":"ok"})).to_request()).await;
        assert_eq!(resp.status(), 200, "合法 token 应 200（反证）");

        mark_pass("UT-S01-AUTH-02");
    }

    /// UT-S01-AUTH-03：flag=off（默认过渡态）匿名直通，行为同 v1.1
    #[actix_web::test]
    async fn ut_s01_auth_03_flag_off_anonymous_passthrough() {
        let db = build_db().await;
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(db))
                .app_data(web::Data::new(DiagramsAuthFlag(false)))
                .service(web::scope("/api/v1").configure(diagrams_v1_routes)),
        )
        .await;

        let created: Value = test::call_and_read_body_json(&app, test::TestRequest::post().uri("/api/v1/diagrams")
            .set_json(serde_json::json!({"name":"anon-ok"})).to_request()).await;
        assert_eq!(created["code"], 0, "flag=off 匿名创建应 200");
        let id = created["data"]["id"].as_str().unwrap().to_string();

        let got: Value = test::call_and_read_body_json(&app, test::TestRequest::get()
            .uri(&format!("/api/v1/diagrams/{id}")).to_request()).await;
        assert_eq!(got["code"], 0, "flag=off 匿名读应 200（share_token 校验整体跳过）");

        mark_pass("UT-S01-AUTH-03");
    }

    /// UT-S01-AUTH-05：share 端点——off 匿名可调 / on 无 token 401；铸造-轮换-软删 404
    #[actix_web::test]
    async fn ut_s01_auth_05_share_endpoint() {
        let db = build_db().await;
        // flag=off：匿名铸造 + 轮换
        let app_off = test::init_service(
            App::new()
                .app_data(web::Data::new(db.clone()))
                .app_data(web::Data::new(DiagramsAuthFlag(false)))
                .service(web::scope("/api/v1").configure(diagrams_v1_routes)),
        )
        .await;
        let created: Value = test::call_and_read_body_json(&app_off, test::TestRequest::post().uri("/api/v1/diagrams")
            .set_json(serde_json::json!({"name":"share-me"})).to_request()).await;
        let id = created["data"]["id"].as_str().unwrap().to_string();

        let mint: Value = test::call_and_read_body_json(&app_off, test::TestRequest::post()
            .uri(&format!("/api/v1/diagrams/{id}/share")).to_request()).await;
        assert_eq!(mint["code"], 0, "flag=off 匿名铸造 share_token 应 200");
        assert_eq!(mint["data"]["id"], id.as_str());
        let token_a = mint["data"]["share_token"].as_str().unwrap().to_string();
        assert!(!token_a.is_empty());

        let rotate: Value = test::call_and_read_body_json(&app_off, test::TestRequest::post()
            .uri(&format!("/api/v1/diagrams/{id}/share")).to_request()).await;
        let token_b = rotate["data"]["share_token"].as_str().unwrap().to_string();
        assert_ne!(token_a, token_b, "重复调用即轮换");

        // flag=on：无 token → 401
        let app_on = test::init_service(
            App::new()
                .app_data(web::Data::new(db.clone()))
                .app_data(web::Data::new(DiagramsAuthFlag(true)))
                .service(web::scope("/api/v1").configure(diagrams_v1_routes)),
        )
        .await;
        let resp = test::call_service(&app_on, test::TestRequest::post()
            .uri(&format!("/api/v1/diagrams/{id}/share")).to_request()).await;
        assert_eq!(resp.status(), 401, "flag=on 匿名铸造应 401");

        // 软删图 share → 404；不存在 id share → 404
        let req = test::TestRequest::delete().uri(&format!("/api/v1/diagrams/{id}")).to_request();
        assert!(test::call_service(&app_off, req).await.status().is_success());
        let resp = test::call_service(&app_off, test::TestRequest::post()
            .uri(&format!("/api/v1/diagrams/{id}/share")).to_request()).await;
        assert_eq!(resp.status(), 404, "软删图铸造应 404");
        let resp = test::call_service(&app_off, test::TestRequest::post()
            .uri("/api/v1/diagrams/no-such-id/share").to_request()).await;
        assert_eq!(resp.status(), 404, "不存在图铸造应 404");

        mark_pass("UT-S01-AUTH-05");
    }

    /// UT-S02-10：flag=on 时 GET 无 token 且无 share_token → 401
    #[actix_web::test]
    async fn ut_s02_10_flag_on_get_no_token_401() {
        let db = build_db().await;
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(db))
                .app_data(web::Data::new(DiagramsAuthFlag(true)))
                .service(web::scope("/api/v1").configure(diagrams_v1_routes)),
        )
        .await;
        let resp = test::call_service(&app, test::TestRequest::get()
            .uri("/api/v1/diagrams/whatever").to_request()).await;
        assert_eq!(resp.status(), 401);
        mark_pass("UT-S02-10");
    }

    /// UT-S02-11：flag=on 时 GET 凭匹配 share_token → 200 匿名读（S02 豁免）
    #[actix_web::test]
    async fn ut_s02_11_share_token_anonymous_read() {
        let db = build_db().await;
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(db))
                .app_data(web::Data::new(DiagramsAuthFlag(true)))
                .service(web::scope("/api/v1").configure(diagrams_v1_routes)),
        )
        .await;
        let token = ut_token();
        let created: Value = test::call_and_read_body_json(&app, test::TestRequest::post().uri("/api/v1/diagrams")
            .insert_header(("Authorization", format!("Bearer {token}")))
            .set_json(serde_json::json!({"name":"shared"})).to_request()).await;
        let id = created["data"]["id"].as_str().unwrap().to_string();
        let mint: Value = test::call_and_read_body_json(&app, test::TestRequest::post()
            .uri(&format!("/api/v1/diagrams/{id}/share"))
            .insert_header(("Authorization", format!("Bearer {token}"))).to_request()).await;
        let share_token = mint["data"]["share_token"].as_str().unwrap().to_string();

        // 匿名（无 Authorization）凭 share_token 读 → 200
        let got: Value = test::call_and_read_body_json(&app, test::TestRequest::get()
            .uri(&format!("/api/v1/diagrams/{id}?share_token={share_token}")).to_request()).await;
        assert_eq!(got["code"], 0, "share_token 匹配应匿名可读");
        assert_eq!(got["data"]["id"], id.as_str());

        mark_pass("UT-S02-11");
    }

    /// UT-S02-12：flag=on 时 share_token 不匹配 / 图未开分享 → 404（不暴露存在性）
    #[actix_web::test]
    async fn ut_s02_12_share_token_mismatch_404() {
        let db = build_db().await;
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(db))
                .app_data(web::Data::new(DiagramsAuthFlag(true)))
                .service(web::scope("/api/v1").configure(diagrams_v1_routes)),
        )
        .await;
        let token = ut_token();
        let auth = format!("Bearer {token}");
        let created: Value = test::call_and_read_body_json(&app, test::TestRequest::post().uri("/api/v1/diagrams")
            .insert_header(("Authorization", auth.clone()))
            .set_json(serde_json::json!({"name":"unshared"})).to_request()).await;
        let id = created["data"]["id"].as_str().unwrap().to_string();

        // 图未开分享：任意 share_token → 404
        let resp = test::call_service(&app, test::TestRequest::get()
            .uri(&format!("/api/v1/diagrams/{id}?share_token=any-token")).to_request()).await;
        assert_eq!(resp.status(), 404, "未开分享应 404");

        // 开分享后错误 token → 404
        let mint: Value = test::call_and_read_body_json(&app, test::TestRequest::post()
            .uri(&format!("/api/v1/diagrams/{id}/share"))
            .insert_header(("Authorization", auth)).to_request()).await;
        assert_eq!(mint["code"], 0);
        let resp = test::call_service(&app, test::TestRequest::get()
            .uri(&format!("/api/v1/diagrams/{id}?share_token=wrong-token")).to_request()).await;
        assert_eq!(resp.status(), 404, "share_token 不匹配应 404");

        mark_pass("UT-S02-12");
    }

    /// UT-S02-13：轮换后旧 share_token 立即 404；软删图 share 铸造 → 404
    #[actix_web::test]
    async fn ut_s02_13_share_rotate_and_soft_delete() {
        let db = build_db().await;
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(db))
                .app_data(web::Data::new(DiagramsAuthFlag(true)))
                .service(web::scope("/api/v1").configure(diagrams_v1_routes)),
        )
        .await;
        let token = ut_token();
        let auth = format!("Bearer {token}");
        let created: Value = test::call_and_read_body_json(&app, test::TestRequest::post().uri("/api/v1/diagrams")
            .insert_header(("Authorization", auth.clone()))
            .set_json(serde_json::json!({"name":"rotate"})).to_request()).await;
        let id = created["data"]["id"].as_str().unwrap().to_string();

        let mint_a: Value = test::call_and_read_body_json(&app, test::TestRequest::post()
            .uri(&format!("/api/v1/diagrams/{id}/share"))
            .insert_header(("Authorization", auth.clone())).to_request()).await;
        let token_a = mint_a["data"]["share_token"].as_str().unwrap().to_string();
        let mint_b: Value = test::call_and_read_body_json(&app, test::TestRequest::post()
            .uri(&format!("/api/v1/diagrams/{id}/share"))
            .insert_header(("Authorization", auth.clone())).to_request()).await;
        let token_b = mint_b["data"]["share_token"].as_str().unwrap().to_string();
        assert_ne!(token_a, token_b);

        let resp = test::call_service(&app, test::TestRequest::get()
            .uri(&format!("/api/v1/diagrams/{id}?share_token={token_a}")).to_request()).await;
        assert_eq!(resp.status(), 404, "轮换后旧 token 立即失效");
        let got: Value = test::call_and_read_body_json(&app, test::TestRequest::get()
            .uri(&format!("/api/v1/diagrams/{id}?share_token={token_b}")).to_request()).await;
        assert_eq!(got["code"], 0, "新 token 可读");

        // 软删后：share 铸造 404，凭 token 读也 404
        let req = test::TestRequest::delete().uri(&format!("/api/v1/diagrams/{id}"))
            .insert_header(("Authorization", auth.clone())).to_request();
        assert!(test::call_service(&app, req).await.status().is_success());
        let resp = test::call_service(&app, test::TestRequest::post()
            .uri(&format!("/api/v1/diagrams/{id}/share"))
            .insert_header(("Authorization", auth)).to_request()).await;
        assert_eq!(resp.status(), 404, "软删图 share 铸造应 404");
        let resp = test::call_service(&app, test::TestRequest::get()
            .uri(&format!("/api/v1/diagrams/{id}?share_token={token_b}")).to_request()).await;
        assert_eq!(resp.status(), 404, "软删图凭 share_token 读应 404");

        mark_pass("UT-S02-13");
    }

    /// ST-S01-AUTH-01：登录态编辑保存全链路（编排 core-S01 v1.1.0：
    /// 匿名 401 → 登录创建 → PUT 保存 → 过期 revision 409 → 删除 → 404）
    #[actix_web::test]
    async fn st_s01_auth_01_login_crud_orchestration() {
        let db = build_db().await;
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(db))
                .app_data(web::Data::new(DiagramsAuthFlag(true)))
                .service(web::scope("/api/v1").configure(diagrams_v1_routes)),
        )
        .await;
        let token = ut_token();
        let auth = format!("Bearer {token}");

        // 步骤 1：匿名创建 → 401
        let resp = test::call_service(&app, test::TestRequest::post().uri("/api/v1/diagrams")
            .set_json(serde_json::json!({"name":"anon"})).to_request()).await;
        assert_eq!(resp.status(), 401);

        // 步骤 2：登录创建 → 200
        let created: Value = test::call_and_read_body_json(&app, test::TestRequest::post().uri("/api/v1/diagrams")
            .insert_header(("Authorization", auth.clone()))
            .set_json(serde_json::json!({"name":"S01 Auth"})).to_request()).await;
        assert_eq!(created["code"], 0);
        let id = created["data"]["id"].as_str().unwrap().to_string();

        // 步骤 3：PUT 正确 revision → 200，revision 自增
        let saved: Value = test::call_and_read_body_json(&app, test::TestRequest::put()
            .uri(&format!("/api/v1/diagrams/{id}"))
            .insert_header(("Authorization", auth.clone()))
            .set_json(serde_json::json!({"expected_revision":0,"diagram":{"id":id,"name":"S01 Auth v2"}})).to_request()).await;
        assert_eq!(saved["code"], 0);
        assert_eq!(saved["data"]["revision"], 1);

        // 步骤 4：PUT 过期 revision → 409
        let resp = test::call_service(&app, test::TestRequest::put()
            .uri(&format!("/api/v1/diagrams/{id}"))
            .insert_header(("Authorization", auth.clone()))
            .set_json(serde_json::json!({"expected_revision":0,"diagram":{"id":id,"name":"stale"}})).to_request()).await;
        assert_eq!(resp.status(), 409);

        // 步骤 5：GET 登录态读 → 200 revision=1
        let got: Value = test::call_and_read_body_json(&app, test::TestRequest::get()
            .uri(&format!("/api/v1/diagrams/{id}"))
            .insert_header(("Authorization", auth.clone())).to_request()).await;
        assert_eq!(got["data"]["revision"], 1);

        // 步骤 6：DELETE → 200；步骤 7：GET → 404
        let req = test::TestRequest::delete().uri(&format!("/api/v1/diagrams/{id}"))
            .insert_header(("Authorization", auth.clone())).to_request();
        assert!(test::call_service(&app, req).await.status().is_success());
        let resp = test::call_service(&app, test::TestRequest::get()
            .uri(&format!("/api/v1/diagrams/{id}"))
            .insert_header(("Authorization", auth)).to_request()).await;
        assert_eq!(resp.status(), 404);

        mark_pass("ST-S01-AUTH-01");
    }

    /// ST-S02-07：分享匿名读全链路（编排 core-S02 v1.1.0：
    /// 铸造 → 匿名读 → 无 token 401 / 错误 token 404 → 轮换失效 → 清理）
    #[actix_web::test]
    async fn st_s02_07_share_anonymous_read_orchestration() {
        let db = build_db().await;
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(db))
                .app_data(web::Data::new(DiagramsAuthFlag(true)))
                .service(web::scope("/api/v1").configure(diagrams_v1_routes)),
        )
        .await;
        let token = ut_token();
        let auth = format!("Bearer {token}");

        // A 创建（登录态）
        let created: Value = test::call_and_read_body_json(&app, test::TestRequest::post().uri("/api/v1/diagrams")
            .insert_header(("Authorization", auth.clone()))
            .set_json(serde_json::json!({"name":"Shared"})).to_request()).await;
        let id = created["data"]["id"].as_str().unwrap().to_string();

        // A 铸造 share_token
        let mint: Value = test::call_and_read_body_json(&app, test::TestRequest::post()
            .uri(&format!("/api/v1/diagrams/{id}/share"))
            .insert_header(("Authorization", auth.clone())).to_request()).await;
        let share_token = mint["data"]["share_token"].as_str().unwrap().to_string();

        // B 匿名凭 share_token 读 → 200
        let got: Value = test::call_and_read_body_json(&app, test::TestRequest::get()
            .uri(&format!("/api/v1/diagrams/{id}?share_token={share_token}")).to_request()).await;
        assert_eq!(got["code"], 0);
        assert_eq!(got["data"]["name"], "Shared");

        // B 无 token 无 share_token → 401；错误 share_token → 404
        let resp = test::call_service(&app, test::TestRequest::get()
            .uri(&format!("/api/v1/diagrams/{id}")).to_request()).await;
        assert_eq!(resp.status(), 401);
        let resp = test::call_service(&app, test::TestRequest::get()
            .uri(&format!("/api/v1/diagrams/{id}?share_token=wrong")).to_request()).await;
        assert_eq!(resp.status(), 404);

        // A 轮换 → 旧 token 404，新 token 200
        let mint2: Value = test::call_and_read_body_json(&app, test::TestRequest::post()
            .uri(&format!("/api/v1/diagrams/{id}/share"))
            .insert_header(("Authorization", auth.clone())).to_request()).await;
        let share_token2 = mint2["data"]["share_token"].as_str().unwrap().to_string();
        let resp = test::call_service(&app, test::TestRequest::get()
            .uri(&format!("/api/v1/diagrams/{id}?share_token={share_token}")).to_request()).await;
        assert_eq!(resp.status(), 404, "轮换后旧 token 失效");
        let got2: Value = test::call_and_read_body_json(&app, test::TestRequest::get()
            .uri(&format!("/api/v1/diagrams/{id}?share_token={share_token2}")).to_request()).await;
        assert_eq!(got2["code"], 0);

        // A 清理
        let req = test::TestRequest::delete().uri(&format!("/api/v1/diagrams/{id}"))
            .insert_header(("Authorization", auth)).to_request();
        assert!(test::call_service(&app, req).await.status().is_success());

        mark_pass("ST-S02-07");
    }
}
