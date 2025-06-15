use actix_web::{get, web};
use sea_orm::DatabaseConnection;

use crate::{common::{CommonResponse, ResponseCode, ResponseMessage}, entity::diagram, error::DrawDBError};


pub fn tables_routes(config: &mut web::ServiceConfig){
    
}

/// 查询tables
#[get("/query/{diagram_id}")]
async fn query(
    db: web::Data<DatabaseConnection>,
    diagram_id: web::Path<String>
)->Result<CommonResponse,DrawDBError>{
    let conn  = db.get_ref();

    
    Ok(CommonResponse::new(
        ResponseCode::Success,
        ResponseMessage::Success,
        Some(serde_json::to_value("").unwrap()),
    ))

}