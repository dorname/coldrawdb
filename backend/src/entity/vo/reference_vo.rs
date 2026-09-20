use crate::entity::reference::Model as ReferenceModel;
use serde::{Serialize, Deserialize};
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReferenceVo {
    pub id: String,
    pub name: Option<String>,
    pub start_table_id: Option<String>,
    pub end_table_id: Option<String>,
    pub cardinality: Option<String>,
    pub delete_constraint: Option<String>,
    pub end_field_id: Option<String>,
    pub start_field_id: Option<String>,
    pub update_constraint: Option<String>,
    // fix-remote-github-issues-7-18（issue #12，core-01b §4.1）：关系线颜色；
    // 存量导入 JSON 无此字段 → None → 落库 ''（跟随源表色），向后兼容
    #[serde(default)]
    pub color: Option<String>,
    // fix-open-issues-19-22（#21）
    #[serde(default)]
    pub line_type: Option<String>,
    #[serde(default)]
    pub stroke_style: Option<String>,
}

impl ReferenceVo {
    pub fn convert_to_reference(&self) -> ReferenceModel {
        ReferenceModel {
            id: self.id.clone(),
            name: self.name.clone(),
            start_table_id: self.start_table_id.clone(),
            end_table_id: self.end_table_id.clone(),
            cardinality: self.cardinality.clone(),
            delete_constraint: self.delete_constraint.clone(),
            end_field_id: self.end_field_id.clone(),
            start_field_id: self.start_field_id.clone(),
            update_constraint: self.update_constraint.clone(),
            color: self.color.clone().unwrap_or_default(),
            line_type: self
                .line_type
                .clone()
                .filter(|s| !s.is_empty())
                .unwrap_or_else(|| "bezier".into()),
            stroke_style: self
                .stroke_style
                .clone()
                .filter(|s| !s.is_empty())
                .unwrap_or_else(|| "solid".into()),
        }
    }
}
