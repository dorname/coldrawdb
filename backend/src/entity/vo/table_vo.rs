use sea_orm::ActiveValue;
use serde::{Deserialize, Serialize};    
use crate::entity::diagram;
use crate::entity::dto::FieldWithTable;
use crate::entity::field::Model as FieldModel;
use crate::entity::table::Model as TableModel;
use crate::entity::table::ActiveModel as TableActiveModel;
use crate::entity::table_link::Model as TableLinkModel;


#[derive(Serialize, Deserialize,Clone,Debug,PartialEq,Eq)]
pub struct TableVo {
    pub id: String,
    pub color: Option<String>,
    pub comment: Option<String>,
    pub locked: Option<bool>,
    pub name: Option<String>,
    pub x: Option<String>,
    pub y: Option<String>,
    pub fields: Option<Vec<FieldVo>>,
    pub diagram_id: String
}

impl TableVo {
    pub fn convert_to_table(&self) -> TableModel {
        TableModel {
            id: self.id.clone(),
            name: self.name.clone(),
            color: self.color.clone(),
            comment: self.comment.clone(),
            locked: self.locked.clone(),
            x: self.x.clone(),
            y: self.y.clone(),
        }
    }
    pub fn build_from_table(table: TableModel,fields: Option<Vec<FieldVo>>,diagram_id:String) -> Self {
        Self {
            id: table.id.clone(),
            name: table.name.clone(),
            fields: fields,
            color: table.color.clone(),
            comment: table.comment.clone(),
            locked: table.locked.clone(),
            x: table.x.clone(),
            y: table.y.clone(),
            diagram_id: diagram_id
        }
    }

    pub fn convert_to_active_model(&self) -> TableActiveModel{
        let id = ActiveValue::Set(self.id.clone());
        let am:TableActiveModel = TableActiveModel{
            id,
            ..Default::default()
        };
        am
    }
}

#[derive(Serialize, Deserialize,Clone,Debug,PartialEq,Eq)]
pub struct FieldVo {
    pub id: String,
    pub table_id: Option<String>,
    pub check: Option<String>,
    pub comment: Option<String>,
    pub default: Option<String>,
    pub increment: Option<bool>,
    pub not_null: Option<bool>,
    pub primary: Option<bool>,
    pub size: Option<i32>,
    pub r#type: Option<String>,
    pub unique: Option<bool>,
    pub name: Option<String>,
}



impl FieldVo {
    pub fn convert_to_field(&self) -> FieldModel {
        FieldModel {
            id: self.id.clone(),
            check: self.check.clone(),
            comment: self.comment.clone(),
            default: self.default.clone(),
            increment: self.increment.clone(),
            not_null: self.not_null.clone(),
            primary: self.primary.clone(),  
            size: self.size.clone(),
            r#type: self.r#type.clone(),
            unique: self.unique.clone(),
            name: self.name.clone(),
        }
    }

    pub fn build_from_field_with_table(field: FieldWithTable) -> Self {
        Self {
            id: field.id.clone(),
            table_id: field.table_id.clone(),
            check: field.check.clone(),
            comment: field.comment.clone(),
            default: field.default.clone(),
            increment: field.increment.clone(),
            not_null: field.not_null.clone(),
            primary: field.primary.clone(),
            size: field.size.clone(),
            r#type: field.r#type.clone(),
            unique: field.unique.clone(),
            name: field.name.clone(),
        }
    }
}

pub fn build_table_link(id: String,table_id: String,field_id: String) -> TableLinkModel {
    TableLinkModel {
        id,
        table_id: Some(table_id),
        field_id: Some(field_id),
        
    }
}
