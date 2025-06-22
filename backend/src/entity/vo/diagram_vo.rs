use sea_orm::ActiveValue;
use serde::{Serialize,Deserialize};
use crate::entity::diagram::{Model as DiagramModel,ActiveModel as Diagram};
use crate::entity::vo::table_vo::TableVo;
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DiagramVo{
    pub id: String,
    pub zoom: Option<String>,
    pub database: Option<String>,
    pub name: Option<String> ,
    // todo 新增、删除、修改表时，都是通过这个表来操作的
    pub tables: Option<Vec<TableVo>>
}

impl DiagramVo {
    //转化成diagram的方法
    pub fn convert_to_diagram(&self, id:String) -> DiagramModel {
        DiagramModel {
            id,
            database: self.database.clone(),
            zoom: self.zoom.clone(),
            name: self.name.clone()
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
        am
    }
}