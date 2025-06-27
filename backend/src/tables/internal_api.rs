use std::collections::HashMap;

use actix_web::web;
use itertools::Itertools;
use sea_orm::{ActiveValue, ColumnTrait, DatabaseConnection, DatabaseTransaction, EntityTrait, Iterable, JoinType, QueryFilter, QuerySelect, RelationTrait};
use crate::entity::dto::FieldWithTable;
use crate::entity::table::Relation as TableRelation;
use crate::entity::vo::{build_table_link, FieldVo};
use crate::entity::{diagram_link, field, prelude::*, table, table_link};
use crate::next_ids;
use crate::{entity::vo::TableVo, error::DrawDBError, next_id};


/// 查询关联表结构的方法
pub async fn query_tables(
    db: web::Data<DatabaseConnection>,
    diagram_id: String
) -> Result<Vec<TableVo>, DrawDBError> {
    let conn = db.get_ref();
    let table_models = Table::find()
    .join(JoinType::InnerJoin, TableRelation::DiagramLink.def())
    .filter(diagram_link::Column::DiagramId.eq(diagram_id.clone()))
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
        TableVo::build_from_table(table.clone(), Some(temp_fields.to_vec()),diagram_id.clone())
    }).collect::<Vec<TableVo>>();
 
    Ok(table_vos)
}

/// 新增表结构处理方法
pub async fn add_table(
    tx: &DatabaseTransaction,
    table_vo:TableVo
)->Result<String, DrawDBError> {
    // 1、新增表
    let table_id = next_id();
    let temp_table_vo = table_vo.clone();
    let mut table_add = temp_table_vo.convert_to_table();
    // 覆盖前端生成的id
    table_add.id = table_id.clone();
    let table_am = table::ActiveModel::from(table_add);
    Table::insert(table_am).exec(tx).await?;
    //2、插入fields
    let field_size = temp_table_vo.fields.clone().unwrap_or(vec![]).len();
    if field_size > 0{
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
        Field::insert_many(field_ams).exec(tx).await?;
        //3、插入默认的关联关系
        let table_link_ams = fields.iter().map(|field|{
            table_link::ActiveModel::from(build_table_link(next_id(), table_id.clone(),field.id.clone()))
        }).collect::<Vec<table_link::ActiveModel>>();
         TableLink::insert_many(table_link_ams).exec(tx).await?;
    }
    //4、返回新增的表id
    Ok(table_id.clone())
}

/// 批量新增方法
pub async fn batch_add_table(
    tx: &DatabaseTransaction,
    table_vos: Vec<TableVo>
)->Result<Vec<String>,DrawDBError>{
    fn take_and_remove(vec: &mut Vec<String>, count: usize) -> Vec<String> {
        let actual_count = count.min(vec.len()); // 确保不超过集合长度
        vec.drain(0..actual_count).collect()
    }
    fn get_fields(table_vos: Vec<TableVo>)->(Vec<field::ActiveModel>,Vec<table_link::ActiveModel>){
        // 1、处理一个元组（tableId,fields_count）
        let count_map = table_vos.iter().map(|table_vo|{
            (table_vo.id.clone(),table_vo.fields.clone().unwrap_or(vec![]).len())
        }).collect::<HashMap<String,usize>>();
        let field_count = count_map.values().sum();
        let field_ids = next_ids(field_count);
        let link_ids = next_ids(field_count);
        // 先处理成一个hashMap类型 key: tableId value:Vec<String> field_ids
        let mut field_ids_map = count_map.iter().map(|(table_id,fields_count)|{
            (table_id.clone(),take_and_remove(&mut field_ids, *fields_count))
        }).collect::<HashMap<String,Vec<String>>>();
        let link_ids_map = count_map.iter().map(|(table_id,fields_count)|{
            (table_id.clone(),take_and_remove(&mut link_ids, *fields_count))
        }).collect::<HashMap<String,Vec<String>>>();
        // 2、hashMap key:tableId value:Vec<fieldVo>
        let field_vo_map = table_vos.iter().map(|table_vo|{
            (table_vo.id.clone(),table_vo.fields.clone().unwrap_or(vec![]))
        }).collect::<HashMap<String,Vec<FieldVo>>>();
        // 3、收集fields 设置field_ids 生成field_ams
        let field_ams = field_vo_map.iter().map(|(table_id,field_vos)|{
            field_vos.iter().enumerate().map(|(index,field_vo)|{
                let mut field_am = field::ActiveModel::from(field_vo.convert_to_field());
                field_am.id = ActiveValue::Set(field_ids_map.get(table_id).unwrap()[index].clone());
                field_am
            }).collect::<Vec<field::ActiveModel>>()
        }).collect::<Vec<Vec<field::ActiveModel>>>();
        // 4、收集table_link_ams
        let table_link_ams = 
        (field_ams,table_link_ams)
    }
    let table_ids = next_ids(table_vos.len());
    let table_ams = table_vos.iter().enumerate().map(|(index,table_vo)|{
        let mut table_am = table::ActiveModel::from(table_vo.convert_to_table());
        table_am.id = ActiveValue::Set(table_ids[index].clone());
        table_am
    }).collect::<Vec<table::ActiveModel>>();
    Table::insert_many(table_ams).exec(tx).await?;
    Ok(vec![])
}

/// 更新表结构
pub async fn update_table(
    tx: &DatabaseTransaction,
    table_vo:TableVo
)->Result<bool,DrawDBError>{
    //1、根据表的Id更新表信息
    let table_id = table_vo.id.clone();
    let table_am = table::ActiveModel::from(table_vo.convert_to_table());
    Table::update(table_am).filter(table::Column::Id.eq(table_id.clone())).exec(tx).await?;
    //2、查询原本所有的字段
    let origin_fields = Field::find()
    .select_only()
    // 先把 field 的所有列都选一遍
    .columns(field::Column::iter())
    // 再手动加上 table_link.table_id
    .column(table_link::Column::TableId)
    .join(
        JoinType::InnerJoin,
        field::Relation::TableLink.def(),
    )
    .filter(table_link::Column::TableId.eq(table_id.clone()))
    .into_model::<FieldWithTable>()  // 映射到我们的 DTO
    .all(tx)
    .await?;
    //3、删除所有原本关联的字段
    let origin_field_ids = origin_fields.iter().map(|field|field.id.clone()).collect::<Vec<String>>();
    Field::delete_many()
    .filter(field::Column::Id.is_in(origin_field_ids))
    .exec(tx)
    .await?;
    //4、删除表与字段构建的关联关系
    TableLink::delete_many()
    .filter(table_link::Column::TableId.eq(table_id.clone()))
    .exec(tx)
    .await?;

    //5、获取当前新的字段
    let fields = table_vo.fields
    .ok_or(DrawDBError::DeconstructError("fields is none".to_string()))?;
    //5、新增字段
    let field_ids = next_ids(fields.len());
    let field_ams = fields.iter().enumerate().map(|(index,field)|{
        let mut field_am = field::ActiveModel::from(field.convert_to_field());
        field_am.id = ActiveValue::Set(field_ids[index].clone());
        field_am
    }).collect::<Vec<field::ActiveModel>>();
    Field::insert_many(field_ams).exec(tx).await?;
    //6、新增表与字段的关联关系
    let table_link_ams = fields.iter().map(|item|{
        table_link::ActiveModel::from(build_table_link(next_id(), table_id.clone(),item.id.clone()))
    }).collect::<Vec<table_link::ActiveModel>>();
    TableLink::insert_many(table_link_ams).exec(tx).await?;
    Ok(true)
}

/// 删除表结构
pub async fn delete_table(
    tx: &DatabaseTransaction,
    table_id: String
)->Result<bool,DrawDBError>{
    //1、删除表
    Table::delete_by_id(table_id.clone()).exec(tx).await?;
    //2、删除表与字段的关联关系
    TableLink::delete_many()
    .filter(table_link::Column::TableId.eq(table_id.clone()))
    .exec(tx)
    .await?;
    //3、删除字段
    let origin_fields = Field::find()
    .select_only()
    // 先把 field 的所有列都选一遍
    .columns(field::Column::iter())
    // 再手动加上 table_link.table_id
    .column(table_link::Column::TableId)
    .join(
        JoinType::InnerJoin,
        field::Relation::TableLink.def(),
    )
    .filter(table_link::Column::TableId.eq(table_id.clone()))
    .into_model::<FieldWithTable>()  // 映射到我们的 DTO
    .all(tx)
    .await?;
    let origin_field_ids = origin_fields.iter().map(|field|field.id.clone()).collect::<Vec<String>>();
    Field::delete_many()
    .filter(field::Column::Id.is_in(origin_field_ids))
    .exec(tx)
    .await?;
    Ok(true)
}