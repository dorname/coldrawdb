use sea_orm::FromQueryResult;

// 1) 定义一个「结果 DTO」，FromQueryResult 会帮你映射 select 出来的每一列
#[derive(Debug, FromQueryResult)]
pub struct FieldWithTable {
    // 这里的字段名要和你 select 出来的 column 一一对应
    pub id: String,
    pub check: Option<String>,
    pub comment: Option<String>,
    pub default: Option<String>,
    pub increment: Option<bool>,
    pub not_null: Option<bool>,
    pub primary: Option<bool>,
    pub size: Option<i32>,
    pub r#type: Option<String>,
    pub unique: Option<bool>,
    pub table_id: String,
    pub name: Option<String>,
}
