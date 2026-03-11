//! `SeaORM` Entity

use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize)]
#[sea_orm(table_name = "template")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: String,
    pub title: Option<String>,
    pub description: Option<String>,
    pub database: Option<String>,
    pub custom: Option<i32>,
    #[sea_orm(column_type = "Text")]
    pub tables_json: Option<String>,
    #[sea_orm(column_type = "Text")]
    pub relationships_json: Option<String>,
    #[sea_orm(column_type = "Text")]
    pub notes_json: Option<String>,
    #[sea_orm(column_type = "Text")]
    pub subject_areas_json: Option<String>,
    #[sea_orm(column_type = "Text")]
    pub todos_json: Option<String>,
    #[sea_orm(column_type = "Text")]
    pub types_json: Option<String>,
    #[sea_orm(column_type = "Text")]
    pub enums_json: Option<String>,
    pub pan: Option<String>,
    pub zoom: Option<String>,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

