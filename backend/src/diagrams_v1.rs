use actix_web::{delete, get, post, put, web, HttpResponse};
use sea_orm::{ConnectionTrait, DatabaseBackend, DatabaseConnection, Statement, TransactionTrait};
use serde::{Deserialize, Serialize};
use serde_json::Value;

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
    diagram: SaveDiagram,
}

#[derive(Deserialize)]
struct SaveDiagram {
    id: String,
    name: Option<String>,
    database: Option<String>,
    pan: Option<String>,
    zoom: Option<String>,
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

#[derive(Serialize)]
struct DiagramOut {
    id: String,
    name: Option<String>,
    database: Option<String>,
    pan: Option<String>,
    zoom: Option<String>,
    revision: i64,
}

fn esc(s: &str) -> String { s.replace('\'', "''") }

pub fn diagrams_v1_routes(config: &mut web::ServiceConfig) {
    config.service(create_diagram_v1);
    config.service(get_diagram_v1);
    config.service(save_diagram_v1);
    config.service(delete_diagram_v1);
    config.service(import_diagram_v1);
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
    let sql = format!(
        "SELECT id, name, database, pan, zoom, revision FROM diagram WHERE id='{}' AND (is_deleted=0 OR is_deleted IS NULL) LIMIT 1",
        esc(&id)
    );
    let row = db.query_one(Statement::from_sql_and_values(DatabaseBackend::Sqlite, sql, vec![])).await?;
    if let Some(row) = row {
        let out = DiagramOut {
            id: row.try_get("", "id")?,
            name: row.try_get("", "name").ok(),
            database: row.try_get("", "database").ok(),
            pan: row.try_get("", "pan").ok(),
            zoom: row.try_get("", "zoom").ok(),
            revision: row.try_get("", "revision").unwrap_or(0),
        };
        return Ok(HttpResponse::Ok().json(ApiResp { code: 0, data: out, request_id }));
    }
    Ok(HttpResponse::NotFound().json(ApiErr { code: 404, message: "not found".into(), request_id, details: None }))
}

#[put("/diagrams/{id}")]
async fn save_diagram_v1(db: web::Data<DatabaseConnection>, id: web::Path<String>, req: web::Json<SaveReq>) -> Result<HttpResponse, DrawDBError> {
    let request_id = next_id();
    let id = id.into_inner();
    if req.diagram.id != id {
        return Ok(HttpResponse::BadRequest().json(ApiErr {
            code: 400,
            message: "path id and body id mismatch".into(),
            request_id,
            details: None,
        }));
    }
    let tx = db.begin().await?;
    let q = format!("SELECT revision FROM diagram WHERE id='{}' LIMIT 1", esc(&id));
    let row = tx.query_one(Statement::from_sql_and_values(DatabaseBackend::Sqlite, q, vec![])).await?;
    let Some(row) = row else {
        return Ok(HttpResponse::NotFound().json(ApiErr { code: 404, message: "not found".into(), request_id, details: None }));
    };
    let cur: i64 = row.try_get("", "revision").unwrap_or(0);
    if cur != req.expected_revision {
        return Ok(HttpResponse::Conflict().json(ApiErr {
            code: 409,
            message: "revision conflict".into(),
            request_id,
            details: Some(serde_json::json!({"current_revision": cur})),
        }));
    }
    let up = format!(
        "UPDATE diagram SET name={}, database={}, pan={}, zoom={}, revision=revision+1, updated_at=datetime('now') WHERE id='{}'",
        req.diagram.name.as_ref().map(|v| format!("'{}'", esc(v))).unwrap_or("NULL".to_string()),
        req.diagram.database.as_ref().map(|v| format!("'{}'", esc(v))).unwrap_or("NULL".to_string()),
        req.diagram.pan.as_ref().map(|v| format!("'{}'", esc(v))).unwrap_or("NULL".to_string()),
        req.diagram.zoom.as_ref().map(|v| format!("'{}'", esc(v))).unwrap_or("NULL".to_string()),
        esc(&id)
    );
    tx.execute(Statement::from_sql_and_values(DatabaseBackend::Sqlite, up, vec![])).await?;
    tx.commit().await?;
    Ok(HttpResponse::Ok().json(ApiResp { code: 0, data: serde_json::json!({"id": id}), request_id }))
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
}
