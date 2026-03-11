use actix_web::{test, web, App};
use backend::app_config;
use backend::init::setup_test_db_memory;

#[actix_web::test]
async fn test_tables_query_tables_empty() {
    let db = setup_test_db_memory().await.unwrap();
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(db))
            .configure(app_config),
    )
    .await;

    let req = test::TestRequest::get()
        .uri("/tables/queryTables/some-diagram-id")
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
    let body = test::read_body(resp).await;
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["code"], 200);
    assert!(json["data"].is_array());
}
