pub trait Model {
    fn new(table_name: String, columns: String, values: String) -> Self {
        Self { table_name, columns, values }
    }
    fn get_sql(&self) -> String;
}

pub struct InsertModel {
    pub table_name: String,
    pub columns: String,
    pub values: String,
}

impl Model for InsertModel {
    fn get_sql(&self) -> String {
        format!("INSERT INTO {} ({}) VALUES ({})", self.table_name, self.columns, self.values)
    }
}

pub struct UpdateModel {
    pub table_name: String,
    pub columns: String,
    pub values: String,
}

impl Model for UpdateModel {
    fn get_sql(&self) -> String {
        format!("UPDATE {} SET {} WHERE {}", self.table_name, self.columns, self.values)
    }
}

pub struct DeleteModel {        
    pub table_name: String,
    pub where_clause: String,
}

impl Model for DeleteModel {
    fn get_sql(&self) -> String {
        format!("DELETE FROM {} WHERE {}", self.table_name, self.where_clause)
    }
}

pub struct SelectModel {
    pub table_name: String,
    pub columns: String,
    pub where_clause: String,
}

impl Model for SelectModel {
    fn get_sql(&self) -> String {
        format!("SELECT {} FROM {} WHERE {}", self.columns, self.table_name, self.where_clause)
    }
}

