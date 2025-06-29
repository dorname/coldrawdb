mod internal_api;
pub use internal_api::*;
use actix_web::{get, post, web};
use sea_orm::{DatabaseConnection, EntityTrait, TransactionTrait};
use crate::entity::{prelude::*, table};
use crate::entity::vo::{TableVo, build_table_link};
use crate::{next_id, next_ids};
use crate::{common::{CommonResponse, ResponseCode, ResponseMessage}, error::DrawDBError};
use crate::entity::{field,table_link};
pub fn tables_routes(config: &mut web::ServiceConfig){
    config.service(query);
}

/// 查询与table关联的field
#[get("/queryTables/{diagram_id}")]
async fn query(
    db: web::Data<DatabaseConnection>,
    diagram_id: web::Path<String>
) -> Result<CommonResponse, DrawDBError> {
    let diagram_id = diagram_id.into_inner();
    let result = query_tables(db,diagram_id).await?;
    Ok(CommonResponse::new(
        ResponseCode::Success,
        ResponseMessage::Success,
        Some(serde_json::to_value(result).unwrap()),
    ))
}

/// 新增tables
#[post("/add")]
async fn add(
   db: web::Data<DatabaseConnection>,
   table_vo: web::Json<TableVo>
)->Result<CommonResponse, DrawDBError> {
    //1、开启事务
    let tx = db.begin().await?;
    //2、新增图表
    let table_vo = table_vo.into_inner();
    let table_id =  add_table(&tx,table_vo).await?;
    //3、提交事务
    tx.commit().await?;
    Ok(CommonResponse::new(
        ResponseCode::Success,
        ResponseMessage::Success,
        Some(serde_json::to_value(table_id).unwrap()),
    ))
}
