use actix_web::{delete, post};
use actix_web::{get, web};
use sea_orm::{ActiveModelTrait, DatabaseConnection, TransactionTrait};
use sea_orm::EntityTrait;
use crate::common::ResponseCode;
use crate::common::ResponseMessage;
use crate::entity::diagram::{self, ActiveModel};
use crate::entity::prelude::*;
use crate::entity::vo::DiagramVo;
use crate::next_id;
use crate::{common::CommonResponse, error::DrawDBError};

/// 图表模块
pub fn diagrams_routes(config: &mut web::ServiceConfig) {
    config.service(query_all_diagrams);
    config.service(add_diagram);
    config.service(update_diagram);
    config.service(delete_diagram);
}

/// 查询所有图表
#[get("/queryAll")]
async fn query_all_diagrams(
    db: web::Data<DatabaseConnection>,
) -> Result<CommonResponse, DrawDBError> {
    let conn = db.get_ref();
    let diagrams = Diagram::find().all(conn).await?;
    Ok(CommonResponse::new(
        ResponseCode::Success,
        ResponseMessage::Success,
        Some(serde_json::to_value(diagrams).unwrap()),
    ))
}

/// 查询图表
#[get("/query/{id}")]
async fn query_diagram(
    db: web::Data<DatabaseConnection>,
    id: web::Path<String>
) -> Result<CommonResponse, DrawDBError> {
    let conn = db.get_ref();
    let id = id.into_inner();
    let diagram = Diagram::find_by_id(id).one(conn).await?;
    Ok(CommonResponse::new(
        ResponseCode::Success,
        ResponseMessage::Success,
        Some(serde_json::to_value(diagram).unwrap()),
    ))
}

/// 新增图表
#[post("/add")]
async fn add_diagram(
    db: web::Data<DatabaseConnection>,
    diagram: web::Json<DiagramVo>
) -> Result<CommonResponse, DrawDBError> {
    // 开始事务
    let tx = db.begin().await?;
    let id = next_id();
    let diagram_model = diagram.into_inner().convert_to_diagram(id);
    // 新增图表
    let active_model = ActiveModel::from(diagram_model);
    let result = active_model.insert(&tx).await?;
    // 新增图表与表的关联关系

    // 提交事务
    tx.commit().await?;
    Ok(CommonResponse::new(ResponseCode::Success,
         ResponseMessage::Success,
          Some(serde_json::to_value(result).unwrap())))
}

///更新图表
#[post("/update")]
async fn update_diagram(
    db: web::Data<DatabaseConnection>,
    diagram: web::Json<DiagramVo>
) -> Result<CommonResponse, DrawDBError>{
    //开启事务
    let tx = db.begin().await?;
    let diagram_model = diagram.convert_to_active_model();
    let result = diagram_model.update(&tx).await?;
    // TODO：
    // 1、删除与表的关联关系
    // 2、删除与引用的关联关系
    // 3、重新构建与表的关联关系
    // 4、重新构建与引用的关联关系
    // 5、更新图表
    // 6、更新引用
    tx.commit().await?;
    Ok(CommonResponse::new(ResponseCode::Success,
        ResponseMessage::Success,
         Some(serde_json::to_value(result).unwrap())))
}

///删除图表
#[delete("/detele/{id}")]
async fn delete_diagram(
    db: web::Data<DatabaseConnection>,
    id: web::Path<String>
)->Result<CommonResponse, DrawDBError>{
    let tx = db.begin().await?;
    let id = id.into_inner();
    Diagram::delete_by_id(&id).exec(&tx).await?;
    tx.commit().await?;
    Ok(CommonResponse::new(ResponseCode::Success,
        ResponseMessage::Success,
         Some(serde_json::to_value(id).unwrap())))
}



