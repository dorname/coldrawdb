use crate::db::models::BusinessModel;
use rusqlite::Row;
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Template {
    pub id: i64,
    pub custom: bool,
    pub content: String,
}

impl Template {
    pub fn new(id: i64, custom: bool, content: String) -> Self {
        Self { id, custom, content }
    }

    pub fn from_tuple(tuple: (i64, bool, String)) -> Self {
        Self { id: tuple.0, custom: tuple.1, content: tuple.2 }
    }
}

impl BusinessModel for Template {
    fn get_columns(&self) -> String {
        "id, custom, content".to_string()
    }

    fn get_values(&self) -> String {
        format!("{}, {}, {}", self.id, self.custom, self.content)
    }

    fn from_raw(row: &Row) -> Self {
        let (id, custom, content) = (
            row.get::<_, i64>(0).expect("Failed to parse template from row"),
            row.get::<_, bool>(1).expect("Failed to parse template from row"),
            row.get::<_, String>(2).expect("Failed to parse template from row"),
        );
        Self::new(id, custom, content)
    }
}
