use crate::entity::task::{ActiveModel as Task, Model as TaskModel};
use sea_orm::ActiveValue;
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
    pub fn convert_to_task(&self, id: String) -> TaskModel {
        TaskModel {
            id,
            complete: self.complete,
            order: self.order,
            details: self.details.clone(),
            title: self.title.clone(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TaskUpdateVo {
    pub id: String,
    pub complete: Option<bool>,
    pub order: Option<i32>,
    pub details: Option<String>,
    pub title: Option<String>,
}

impl TaskUpdateVo {
    pub fn convert_to_task(&self) -> TaskModel {
        TaskModel {
            id: self.id.clone(),
            complete: self.complete,
            order: self.order,
            details: self.details.clone(),
            title: self.title.clone(),
        }
    }
    pub fn convert_to_active_model(&self) -> Task {
        let id = ActiveValue::Set(self.id.clone());
        // - 主键要用 Set 或 Unchanged，确保 SQL 里有 WHERE
        let mut am: Task = Task {
                id,
                ..Default::default()
            };
    
            // 只有当客户端传了新值才标记成 Set
            if let Some(t) = &self.title {
                am.title = ActiveValue::Set(Some(t.clone()));
            }
            if let Some(c) = &self.complete {
                am.complete = ActiveValue::Set(Some(c.clone()));
            }
            if let Some(o) = &self.order {
                am.order = ActiveValue::Set(Some(o.clone()));
            }
            if let Some(d) = &self.details {
                am.details = ActiveValue::Set(Some(d.clone()));
            }

            am
        }
    }

