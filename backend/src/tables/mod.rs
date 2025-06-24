use std::collections::HashMap;

use actix_web::middleware::Next;
use actix_web::web::Json;
use actix_web::{get, post, web};
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, Iterable, JoinType, ModelTrait, QueryFilter, QuerySelect, RelationTrait, TransactionTrait};
use tracing_subscriber::registry::Data;
use crate::entity::area::ActiveModel;
use crate::entity::dto::FieldWithTable;
use crate::entity::{prelude::*, table};
use crate::entity::vo::{FieldVo, TableVo, build_table_link};
use crate::{next_id, next_ids};
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
        TableVo::build_from_table(table.clone(), Some(temp_fields.to_vec()))
    }).collect::<Vec<TableVo>>();
 
    Ok(CommonResponse::new(
        ResponseCode::Success,
        ResponseMessage::Success,
        Some(serde_json::to_value(table_vos).unwrap()),
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
    let  table_id = next_id();
    let temp_table_vo = table_vo.into_inner();
    let mut table_add = temp_table_vo.convert_to_table();
    table_add.id = table_id.clone(); 
    let table_am = table::ActiveModel::from(table_add);
    let insert_rsult = Table::insert(table_am).exec(&tx).await?;
    //2、TODO 插入默认的field
    let field_size = temp_table_vo.fields.clone().unwrap_or(vec![]).len();
    let field_ids = next_ids(field_size);
    let mut fields = temp_table_vo.fields.clone().unwrap_or(vec![]).iter().map( |field|{
         field.convert_to_field()
    }).collect::<Vec<field::Model>>();
    fields.iter_mut().enumerate().for_each(|(index,field)|{
        field.id = field_ids[index].clone();
    });
    let field_ams = fields.iter()
    .map(|item|field::ActiveModel::from(item.clone()))
    .collect::<Vec<field::ActiveModel>>(); 
     Field::insert_many(field_ams).exec(&tx).await?;
    //3、TODO 插入默认的关联关系
    let table_link_ams = fields.iter().map(|field|{
        table_link::ActiveModel::from(build_table_link(next_id(), table_id.clone(),field.id.clone()))
    }).collect::<Vec<table_link::ActiveModel>>();
     TableLink::insert_many(table_link_ams).exec(&tx).await?;
    //4、提交事务
    tx.commit().await?;
    Ok(CommonResponse::new(
        ResponseCode::Success,
        ResponseMessage::Success,
        Some(serde_json::to_value(insert_rsult.last_insert_id).unwrap()),
    ))
}


pub async fn add_table(
    db: web::Data<DatabaseConnection>,
    table_vo:TableVo
)->Result<CommonResponse, DrawDBError> {
    //1、开启事务
    let tx = db.begin().await?;
    let  table_id = next_id();
    let temp_table_vo = table_vo.clone();
    let mut table_add = temp_table_vo.convert_to_table();
    table_add.id = table_id.clone(); 
    let table_am = table::ActiveModel::from(table_add);
    Table::insert(table_am).exec(&tx).await?;
    //2、TODO 插入默认的field
    let field_size = temp_table_vo.fields.clone().unwrap_or(vec![]).len();
    let field_ids = next_ids(field_size);
    let mut fields = temp_table_vo.fields.clone().unwrap_or(vec![]).iter().map( |field|{
         field.convert_to_field()
    }).collect::<Vec<field::Model>>();
    fields.iter_mut().enumerate().for_each(|(index,field)|{
        field.id = field_ids[index].clone();
    });
    let field_ams = fields.iter()
    .map(|item|field::ActiveModel::from(item.clone()))
    .collect::<Vec<field::ActiveModel>>(); 
     Field::insert_many(field_ams).exec(&tx).await?;
    //3、TODO 插入默认的关联关系
    let table_link_ams = fields.iter().map(|field|{
        table_link::ActiveModel::from(build_table_link(next_id(), table_id.clone(),field.id.clone()))
    }).collect::<Vec<table_link::ActiveModel>>();
     TableLink::insert_many(table_link_ams).exec(&tx).await?;
    //4、提交事务
    tx.commit().await?;
    Ok(CommonResponse::new(ResponseCode::Success, ResponseMessage::Success, Some(serde_json::to_value(true).unwrap())))
}