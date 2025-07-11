use sea_orm::ActiveValue;
use serde::{Serialize,Deserialize};
use crate::entity::diagram::{Model as DiagramModel,ActiveModel as Diagram};
use crate::entity::vo::table_vo::TableVo;
use crate::entity::vo::area_vo::AreaVo;
use crate::entity::vo::reference_vo::ReferenceVo;
use crate::entity::vo::indice_vo::IndiceVo;
use crate::entity::vo::note_vo::NoteVo;
use crate::entity::vo::TaskVo;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DiagramVo{
    pub id: String,
    pub zoom: Option<String>,
    pub database: Option<String>,
    pub name: Option<String> ,
    // todo 新增、删除、修改表时，都是通过这个表来操作的
    pub tables: Option<Vec<TableVo>>,
    // todo 新增、删除、修改区域时，都是通过这个表来操作的
    pub areas: Option<Vec<AreaVo>>,
    // todo 新增、删除、修改关联关系时，都是通过这个表来操作的
    pub references: Option<Vec<ReferenceVo>>,
    // todo 新增、删除、修改索引时，都是通过这个表来操作的
    pub indices: Option<Vec<IndiceVo>>,
    // todo 新增、删除、修改任务时，都是通过这个表来操作的
    pub notes: Option<Vec<NoteVo>>,
    // tasks 
    pub tasks: Option<Vec<TaskVo>>,
    pub pan: Option<String>,
    #[serde(rename = "lastModified")]
    pub last_modified: Option<String>
}

impl DiagramVo {
    //转化成diagram的方法
    pub fn convert_to_diagram(&self, id:String) -> DiagramModel {
        DiagramModel {
            id,
            database: self.database.clone(),
            zoom: self.zoom.clone(),
            name: self.name.clone(),
            pan: self.pan.clone(),
            last_modified: self.last_modified.clone()
        }
    }

    pub fn from(diagram: &DiagramModel) -> Self {
        Self {
            id: diagram.id.clone(),
            database: diagram.database.clone(),
            zoom: diagram.zoom.clone(),
            name: diagram.name.clone(),
            tables: None,
            areas: None,
            references: None,
            indices: None,
            notes: None,
            tasks:None,
            pan: diagram.pan.clone(),
            last_modified: diagram.last_modified.clone()
        }
    }

    // 转化成diagram_active_model
    pub fn convert_to_active_model(&self)-> Diagram {
        let id = ActiveValue::Set(self.id.clone());

        let mut am = Diagram{
            id,
            ..Default::default()
        };
        if let Some(_) = &self.database{
            am.database = ActiveValue::Set(self.database.clone());
        }
        if let Some(_) = &self.name  {
            am.name  = ActiveValue::Set(self.name.clone());
        }
        if let Some(_) = &self.zoom{
            am.zoom = ActiveValue::Set(self.zoom.clone());
        }
        if let Some(_) = &self.pan{
            am.pan = ActiveValue::Set(self.pan.clone());
        }
        if let Some(_) = &self.last_modified{
            am.last_modified = ActiveValue::Set(self.last_modified.clone());
        }
        am
    }
}