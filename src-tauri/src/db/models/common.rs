use rusqlite::Row;


pub trait Model {
    fn get_insert_sql(&self) -> String;
    fn get_update_sql(&self) -> String;
    fn get_delete_sql(&self) -> String;
    fn get_select_sql(&self) -> String;
}


// 增删改查的通用模型
pub struct CommonModel {
    pub table_name: String,
    pub columns: String,
    pub values: String,
    pub where_clause: String,
}

impl CommonModel {
    pub fn new(table_name: String, columns: String, values: String, where_clause: String) -> Self {
        Self { table_name, columns, values, where_clause }
    }
}

impl Model for CommonModel {
    fn get_insert_sql(&self) -> String {
        format!("INSERT INTO {} ({}) VALUES ({})", self.table_name, self.columns, self.values)
    }
    fn get_update_sql(&self) -> String {
        format!("UPDATE {} SET {} WHERE {}", self.table_name, self.columns, self.values)
    }
    fn get_delete_sql(&self) -> String {
        format!("DELETE FROM {} WHERE {}", self.table_name, self.where_clause)
    }
    fn get_select_sql(&self) -> String {
        format!("SELECT {} FROM {} WHERE {}", self.columns, self.table_name, self.where_clause)
    }
}

/// 业务公共特征
pub trait BusinessModel {
    fn get_columns(&self) -> String;
    fn get_values(&self) -> String;
    fn from_raw(row: &Row) -> Self;
}


/// 任意实现了BusinessModel的类型可以 转化为 CommonModel
pub trait ToCommonModel {
    fn to_common_model(self,table_name: String,where_clause: String) -> CommonModel;
}

impl<T: BusinessModel> ToCommonModel for T {
    fn to_common_model(self,table_name: String,where_clause: String) -> CommonModel {
        CommonModel::new(table_name, self.get_columns(), self.get_values(), where_clause)
    }
}