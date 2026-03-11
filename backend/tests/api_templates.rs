use actix_web::{test, web, App};
use backend::app_config;
use backend::init::setup_test_db_memory;
use serde_json::json;

#[actix_web::test]
async fn test_templates_query_all() {
    let db = setup_test_db_memory().await.unwrap();
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(db))
            .configure(app_config),
    )
    .await;

    let req = test::TestRequest::get().uri("/templates/queryAll").to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
    let body = test::read_body(resp).await;
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["code"], 200);
    assert!(json["data"].is_array());
}

#[actix_web::test]
async fn test_templates_query_not_found() {
    let db = setup_test_db_memory().await.unwrap();
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(db))
            .configure(app_config),
    )
    .await;

    let req = test::TestRequest::get()
        .uri("/templates/query/nonexistent-id")
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
    let body = test::read_body(resp).await;
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["code"], 404);
}

#[actix_web::test]
async fn test_templates_add() {
    let db = setup_test_db_memory().await.unwrap();
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(db))
            .configure(app_config),
    )
    .await;

    let payload = json!({
        "id": "",
        "title": "Test Template",
        "database": "generic",
        "custom": 1,
        "tables": [],
        "relationships": [],
        "notes": [],
        "subjectAreas": [],
        "todos": []
    });
    let req = test::TestRequest::post()
        .uri("/templates/add")
        .set_payload(payload.to_string())
        .insert_header((actix_web::http::header::CONTENT_TYPE, "application/json"))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
    let body = test::read_body(resp).await;
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["code"], 200);
    let id = json["data"]["id"].as_str().unwrap();
    assert!(!id.is_empty());
}

#[actix_web::test]
async fn test_templates_add_then_query_update_delete() {
    let db = setup_test_db_memory().await.unwrap();
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(db))
            .configure(app_config),
    )
    .await;

    let payload = json!({
        "id": "",
        "title": "Template to update",
        "database": "generic",
        "custom": 1,
        "tables": [],
        "relationships": [],
        "notes": [],
        "subjectAreas": [],
        "todos": []
    });
    let req = test::TestRequest::post()
        .uri("/templates/add")
        .set_payload(payload.to_string())
        .insert_header((actix_web::http::header::CONTENT_TYPE, "application/json"))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
    let body = test::read_body(resp).await;
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    let id = json["data"]["id"].as_str().unwrap().to_string();

    let req = test::TestRequest::get()
        .uri(&format!("/templates/query/{}", id))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
    let body = test::read_body(resp).await;
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["code"], 200);
    assert_eq!(json["data"]["title"], "Template to update");

    let update_payload = json!({
        "id": id,
        "title": "Updated template",
        "database": "generic",
        "custom": 1,
        "tables": [],
        "relationships": [],
        "notes": [],
        "subjectAreas": [],
        "todos": []
    });
    let req = test::TestRequest::post()
        .uri("/templates/update")
        .set_payload(update_payload.to_string())
        .insert_header((actix_web::http::header::CONTENT_TYPE, "application/json"))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
    let body = test::read_body(resp).await;
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["code"], 200);

    let req = test::TestRequest::delete()
        .uri(&format!("/templates/delete/{}", id))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
    let body = test::read_body(resp).await;
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["code"], 200);
}
