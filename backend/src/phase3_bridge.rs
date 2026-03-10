use actix_web::{get, post, put, web, HttpResponse};
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

#[derive(Serialize, Deserialize)]
struct BridgeConfig {
    db_read_preferred: bool,
    db_write_enabled: bool,
    dual_write_local: bool,
    updated_at: String,
}

#[derive(Deserialize)]
struct UpdateBridgeConfigReq {
    db_read_preferred: Option<bool>,
    db_write_enabled: Option<bool>,
    dual_write_local: Option<bool>,
}

#[derive(Deserialize)]
struct ImportLocalDraftReq {
    source: Option<String>,
    payload: Value,
}

fn esc(s: &str) -> String {
    s.replace('\'', "''")
}

pub fn phase3_bridge_routes(config: &mut web::ServiceConfig) {
    config.service(get_bridge_config);
    config.service(update_bridge_config);
    config.service(import_local_draft);
    config.service(query_import_logs);
    config.service(retry_import_log);
}

#[derive(Deserialize)]
struct QueryImportLogReq {
    status: Option<String>,
}

#[get("/bridge/import/local/logs")]
async fn query_import_logs(
    db: web::Data<DatabaseConnection>,
    query: web::Query<QueryImportLogReq>,
) -> Result<HttpResponse, DrawDBError> {
    let request_id = next_id();
    let mut sql = "SELECT id, source, imported_diagram_id, status, retry_count, error_message, created_at, updated_at FROM local_draft_import_log".to_string();
    if let Some(status) = &query.status {
        sql.push_str(&format!(" WHERE status='{}'", esc(status)));
    }
    sql.push_str(" ORDER BY created_at DESC LIMIT 100");

    let rows = db
        .query_all(Statement::from_sql_and_values(
            DatabaseBackend::Sqlite,
            sql,
            vec![],
        ))
        .await?;

    let data: Vec<Value> = rows
        .iter()
        .map(|row| {
            serde_json::json!({
                "id": row.try_get::<String>("", "id").unwrap_or_default(),
                "source": row.try_get::<String>("", "source").ok(),
                "imported_diagram_id": row.try_get::<String>("", "imported_diagram_id").ok(),
                "status": row.try_get::<String>("", "status").unwrap_or_default(),
                "retry_count": row.try_get::<i64>("", "retry_count").unwrap_or(0),
                "error_message": row.try_get::<String>("", "error_message").ok(),
                "created_at": row.try_get::<String>("", "created_at").ok(),
                "updated_at": row.try_get::<String>("", "updated_at").ok(),
            })
        })
        .collect();

    Ok(HttpResponse::Ok().json(ApiResp {
        code: 0,
        data,
        request_id,
    }))
}

#[post("/bridge/import/local/retry/{id}")]
async fn retry_import_log(
    db: web::Data<DatabaseConnection>,
    id: web::Path<String>,
) -> Result<HttpResponse, DrawDBError> {
    let request_id = next_id();
    let id = id.into_inner();

    let q = format!(
        "SELECT payload, source, retry_count FROM local_draft_import_log WHERE id='{}' LIMIT 1",
        esc(&id)
    );
    let row = db
        .query_one(Statement::from_sql_and_values(DatabaseBackend::Sqlite, q, vec![]))
        .await?;

    let Some(row) = row else {
        return Ok(HttpResponse::NotFound().json(ApiErr {
            code: 404,
            message: "import log not found".into(),
            request_id,
            details: None,
        }));
    };

    let payload_text: String = row.try_get("", "payload").unwrap_or_else(|_| "{}".to_string());
    let source: String = row.try_get("", "source").unwrap_or_else(|_| "localStorage".to_string());
    let retry_count: i64 = row.try_get("", "retry_count").unwrap_or(0);

    let parsed: Result<Value, _> = serde_json::from_str(&payload_text);
    let Ok(payload) = parsed else {
        let fail = format!(
            "UPDATE local_draft_import_log SET status='failed', retry_count={}, error_message='invalid payload json', updated_at=datetime('now') WHERE id='{}'",
            retry_count + 1,
            esc(&id)
        );
        db.execute(Statement::from_sql_and_values(DatabaseBackend::Sqlite, fail, vec![]))
            .await?;
        return Ok(HttpResponse::BadRequest().json(ApiErr {
            code: 400,
            message: "invalid payload json".into(),
            request_id,
            details: None,
        }));
    };

    if !payload.is_object() {
        let fail = format!(
            "UPDATE local_draft_import_log SET status='failed', retry_count={}, error_message='payload must be object', updated_at=datetime('now') WHERE id='{}'",
            retry_count + 1,
            esc(&id)
        );
        db.execute(Statement::from_sql_and_values(DatabaseBackend::Sqlite, fail, vec![]))
            .await?;
        return Ok(HttpResponse::BadRequest().json(ApiErr {
            code: 400,
            message: "payload must be object".into(),
            request_id,
            details: None,
        }));
    }

    let diagram_id = next_id();
    let name = payload
        .get("name")
        .and_then(|v| v.as_str())
        .unwrap_or("local_draft_retry");

    let tx = db.begin().await?;
    let insert_diagram = format!(
        "INSERT INTO diagram(id, name, database, pan, zoom, revision, updated_at, is_deleted) VALUES('{}','{}',NULL,'','',0,datetime('now'),0)",
        esc(&diagram_id),
        esc(name)
    );
    tx.execute(Statement::from_sql_and_values(
        DatabaseBackend::Sqlite,
        insert_diagram,
        vec![],
    ))
    .await?;

    let ok_sql = format!(
        "UPDATE local_draft_import_log SET status='success', retry_count={}, imported_diagram_id='{}', source='{}', error_message=NULL, updated_at=datetime('now') WHERE id='{}'",
        retry_count + 1,
        esc(&diagram_id),
        esc(&source),
        esc(&id)
    );
    tx.execute(Statement::from_sql_and_values(DatabaseBackend::Sqlite, ok_sql, vec![]))
        .await?;
    tx.commit().await?;

    Ok(HttpResponse::Ok().json(ApiResp {
        code: 0,
        data: serde_json::json!({"id": id, "status": "success", "retry_count": retry_count + 1, "diagram_id": diagram_id}),
        request_id,
    }))
}

#[get("/bridge/config")]
async fn get_bridge_config(db: web::Data<DatabaseConnection>) -> Result<HttpResponse, DrawDBError> {
    let request_id = next_id();
    let sql = "SELECT db_read_preferred, db_write_enabled, dual_write_local, updated_at FROM bridge_config WHERE id=1 LIMIT 1";
    let row = db
        .query_one(Statement::from_sql_and_values(DatabaseBackend::Sqlite, sql, vec![]))
        .await?;

    if let Some(row) = row {
        let data = BridgeConfig {
            db_read_preferred: row.try_get("", "db_read_preferred").unwrap_or(false),
            db_write_enabled: row.try_get("", "db_write_enabled").unwrap_or(false),
            dual_write_local: row.try_get("", "dual_write_local").unwrap_or(false),
            updated_at: row
                .try_get::<String>("", "updated_at")
                .unwrap_or_else(|_| "".to_string()),
        };
        return Ok(HttpResponse::Ok().json(ApiResp {
            code: 0,
            data,
            request_id,
        }));
    }

    Ok(HttpResponse::NotFound().json(ApiErr {
        code: 404,
        message: "bridge config not found".into(),
        request_id,
        details: None,
    }))
}

#[put("/bridge/config")]
async fn update_bridge_config(
    db: web::Data<DatabaseConnection>,
    req: web::Json<UpdateBridgeConfigReq>,
) -> Result<HttpResponse, DrawDBError> {
    let request_id = next_id();
    let tx = db.begin().await?;

    let read = req
        .db_read_preferred
        .map(|v| if v { 1 } else { 0 })
        .map(|v| v.to_string())
        .unwrap_or("db_read_preferred".to_string());
    let write = req
        .db_write_enabled
        .map(|v| if v { 1 } else { 0 })
        .map(|v| v.to_string())
        .unwrap_or("db_write_enabled".to_string());
    let dual = req
        .dual_write_local
        .map(|v| if v { 1 } else { 0 })
        .map(|v| v.to_string())
        .unwrap_or("dual_write_local".to_string());

    let up = format!(
        "UPDATE bridge_config SET db_read_preferred={}, db_write_enabled={}, dual_write_local={}, updated_at=datetime('now') WHERE id=1",
        read, write, dual
    );

    tx.execute(Statement::from_sql_and_values(DatabaseBackend::Sqlite, up, vec![]))
        .await?;
    tx.commit().await?;

    Ok(HttpResponse::Ok().json(ApiResp {
        code: 0,
        data: serde_json::json!({"updated": true}),
        request_id,
    }))
}

#[post("/bridge/import/local")]
async fn import_local_draft(
    db: web::Data<DatabaseConnection>,
    req: web::Json<ImportLocalDraftReq>,
) -> Result<HttpResponse, DrawDBError> {
    let request_id = next_id();
    if !req.payload.is_object() {
        return Ok(HttpResponse::BadRequest().json(ApiErr {
            code: 400,
            message: "payload must be object".into(),
            request_id,
            details: Some(serde_json::json!({"field":"payload"})),
        }));
    }

    let log_id = next_id();
    let diagram_id = next_id();
    let name = req
        .payload
        .get("name")
        .and_then(|v| v.as_str())
        .unwrap_or("local_draft");

    let tx = db.begin().await?;
    let insert_diagram = format!(
        "INSERT INTO diagram(id, name, database, pan, zoom, revision, updated_at, is_deleted) VALUES('{}','{}',NULL,'','',0,datetime('now'),0)",
        esc(&diagram_id),
        esc(name)
    );
    tx.execute(Statement::from_sql_and_values(
        DatabaseBackend::Sqlite,
        insert_diagram,
        vec![],
    ))
    .await?;

    let payload_text = esc(&req.payload.to_string());
    let source = req.source.clone().unwrap_or_else(|| "localStorage".to_string());
    let insert_log = format!(
        "INSERT INTO local_draft_import_log(id, source, payload, imported_diagram_id, status, retry_count, error_message, updated_at) VALUES('{}','{}','{}','{}','success',0,NULL,datetime('now'))",
        esc(&log_id),
        esc(&source),
        payload_text,
        esc(&diagram_id)
    );

    tx.execute(Statement::from_sql_and_values(
        DatabaseBackend::Sqlite,
        insert_log,
        vec![],
    ))
    .await?;
    tx.commit().await?;

    Ok(HttpResponse::Ok().json(ApiResp {
        code: 0,
        data: serde_json::json!({
            "log_id": log_id,
            "diagram_id": diagram_id,
            "status": "success"
        }),
        request_id,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::{test, App};
    use sea_orm::Database;

    use crate::init::{apply_migrations, init_table};

    async fn build_db() -> DatabaseConnection {
        let db_path = format!(
            "{}/drawdb_phase3_bridge_{}.sqlite",
            std::env::temp_dir().display(),
            std::process::id()
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

    #[actix_web::test]
    async fn test_bridge_config_update_and_import_local() {
        let db = build_db().await;
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(db))
                .service(web::scope("/api/v1").configure(phase3_bridge_routes)),
        )
        .await;

        let req = test::TestRequest::get()
            .uri("/api/v1/bridge/config")
            .to_request();
        let cfg: Value = test::call_and_read_body_json(&app, req).await;
        assert_eq!(cfg["code"], 0);

        let req = test::TestRequest::put()
            .uri("/api/v1/bridge/config")
            .set_json(serde_json::json!({"dual_write_local": true}))
            .to_request();
        let up = test::call_service(&app, req).await;
        assert!(up.status().is_success());

        let req = test::TestRequest::post()
            .uri("/api/v1/bridge/import/local")
            .set_json(serde_json::json!({"source":"localStorage","payload":{"name":"d-bridge"}}))
            .to_request();
        let ok: Value = test::call_and_read_body_json(&app, req).await;
        assert_eq!(ok["code"], 0);

        let req = test::TestRequest::post()
            .uri("/api/v1/bridge/import/local")
            .set_json(serde_json::json!({"payload":"invalid"}))
            .to_request();
        let bad = test::call_service(&app, req).await;
        assert_eq!(bad.status(), 400);

        let req = test::TestRequest::get()
            .uri("/api/v1/bridge/import/local/logs")
            .to_request();
        let logs: Value = test::call_and_read_body_json(&app, req).await;
        assert_eq!(logs["code"], 0);
        assert!(logs["data"].as_array().unwrap().len() >= 1);

        let id = logs["data"][0]["id"].as_str().unwrap().to_string();
        let req = test::TestRequest::post()
            .uri(&format!("/api/v1/bridge/import/local/retry/{}", id))
            .to_request();
        let retried: Value = test::call_and_read_body_json(&app, req).await;
        assert_eq!(retried["code"], 0);
        assert_eq!(retried["data"]["status"], "success");
    }
}
