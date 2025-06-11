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
    config.service(add_todo);
    config.service(update_todo);
    config.service(delete_todo);
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
#[get("/query_all_todos/{diagram_id}/{order_field}")]
async fn query_all_todos(
    db: web::Data<DatabaseConnection>,  
    diagram_id: web::Path<String>,
    order_field: web::Path<String>,
) -> Result<CommonResponse, DrawDBError> {
    let diagram_id = diagram_id.into_inner();
    let order_field = order_field.into_inner();
    let order_column = match order_field.as_str() {
        _ => task::Column::Order,
        "1" => task::Column::Complete,
        "2" => task::Column::Title
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
#[post("/add_todo")]
async fn add_todo(
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
#[post("/update_todo")]
async fn update_todo(
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
#[delete("/delete_todo/{id}")]
async fn delete_todo(
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
        // 创建测试数据库连接
        let db = Database::connect("sqlite://test.sqlite").await.unwrap();
        let db = web::Data::new(db);
        // 创建测试应用
        let app = test::init_service(
            App::new()
                .app_data(db.clone())
                .configure(todos_routes)
        ).await;

        // 创建测试请求
        let req = test::TestRequest::get()
            .uri("/query_all_todos/1")
            .to_request();

        // 发送请求并获取响应
        let resp = test::call_service(&app, req).await;
        println!("Status: {:?}", resp.status());
        assert!(resp.status().is_success());

        // 解析响应体
        let body = test::read_body(resp).await;
        println!("Response body: {:?}", String::from_utf8(body.to_vec()));
    }

    /// 新增todo
    #[actix_web::test]
    async fn test_add_todo() {
        let db = Database::connect("sqlite://test.sqlite").await.unwrap();
        let db = web::Data::new(db);
        let app = test::init_service(
            App::new()
                .app_data(db.clone())
                .configure(todos_routes)
        ).await;
        let req = test::TestRequest::post()
            .uri("/add_todo")
            .set_json(json!({
                "diagram_id": "1",
                "complete": false,
                "order": 0,
                "details": "test",
                "title": "test"
            }))
            .to_request();
        let resp = test::call_service(&app, req).await;
        println!("Status: {:?}", resp.status());
        assert!(resp.status().is_success());
        let body = test::read_body(resp).await;
        println!("Response body: {:?}", String::from_utf8(body.to_vec()));
    }

    /// 更新todo
    #[actix_web::test]
    async fn test_update_todo() {
        let db = Database::connect("sqlite://test.sqlite").await.unwrap();
        let db = web::Data::new(db);
        let app = test::init_service(
            App::new()
                .app_data(db.clone())
                .configure(todos_routes)
        ).await;
        let req = test::TestRequest::post()
            .uri("/update_todo")
            .set_json(json!({
                "id": "7338216606830563329",
                "complete": true,
                "order": 1,
                "details": "test66",
                "title": "test1122" 
            }))
            .to_request();
        let resp = test::call_service(&app, req).await;
        println!("Status: {:?}", resp.status());
        assert!(resp.status().is_success());
        let body = test::read_body(resp).await; 
        println!("Response body: {:?}", String::from_utf8(body.to_vec()));
    }

    /// 删除todo
    #[actix_web::test]
    async fn test_delete_todo() {
        let db = Database::connect("sqlite://test.sqlite").await.unwrap();
        let db = web::Data::new(db);
        let app = test::init_service(
            App::new()
                .app_data(db.clone())
                .configure(todos_routes)
        ).await;
        let req = test::TestRequest::delete()
            .uri("/delete_todo/7338216606830563329")
            .to_request();
        let resp = test::call_service(&app, req).await;
        println!("Status: {:?}", resp.status());
        assert!(resp.status().is_success());
        let body = test::read_body(resp).await;
        println!("Response body: {:?}", String::from_utf8(body.to_vec()));
    }
}
