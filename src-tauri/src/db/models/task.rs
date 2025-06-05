use crate::db::models::BusinessModel;
use rusqlite::Row;
use serde::{Serialize, Deserialize};
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    /// 主键字段
    id: i64,
    /// 完成字段
    compele: bool,
    /// 详情字段
    details: String,
    /// 排序字段
    task_order: i64,
    /// 优先级字段
    /// 0 无 
    /// 1 低
    /// 2 中
    /// 3 高
    priority: i64,
    /// 标题字段
    title: String,
}

impl Task {
    pub fn new(id: i64, compele: bool, details: String, task_order: i64, priority: i64, title: String) -> Self {
        Self { id, compele, details, task_order, priority, title }
    }

    pub fn empty_task() -> Self {
        Self { id: 0, compele: false, details: "".to_string(), task_order: 0, priority: 0, title: "".to_string() }
    }

    pub fn from_tuple(tuple: (i64, bool, String, i64, i64, String)) -> Self {
        Self { id: tuple.0, compele: tuple.1, details: tuple.2, task_order: tuple.3, priority: tuple.4, title: tuple.5 }
    }
}

impl BusinessModel for Task {
    fn get_columns(&self) -> String {
        "id, compele, details, task_order, priority, title".to_string()
    }

    fn get_values(&self) -> String {
        format!("{}, {}, {}, {}, {}, {}", self.id, self.compele, self.details, self.task_order, self.priority, self.title)
    }

    fn from_raw(row: &Row) -> Self {
        let (id, compele, details, task_order, priority, title) = 
        (row.get::<_, i64>(0).expect("Failed to parse task from row"), 
        row.get::<_, bool>(1).expect("Failed to parse task from row"), 
        row.get::<_, String>(2).expect("Failed to parse task from row"), 
        row.get::<_, i64>(3).expect("Failed to parse task from row"), 
        row.get::<_, i64>(4).expect("Failed to parse task from row"), 
        row.get::<_, String>(5).expect("Failed to parse task from row"));
        Self::new(id, compele, details, task_order, priority, title)
    }

    fn get_order_by(&self) -> String {
        "task_order".to_string()
    }
}