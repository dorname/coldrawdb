use actix_web::{test, web, App};
use backend::app_config;
use backend::init::setup_test_db_memory;
use serde_json::json;

#[actix_web::test]
async fn test_todos_hello() {
    let db = setup_test_db_memory().await.unwrap();
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(db))
            .configure(app_config),
    )
    .await;

    let req = test::TestRequest::get().uri("/todos/hello").to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
    let body = test::read_body(resp).await;
    assert_eq!(body, "List of todos");
}

#[actix_web::test]
async fn test_todos_query_empty() {
    let db = setup_test_db_memory().await.unwrap();
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(db))
            .configure(app_config),
    )
    .await;

    let req = test::TestRequest::get()
        .uri("/todos/query/some-diagram-id/0")
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
    let body = test::read_body(resp).await;
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["code"], 200);
    assert!(json["data"].is_array());
    assert!(json["data"].as_array().unwrap().is_empty());
}

#[actix_web::test]
async fn test_todos_add_then_query_then_update_then_delete() {
    let db = setup_test_db_memory().await.unwrap();
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(db))
            .configure(app_config),
    )
    .await;

    let diagram_payload = json!({
        "id": "",
        "database": "generic",
        "name": "Diagram for todos",
        "tables": [],
        "areas": [],
        "references": [],
        "notes": [],
        "tasks": []
    });
    let req = test::TestRequest::post()
        .uri("/diagrams/add")
        .set_payload(diagram_payload.to_string())
        .insert_header((actix_web::http::header::CONTENT_TYPE, "application/json"))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
    let body = test::read_body(resp).await;
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    let diagram_id = json["data"]["id"].as_str().unwrap().to_string();

    let todo_payload = json!({
        "diagram_id": diagram_id,
        "title": "Test todo",
        "complete": false,
        "order": 0
    });
    let req = test::TestRequest::post()
        .uri("/todos/add")
        .set_payload(todo_payload.to_string())
        .insert_header((actix_web::http::header::CONTENT_TYPE, "application/json"))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
    let body = test::read_body(resp).await;
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["code"], 200);

    let req = test::TestRequest::get()
        .uri(&format!("/todos/query/{}/0", diagram_id))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
    let body = test::read_body(resp).await;
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["code"], 200);
    let todos = json["data"].as_array().unwrap();
    assert!(!todos.is_empty());
    let task_id = todos[0]["id"].as_str().unwrap().to_string();

    let update_payload = json!({
        "id": task_id,
        "title": "Updated todo",
        "complete": true,
        "order": 1
    });
    let req = test::TestRequest::post()
        .uri("/todos/update")
        .set_payload(update_payload.to_string())
        .insert_header((actix_web::http::header::CONTENT_TYPE, "application/json"))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
    let body = test::read_body(resp).await;
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["code"], 200);

    let req = test::TestRequest::delete()
        .uri(&format!("/todos/delete/{}", task_id))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
    let body = test::read_body(resp).await;
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["code"], 200);
}
