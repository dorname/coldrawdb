use std::collections::HashMap;

use actix_web::{get, web};
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, Iterable, JoinType, QueryFilter, QuerySelect, RelationTrait};
use crate::entity::dto::FieldWithTable;
use crate::entity::prelude::*;
use crate::entity::vo::{FieldVo, TableVo};
use crate::{common::{CommonResponse, ResponseCode, ResponseMessage}, error::DrawDBError};
use crate::entity::{table::Relation as TableRelation,diagram_link,field,table_link};
use itertools::Itertools;
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
    let table_models = Table::find()
    .join(JoinType::InnerJoin, TableRelation::DiagramLink.def())
    .filter(diagram_link::Column::DiagramId.eq(diagram_id))
    .all(conn)
    .await?;
    let table_ids = table_models.iter().map(|table| table.id.clone()).collect::<Vec<String>>();
     
    let fields: Vec<FieldWithTable> = Field::find()
    .select_only()
    // 先把 field 的所有列都选一遍
    .columns(field::Column::iter())
    // 再手动加上 table_link.table_id
    .column(table_link::Column::TableId)
    .join(
        JoinType::InnerJoin,
        field::Relation::TableLink.def(),
    )
    .filter(table_link::Column::TableId.is_in(table_ids.clone()))
    .into_model::<FieldWithTable>()  // 映射到我们的 DTO
    .all(conn)
    .await?;

    let field_map: HashMap<String, Vec<FieldVo>> = fields.into_iter()
    .map(FieldVo::build_from_field_with_table)
    .into_group_map_by(|vo| vo.table_id.clone());

    let table_vos = table_models.iter().map(|table|{
        let binding = vec![];
        let temp_fields = field_map.get(&table.id).unwrap_or(&binding);
        TableVo::build_from_table(table.clone(), temp_fields.to_vec())
    }).collect::<Vec<TableVo>>();
 
    Ok(CommonResponse::new(
        ResponseCode::Success,
        ResponseMessage::Success,
        Some(serde_json::to_value(table_vos).unwrap()),
    ))
}
