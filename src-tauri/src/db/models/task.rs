use crate::db::models::BusinessModel;
use rusqlite::Row;
use serde::{Serialize, Deserialize};
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    /// 主键字段
    id: i64,
    /// 完成字段
    complete: bool,
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
    /// 所属diagram_id
    diagram_id: i64,
}

impl Task {
    pub fn new(id: i64, complete: bool, details: String, task_order: i64, priority: i64, title: String, diagram_id: i64) -> Self {
        Self { id, complete, details, task_order, priority, title, diagram_id }
    }

    pub fn empty_task() -> Self {
        Self { id: 0, complete: false, details: "".to_string(), task_order: 0, priority: 0, title: "".to_string(), diagram_id: 0 }
    }

    pub fn from_tuple(tuple: (i64, bool, String, i64, i64, String, i64)) -> Self {
        Self { id: tuple.0, complete: tuple.1, details: tuple.2, task_order: tuple.3, priority: tuple.4, title: tuple.5, diagram_id: tuple.6 }
    }
}

impl BusinessModel for Task {
    fn get_columns(&self,action: &str) -> String {
        match action {
            "insert" => "complete, details, task_order, priority, title, diagram_id".to_string(),
            _ => "id, complete, details, task_order, priority, title, diagram_id".to_string(),
        }
    }

    fn get_values(&self,action: &str) -> String {
        match action {
            "insert" => format!("{}, '{}', {}, {}, '{}', {}", self.complete, self.details, self.task_order, self.priority, self.title, self.diagram_id),
            _ => format!("{}, {}, {}, {}, {}, {}, {}", self.id, self.complete, self.details, self.task_order, self.priority, self.title, self.diagram_id),
        }
    }

    fn from_raw(row: &Row) -> Self {
        let (id, complete, details, task_order, priority, title, diagram_id) = 
        (row.get::<_, i64>(0).expect("Failed to parse task from row"), 
        row.get::<_, bool>(1).expect("Failed to parse task from row"), 
        row.get::<_, String>(2).expect("Failed to parse task from row"), 
        row.get::<_, i64>(3).expect("Failed to parse task from row"), 
        row.get::<_, i64>(4).expect("Failed to parse task from row"), 
        row.get::<_, String>(5).expect("Failed to parse task from row"),
        row.get::<_, i64>(6).expect("Failed to parse task from row"));
        Self::new(id, complete, details, task_order, priority, title, diagram_id)
    }

    fn get_order_by(&self) -> String {
        "task_order".to_string()
    }
}