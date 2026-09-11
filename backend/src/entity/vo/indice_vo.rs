use crate::entity::indice::Model as IndiceModel;
use serde::{Serialize, Deserialize};
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct IndiceVo {
    pub id: String,
    pub name: Option<String>,
    pub unique: Option<bool>,
}

impl IndiceVo {
    pub fn convert_to_indice(&self) -> IndiceModel {
        IndiceModel {
            id: self.id.clone(),
            name: self.name.clone(),
            unique: self.unique.clone(),    
        }
    }
}