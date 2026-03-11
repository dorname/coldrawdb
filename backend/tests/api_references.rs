use actix_web::{test, web, App};
use backend::app_config;
use backend::init::setup_test_db_memory;
use serde_json::json;

#[actix_web::test]
async fn test_references_add_one_then_delete() {
    let db = setup_test_db_memory().await.unwrap();
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(db))
            .configure(app_config),
    )
    .await;

    let payload = json!([{
        "id": "ref-test-1",
        "name": null,
        "start_table_id": null,
        "end_table_id": null,
        "cardinality": null,
        "delete_constraint": null,
        "end_field_id": null,
        "start_field_id": null,
        "update_constraint": null
    }]);
    let req = test::TestRequest::post()
        .uri("/references/add")
        .set_payload(payload.to_string())
        .insert_header((actix_web::http::header::CONTENT_TYPE, "application/json"))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success(), "references add failed: {:?}", resp.status());
    let body = test::read_body(resp).await;
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["code"], 200);

    let del_payload = json!([{ "id": "ref-test-1", "name": null, "start_table_id": null, "end_table_id": null, "cardinality": null, "delete_constraint": null, "end_field_id": null, "start_field_id": null, "update_constraint": null }]);
    let req = test::TestRequest::post()
        .uri("/references/delete")
        .set_payload(del_payload.to_string())
        .insert_header((actix_web::http::header::CONTENT_TYPE, "application/json"))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
    let body = test::read_body(resp).await;
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["code"], 200);
}

#[actix_web::test]
async fn test_references_delete_empty() {
    let db = setup_test_db_memory().await.unwrap();
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(db))
            .configure(app_config),
    )
    .await;

    let payload = json!([]);
    let req = test::TestRequest::post()
        .uri("/references/delete")
        .set_payload(payload.to_string())
        .insert_header((actix_web::http::header::CONTENT_TYPE, "application/json"))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
    let body = test::read_body(resp).await;
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["code"], 200);
}
