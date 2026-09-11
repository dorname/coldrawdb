use crate::entity::area::Model as AreaModel;
use serde::{Serialize, Deserialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AreaVo {
    pub id: String,
    pub color: Option<String>,
    pub height: Option<String>,
    pub name: Option<String>,
    pub width: Option<String>,
    pub x: Option<String>,
    pub y: Option<String>,
}

impl AreaVo {
    pub fn convert_to_area(&self) -> AreaModel {
        AreaModel {
            id: self.id.clone(),
            color: self.color.clone(),
            height: self.height.clone(),
            name: self.name.clone(),
            width: self.width.clone(),
            x: self.x.clone(),
            y: self.y.clone(),
        }
    }
}