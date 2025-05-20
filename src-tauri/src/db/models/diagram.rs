
use crate::db::models::common::*;
// 图表
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
}
