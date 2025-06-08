use actix_web::{get, web, HttpResponse, Responder};
use sea_orm::ColumnTrait;
use sea_orm::DatabaseConnection;
use sea_orm::EntityTrait;
use sea_orm::QueryFilter;

use crate::common::CommonResponse;
use crate::common::ResponseCode;
use crate::common::ResponseMessage;
use crate::entity::diagram_link;
use crate::entity::prelude::*;
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
