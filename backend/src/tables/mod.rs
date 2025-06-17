use actix_web::{get, web};
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};
use crate::entity::prelude::*;
use crate::{common::{CommonResponse, ResponseCode, ResponseMessage}, entity::diagram, error::DrawDBError};


pub fn tables_routes(config: &mut web::ServiceConfig){
    config.service(query_tables);
}

/// 查询与table关联的field
#[get("/queryTables/{diagram_id}")]
async fn query_tables(
    db: web::Data<DatabaseConnection>,
    diagram_id: web::Path<String>
) -> Result<CommonResponse, DrawDBError> {
    let conn = db.get_ref();
    let diagram_id = diagram_id.into_inner();
    let field_model = Table::find()
    .find_with_related(Field)
    .filter(diagram::Column::Id.eq(diagram_id))
    .all(conn)
    .await?;
    Ok(CommonResponse::new(
        ResponseCode::Success,
        ResponseMessage::Success,
        Some(serde_json::to_value(field_model).unwrap()),
    ))
}
