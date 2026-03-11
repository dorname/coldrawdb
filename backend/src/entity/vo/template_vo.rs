use serde::{Deserialize, Serialize};

use crate::entity::template;
use crate::entity::vo::area_vo::AreaVo;
use crate::entity::vo::note_vo::NoteVo;
use crate::entity::vo::reference_vo::ReferenceVo;
use crate::entity::vo::table_vo::TableVo;
use crate::entity::vo::TaskVo;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TemplateVo {
    pub id: String,
    pub title: Option<String>,
    pub description: Option<String>,
    pub database: Option<String>,
    pub custom: Option<i32>,
    pub tables: Option<Vec<TableVo>>,
    pub relationships: Option<Vec<ReferenceVo>>,
    pub notes: Option<Vec<NoteVo>>,
    #[serde(rename = "subjectAreas")]
    pub subject_areas: Option<Vec<AreaVo>>,
    pub todos: Option<Vec<TaskVo>>,
    pub types: Option<serde_json::Value>,
    pub enums: Option<serde_json::Value>,
    pub pan: Option<String>,
    pub zoom: Option<String>,
}

impl TemplateVo {
    pub fn convert_to_model(&self, id: String) -> template::Model {
        template::Model {
            id,
            title: self.title.clone(),
            description: self.description.clone(),
            database: self.database.clone(),
            custom: self.custom,
            tables_json: self
                .tables
                .as_ref()
                .and_then(|v| serde_json::to_string(v).ok()),
            relationships_json: self
                .relationships
                .as_ref()
                .and_then(|v| serde_json::to_string(v).ok()),
            notes_json: self
                .notes
                .as_ref()
                .and_then(|v| serde_json::to_string(v).ok()),
            subject_areas_json: self
                .subject_areas
                .as_ref()
                .and_then(|v| serde_json::to_string(v).ok()),
            todos_json: self
                .todos
                .as_ref()
                .and_then(|v| serde_json::to_string(v).ok()),
            types_json: self
                .types
                .as_ref()
                .and_then(|v| serde_json::to_string(v).ok()),
            enums_json: self
                .enums
                .as_ref()
                .and_then(|v| serde_json::to_string(v).ok()),
            pan: self.pan.clone(),
            zoom: self.zoom.clone(),
            created_at: None,
            updated_at: None,
        }
    }

    pub fn from(model: &template::Model) -> Self {
        Self {
            id: model.id.clone(),
            title: model.title.clone(),
            description: model.description.clone(),
            database: model.database.clone(),
            custom: model.custom,
            tables: model
                .tables_json
                .as_ref()
                .and_then(|s| serde_json::from_str(s).ok()),
            relationships: model
                .relationships_json
                .as_ref()
                .and_then(|s| serde_json::from_str(s).ok()),
            notes: model
                .notes_json
                .as_ref()
                .and_then(|s| serde_json::from_str(s).ok()),
            subject_areas: model
                .subject_areas_json
                .as_ref()
                .and_then(|s| serde_json::from_str(s).ok()),
            todos: model
                .todos_json
                .as_ref()
                .and_then(|s| serde_json::from_str(s).ok()),
            types: model
                .types_json
                .as_ref()
                .and_then(|s| serde_json::from_str(s).ok()),
            enums: model
                .enums_json
                .as_ref()
                .and_then(|s| serde_json::from_str(s).ok()),
            pan: model.pan.clone(),
            zoom: model.zoom.clone(),
        }
    }
}

