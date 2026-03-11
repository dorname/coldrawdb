use actix_web::{get, post, delete, web, HttpResponse, Responder};
use sea_orm::ActiveModelTrait;
use sea_orm::ColumnTrait;
use sea_orm::DatabaseConnection;
use sea_orm::EntityTrait;
use sea_orm::Iterable;
use sea_orm::QueryFilter;
use sea_orm::QueryOrder;
use sea_orm::QuerySelect;
use sea_orm::TransactionTrait;

use crate::common::CommonResponse;
use crate::common::ResponseCode;
use crate::common::ResponseMessage;
use crate::entity::diagram_link;
use crate::entity::prelude::*;
use crate::entity::task;
use crate::entity::vo::*;
use crate::error::DrawDBError;
use crate::next_id;

pub fn todos_routes(config: &mut web::ServiceConfig) {
    config.route("/test", web::get().to(get_todos_example));
    config.service(hello_todos_example);
    config.service(query_all_todos);
    config.service(add);
    config.service(update);
    config.service(delete);
}

async fn get_todos_example() -> impl Responder {
    HttpResponse::Ok().body("List of todos")
}

#[get("/hello")]
async fn hello_todos_example() -> impl Responder {
    HttpResponse::Ok().body("List of todos")
}

/// 根据diagram_id获取关联的task
/// 参数：diagram_id
/// 返回：所有关联的task
#[get("/query/{diagram_id}/{order_field}")]
async fn query_all_todos(
    db: web::Data<DatabaseConnection>,  
    diagram_id: web::Path<(String, String)>
) -> Result<CommonResponse, DrawDBError> {
    let (diagram_id, order_field) = diagram_id.into_inner();
    let order_column = match order_field.as_str() {
        "1" => task::Column::Complete,
        "2" => task::Column::Title,
        _ => task::Column::Order
    };
    let conn = db.get_ref();
    // select * from task as t
    //inner join diagram_link as link
    //on t.id = link.task_id 
    //where link.diagram_id = ?
    let todos = Task::find()
    .select_only()
    .columns(task::Column::iter())
        .inner_join(DiagramLink)
        .filter(diagram_link::Column::DiagramId.eq(diagram_id))
        .order_by_desc(order_column)
    .all(conn)
        .await?;
    Ok(CommonResponse::new(
        ResponseCode::Success,
        ResponseMessage::Success,
        Some(serde_json::to_value(todos).unwrap()),
    ))
}

/// 新增todo
#[post("/add")]
async fn add(
    db: web::Data<DatabaseConnection>,
    todo: web::Json<TaskAddVo>,
) -> Result<CommonResponse, DrawDBError> {
    // 开启事务
    let tx = db.begin().await?;
    
    // 插入task
    let task_id = next_id();
    let task = todo.convert_to_task(task_id.clone());
    let task_active_model = task::ActiveModel::from(task);
    // 这种写法能看返回最新插入的id，但不会返回整个model
    let task_model = Task::insert(task_active_model).exec(&tx).await?;
    // 插入diagram_link
    let diagram_link_id = next_id();
    let diagram_link = diagram_link::Model::new(
        diagram_link_id, 
        Some(todo.diagram_id.clone()),
        Some(task_id), 
        None,
        None,
        None,
        None
    );
    let diagram_link_active_model = diagram_link::ActiveModel::from(diagram_link);
    DiagramLink::insert(diagram_link_active_model).exec(&tx).await?;
    
    // 提交事务
    tx.commit().await?;
    
    Ok(CommonResponse::new(
        ResponseCode::Success,
        ResponseMessage::Success,
        Some(serde_json::to_value(task_model.last_insert_id).unwrap()),
    ))
}
/// 更新todo
#[post("/update")]
async fn update(
    db: web::Data<DatabaseConnection>,
    todo: web::Json<TaskUpdateVo>,
) -> Result<CommonResponse, DrawDBError> {    // 开始事务
    let tx = db.begin().await?;
    let task = todo.convert_to_active_model();
    // 两种更新的写法返回结果的类型是一样的
    // let task_model = Task::update(task).exec(&tx).await?;
    let task_model = task.update(&tx).await?;
    // 提交事务
    tx.commit().await?;
    Ok(CommonResponse::new(
        ResponseCode::Success,
        ResponseMessage::Success,
        Some(serde_json::to_value(task_model).unwrap()),
    ))
}
/// 删除todo
#[delete("/delete/{id}")]
async fn delete(
    db: web::Data<DatabaseConnection>,
    id: web::Path<String>,
) -> Result<CommonResponse, DrawDBError> {
    let id = id.into_inner();
    // 开启事务
    let tx = db.begin().await?;
    // 删除task
    Task::delete_by_id(id.clone()).exec(&tx).await?;
    // 删除diagram_link
    DiagramLink::delete_many()
    .filter(diagram_link::Column::TaskId.eq(id.clone()))
    .exec(&tx).await?;
    // 提交事务
    tx.commit().await?;
    Ok(CommonResponse::new(
        ResponseCode::Success,
        ResponseMessage::Success,
        Some(serde_json::to_value(id).unwrap()),
    ))
}

#[cfg(test)]
mod test {
    use actix_web::{test, web, App};
    use sea_orm::Database;
    use serde_json::json;

    use super::*;

    #[actix_web::test]
    async fn test_query_all_todos() {
        let db = crate::init::setup_test_db_memory().await.unwrap();
        let db = web::Data::new(db);
        let app = test::init_service(
            App::new()
                .app_data(db.clone())
                .configure(todos_routes)
        ).await;

        let req = test::TestRequest::get()
            .uri("/query/some-diagram-id/1")
            .to_request();

        // 发送请求并获取响应
        let resp = test::call_service(&app, req).await;
        println!("Status: {:?}", resp.status());
        assert!(resp.status().is_success());

        // 解析响应体
        let body = test::read_body(resp).await;
        println!("Response body: {:?}", String::from_utf8(body.to_vec()));
    }

    /// 新增todo（使用内存 DB，先创建 diagram 再 add todo）
    #[actix_web::test]
    async fn test_add_todo() {
        let db = crate::init::setup_test_db_memory().await.unwrap();
        let db = web::Data::new(db);
        let app = test::init_service(
            App::new()
                .app_data(db.clone())
                .configure(crate::app_config)
        ).await;
        let diagram_payload = json!({
            "id": "", "database": "generic", "name": "D",
            "tables": [], "areas": [], "references": [], "notes": [], "tasks": []
        });
        let req = test::TestRequest::post()
            .uri("/diagrams/add")
            .set_json(&diagram_payload)
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());
        let body = test::read_body(resp).await;
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        let diagram_id = json["data"]["id"].as_str().unwrap();
        let req = test::TestRequest::post()
            .uri("/todos/add")
            .set_json(json!({
                "diagram_id": diagram_id,
                "complete": false,
                "order": 0,
                "details": "test",
                "title": "test"
            }))
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());
    }

    /// 更新todo（使用内存 DB，先创建 diagram 和 todo 再更新）
    #[actix_web::test]
    async fn test_update_todo() {
        let db = crate::init::setup_test_db_memory().await.unwrap();
        let db = web::Data::new(db);
        let app = test::init_service(
            App::new()
                .app_data(db.clone())
                .configure(crate::app_config)
        ).await;
        let diagram_payload = json!({
            "id": "", "database": "generic", "name": "D",
            "tables": [], "areas": [], "references": [], "notes": [], "tasks": []
        });
        let req = test::TestRequest::post()
            .uri("/diagrams/add")
            .set_json(&diagram_payload)
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());
        let body = test::read_body(resp).await;
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        let diagram_id = json["data"]["id"].as_str().unwrap().to_string();
        let req = test::TestRequest::post()
            .uri("/todos/add")
            .set_json(json!({ "diagram_id": diagram_id, "title": "t", "complete": false, "order": 0 }))
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());
        let req = test::TestRequest::get()
            .uri(&format!("/todos/query/{}/0", diagram_id))
            .to_request();
        let resp = test::call_service(&app, req).await;
        let body = test::read_body(resp).await;
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        let task_id = json["data"][0]["id"].as_str().unwrap().to_string();
        let req = test::TestRequest::post()
            .uri("/todos/update")
            .set_json(json!({
                "id": task_id,
                "complete": true,
                "order": 1,
                "details": "test66",
                "title": "test1122"
            }))
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());
    }

    /// 删除todo（使用内存 DB，先创建 diagram 和 todo 再删除）
    #[actix_web::test]
    async fn test_delete_todo() {
        let db = crate::init::setup_test_db_memory().await.unwrap();
        let db = web::Data::new(db);
        let app = test::init_service(
            App::new()
                .app_data(db.clone())
                .configure(crate::app_config)
        ).await;
        let diagram_payload = json!({
            "id": "", "database": "generic", "name": "D",
            "tables": [], "areas": [], "references": [], "notes": [], "tasks": []
        });
        let req = test::TestRequest::post()
            .uri("/diagrams/add")
            .set_json(&diagram_payload)
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());
        let body = test::read_body(resp).await;
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        let diagram_id = json["data"]["id"].as_str().unwrap().to_string();
        let req = test::TestRequest::post()
            .uri("/todos/add")
            .set_json(json!({ "diagram_id": diagram_id, "title": "t", "complete": false, "order": 0 }))
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());
        let req = test::TestRequest::get()
            .uri(&format!("/todos/query/{}/0", diagram_id))
            .to_request();
        let resp = test::call_service(&app, req).await;
        let body = test::read_body(resp).await;
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        let task_id = json["data"][0]["id"].as_str().unwrap().to_string();
        let req = test::TestRequest::delete()
            .uri(&format!("/todos/delete/{}", task_id))
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());
    }
}
