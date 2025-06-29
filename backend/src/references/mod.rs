
mod internal_api;
use actix_web::{post, web};
pub use internal_api::*;
use sea_orm::{DatabaseConnection, TransactionTrait};

use crate::{common::{CommonResponse, ResponseCode, ResponseMessage}, entity::vo::ReferenceVo, error::DrawDBError};

/// 新增引用
#[post("/add")]
async fn add_reference(
    db: web::Data<DatabaseConnection>,
    reference: web::Json<Vec<ReferenceVo>>
) -> Result<CommonResponse, DrawDBError> {
    let tx = db.begin().await?;
    let result = add_references(&tx, reference.into_inner()).await?;
    tx.commit().await?;
    Ok(CommonResponse::new(ResponseCode::Success,
        ResponseMessage::Success,
         Some(serde_json::to_value(result).unwrap())))
}

/// 删除引用
#[post("/delete")]
async fn delete_reference(
    db: web::Data<DatabaseConnection>,
    reference: web::Json<Vec<ReferenceVo>>
) -> Result<CommonResponse, DrawDBError> {
    let tx = db.begin().await?;
    let result = delete_references(&tx, reference.into_inner()).await?;
    tx.commit().await?;
    Ok(CommonResponse::new(ResponseCode::Success,
        ResponseMessage::Success,
         Some(serde_json::to_value(result).unwrap())))
}

