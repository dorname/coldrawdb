use sea_orm::ActiveValue;
use serde::{Deserialize, Serialize};

use crate::entity::diagram::{ActiveModel as Diagram, Model as DiagramModel};
use crate::entity::vo::indice_vo::IndiceVo;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DiagramVo {
    pub id: String,
    pub zoom: Option<String>,
    pub database: Option<String>,
    pub name: Option<String>,
    pub tables: Option<serde_json::Value>,
    pub areas: Option<serde_json::Value>,
    pub references: Option<serde_json::Value>,
    pub indices: Option<Vec<IndiceVo>>,
    pub notes: Option<serde_json::Value>,
    pub tasks: Option<serde_json::Value>,
    pub pan: Option<String>,
    #[serde(rename = "lastModified")]
    pub last_modified: Option<String>,
    #[serde(rename = "gistId")]
    pub gist_id: Option<String>,
    #[serde(rename = "loadedFromGistId")]
    pub loaded_from_gist_id: Option<String>,
    pub enums: Option<serde_json::Value>,
    pub types: Option<serde_json::Value>,
}

impl DiagramVo {
    pub fn convert_to_diagram(&self, id: String) -> DiagramModel {
        DiagramModel {
            id,
            database: self.database.clone(),
            zoom: self.zoom.clone(),
            name: self.name.clone(),
            pan: self.pan.clone(),
            last_modified: self.last_modified.clone(),
            gist_id: self.gist_id.clone(),
            loaded_from_gist_id: self.loaded_from_gist_id.clone(),
            tables_json: self.tables.as_ref().and_then(|v| serde_json::to_string(v).ok()),
            references_json: self.references.as_ref().and_then(|v| serde_json::to_string(v).ok()),
            notes_json: self.notes.as_ref().and_then(|v| serde_json::to_string(v).ok()),
            areas_json: self.areas.as_ref().and_then(|v| serde_json::to_string(v).ok()),
            tasks_json: self.tasks.as_ref().and_then(|v| serde_json::to_string(v).ok()),
            enums_json: self.enums.as_ref().and_then(|v| serde_json::to_string(v).ok()),
            types_json: self.types.as_ref().and_then(|v| serde_json::to_string(v).ok()),
        }
    }

    pub fn from(diagram: &DiagramModel) -> Self {
        Self {
            id: diagram.id.clone(),
            database: diagram.database.clone(),
            zoom: diagram.zoom.clone(),
            name: diagram.name.clone(),
            tables: diagram.tables_json.as_ref().and_then(|s| serde_json::from_str(s).ok()),
            areas: diagram.areas_json.as_ref().and_then(|s| serde_json::from_str(s).ok()),
            references: diagram.references_json.as_ref().and_then(|s| serde_json::from_str(s).ok()),
            indices: None,
            notes: diagram.notes_json.as_ref().and_then(|s| serde_json::from_str(s).ok()),
            tasks: diagram.tasks_json.as_ref().and_then(|s| serde_json::from_str(s).ok()),
            pan: diagram.pan.clone(),
            last_modified: diagram.last_modified.clone(),
            gist_id: diagram.gist_id.clone(),
            loaded_from_gist_id: diagram.loaded_from_gist_id.clone(),
            enums: diagram.enums_json.as_ref().and_then(|s| serde_json::from_str(s).ok()),
            types: diagram.types_json.as_ref().and_then(|s| serde_json::from_str(s).ok()),
        }
    }

    pub fn convert_to_active_model(&self) -> Diagram {
        let id = ActiveValue::Set(self.id.clone());

        let mut am = Diagram {
            id,
            ..Default::default()
        };
        if self.database.is_some() {
            am.database = ActiveValue::Set(self.database.clone());
        }
        if self.name.is_some() {
            am.name = ActiveValue::Set(self.name.clone());
        }
        if self.zoom.is_some() {
            am.zoom = ActiveValue::Set(self.zoom.clone());
        }
        if self.pan.is_some() {
            am.pan = ActiveValue::Set(self.pan.clone());
        }
        if self.last_modified.is_some() {
            am.last_modified = ActiveValue::Set(self.last_modified.clone());
        }
        if self.gist_id.is_some() {
            am.gist_id = ActiveValue::Set(self.gist_id.clone());
        }
        if self.loaded_from_gist_id.is_some() {
            am.loaded_from_gist_id = ActiveValue::Set(self.loaded_from_gist_id.clone());
        }
        am.tables_json = ActiveValue::Set(
            self.tables.as_ref().and_then(|v| serde_json::to_string(v).ok())
        );
        am.references_json = ActiveValue::Set(
            self.references.as_ref().and_then(|v| serde_json::to_string(v).ok())
        );
        am.notes_json = ActiveValue::Set(
            self.notes.as_ref().and_then(|v| serde_json::to_string(v).ok())
        );
        am.areas_json = ActiveValue::Set(
            self.areas.as_ref().and_then(|v| serde_json::to_string(v).ok())
        );
        am.tasks_json = ActiveValue::Set(
            self.tasks.as_ref().and_then(|v| serde_json::to_string(v).ok())
        );
        am.enums_json = ActiveValue::Set(
            self.enums.as_ref().and_then(|v| serde_json::to_string(v).ok())
        );
        am.types_json = ActiveValue::Set(
            self.types.as_ref().and_then(|v| serde_json::to_string(v).ok())
        );
        am
    }
}
