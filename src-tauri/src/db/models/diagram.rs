
use crate::db::models::common::*;
use rusqlite::Row;
use serde::{Serialize, Deserialize};
// 图表
#[derive(Debug, Clone,Copy,Serialize,Deserialize)]
pub struct Diagram {
    pub id: i64,
    pub last_modified: i64,
    pub loaded_from_gist_id: Option<i64>,
}

impl BusinessModel for Diagram {
    fn get_columns(&self) -> String {
        "id, last_modified, loaded_from_gist_id".to_string()
    }
    fn get_values(&self) -> String {
        format!("{}, {}, {}",
        self.id, 
        self.last_modified, 
        self.loaded_from_gist_id.map_or("null".to_string(), |nll| nll.to_string()))
    }
    fn from_raw(row: &Row) -> Self {
       let (id, last_modified, loaded_from_gist_id) = 
       (row.get::<_, i64>(0).expect("Failed to parse diagram from row"), 
       row.get::<_, i64>(1).expect("Failed to parse diagram from row"), 
       row.get::<_, Option<i64>>(2).expect("Failed to parse diagram from row"));
       Self::new(id, last_modified, loaded_from_gist_id)
    }
    fn get_order_by(&self) -> String {
        "id".to_string()
    }
}

impl Diagram{
    pub fn new(id: i64, last_modified: i64, loaded_from_gist_id: Option<i64>) -> Self {
        Self { id, last_modified, loaded_from_gist_id }
    }

    pub fn from_tuple(tuple: (i64, i64, Option<i64>)) -> Self {
        Self { id: tuple.0, last_modified: tuple.1, loaded_from_gist_id: tuple.2 }
    }
    fn get_order_by(&self) -> String {
        "id".to_string()
    }
}