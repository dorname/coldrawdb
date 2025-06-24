use crate::entity::note::Model as NoteModel;
use serde::{Serialize, Deserialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct NoteVo {
    pub id: String,
    pub content: Option<String>,
    pub color: Option<String>,
    pub title: Option<String>,
    pub height: Option<String>,
    pub x: Option<String>,
    pub y: Option<String>,
}

impl NoteVo {
    pub fn convert_to_note(&self) -> NoteModel {
        NoteModel {
            id: self.id.clone(),
            content: self.content.clone(),
            color: self.color.clone(),
            title: self.title.clone(),
            height: self.height.clone(),
            x: self.x.clone(),
            y: self.y.clone(),
        }
    }
}