use crate::entity::task::Model as Task;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TaskAddVo {
    pub diagram_id: String,
    pub complete: Option<bool>,
    pub order: Option<i32>,
    pub details: Option<String>,
    pub title: Option<String>,
}

impl TaskAddVo {
    pub fn convert_to_task(&self, id: String) -> Task {
        Task {
            id,
            complete: self.complete,
            order: self.order,
            details: self.details.clone(),
            title: self.title.clone(),
        }
    }
}