use actix_web::{get, web, HttpResponse, Responder};
use sea_orm::ColumnTrait;
use sea_orm::DatabaseConnection;
use sea_orm::EntityTrait;
use sea_orm::Iterable;
use sea_orm::QueryFilter;
use sea_orm::QuerySelect;

use crate::common::CommonResponse;
use crate::common::ResponseCode;
use crate::common::ResponseMessage;
use crate::entity::diagram_link;
use crate::entity::prelude::*;
use crate::entity::task;
use crate::error::DrawDBError;

pub fn todos_routes(config: &mut web::ServiceConfig) {
    config.route("/test", web::get().to(get_todos_example));
    config.service(hello_todos_example);
    config.service(query_all_todos);
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
#[get("/query_all_todos/{diagram_id}")]
async fn query_all_todos(
    db: web::Data<Option<DatabaseConnection>>,
    diagram_id: web::Path<String>,
) -> Result<CommonResponse, DrawDBError> {
    let diagram_id = diagram_id.into_inner();
    if db.is_none() {
        return Err(DrawDBError::OtherError("数据库连接为空".to_string()));
    }
    let conn = db.as_ref().clone().unwrap();
    // select * from task as t
    //inner join diagram_link as link
    //on t.id = link.task_id
    //where link.diagram_id = ?
    let todos = Task::find()
    .select_only()
    .columns(task::Column::iter())
    .inner_join(DiagramLink)
    .filter(diagram_link::Column::DiagramId.eq(diagram_id))
    .all(&conn)
    .await?;
    Ok(CommonResponse::new(
        ResponseCode::Success,
        ResponseMessage::Success,
        Some(serde_json::to_value(todos).unwrap()),
    ))
}

#[cfg(test)]
mod test {
    use actix_web::{test, web, App};
    use sea_orm::Database;

    use super::*;

    #[actix_web::test]
    async fn test_query_all_todos() {
        // 创建测试数据库连接
        let db = Database::connect("sqlite://test.sqlite").await.unwrap();
        let db = web::Data::new(Some(db));
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

    /// 根据diagram_id获取关联的task
    /// 参数：diagram_id
    /// 返回：所有关联的(diagram_id, Vec<Task>)
    #[actix_web::test]
    async fn test_query_all_todos_2() {
        let db = Database::connect("sqlite://test.sqlite").await.unwrap();
        let db = web::Data::new(Some(db));
        async fn query_all_todos(
            db: web::Data<Option<DatabaseConnection>>,
            diagram_id: web::Path<String>,
        ) -> Result<CommonResponse, DrawDBError> {
            let diagram_id = diagram_id.into_inner();
            if db.is_none() {
                return Err(DrawDBError::OtherError("数据库连接为空".to_string()));
            }
            let conn = db.as_ref().clone().unwrap();
            
            // select * from task as t
            //inner join diagram_link as link
            //on t.id = link.task_id
            //inner join diagram as d
            //on d.id = link.diagram_id
            //where d.id = ?
            let todos = Diagram::find_by_id(diagram_id)
            .find_with_related(Task)
            .all(&conn)
            .await?;
            Ok(CommonResponse::new(
                ResponseCode::Success,
                ResponseMessage::Success,
                Some(serde_json::to_value(todos).unwrap()),
            ))
        }
        let app = test::init_service(
            App::new()
                .app_data(db.clone())
                .route("/query_all_todos/{diagram_id}", web::get().to(query_all_todos))
        ).await;
        let req = test::TestRequest::get()
            .uri("/query_all_todos/1")
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());
        let body = test::read_body(resp).await;
        println!("Response body: {:?}", String::from_utf8(body.to_vec()));
    }
}