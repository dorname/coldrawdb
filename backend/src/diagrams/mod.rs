use actix_web::{delete, post};
use actix_web::{get, web};
use sea_orm::{ActiveModelTrait, DatabaseConnection, TransactionTrait};
use sea_orm::EntityTrait;
use crate::common::ResponseCode;
use crate::common::ResponseMessage;
use crate::entity::diagram::{self, ActiveModel};
use crate::entity::prelude::*;
use crate::entity::vo::DiagramVo;
use crate::next_id;
use crate::{common::CommonResponse, error::DrawDBError};

/// 图表模块
pub fn diagrams_routes(config: &mut web::ServiceConfig) {
    config.service(query_all_diagrams);
    config.service(add_diagram);
    config.service(update_diagram);
    config.service(delete_diagram);
}

/// 查询所有图表
#[get("/queryAll")]
async fn query_all_diagrams(
    db: web::Data<DatabaseConnection>,
) -> Result<CommonResponse, DrawDBError> {
    let conn = db.get_ref();
    let diagrams = Diagram::find().all(conn).await?;
    let diagram_vos:Vec<DiagramVo> = diagrams
    .iter()
    .map(|diagram| DiagramVo::from(diagram)).collect();
    Ok(CommonResponse::new(
        ResponseCode::Success,
        ResponseMessage::Success,
        Some(serde_json::to_value(diagram_vos).unwrap()),
    ))
}

/// 查询图表
#[get("/query/{id}")]
async fn query_diagram(
    db: web::Data<DatabaseConnection>,
    id: web::Path<String>
) -> Result<CommonResponse, DrawDBError> {
    let conn = db.get_ref();
    let id = id.into_inner();
    let diagram = Diagram::find_by_id(id).one(conn).await?;
    Ok(CommonResponse::new(
        ResponseCode::Success,
        ResponseMessage::Success,
        Some(serde_json::to_value(diagram).unwrap()),
    ))
}

/// 新增图表
#[post("/add")]
async fn add_diagram(
    db: web::Data<DatabaseConnection>,
    diagram: web::Json<DiagramVo>
) -> Result<CommonResponse, DrawDBError> {
    // 开始事务
    let tx = db.begin().await?;
    let id = next_id();
    let diagram_model = diagram.into_inner().convert_to_diagram(id);
    // 新增图表
    let active_model = ActiveModel::from(diagram_model);
    let result = active_model.insert(&tx).await?;
    // 新增图表与表的关联关系

    // 提交事务
    tx.commit().await?;
    Ok(CommonResponse::new(ResponseCode::Success,
         ResponseMessage::Success,
          Some(serde_json::to_value(result).unwrap())))
}

///更新图表
#[post("/update")]
async fn update_diagram(
    db: web::Data<DatabaseConnection>,
    diagram: web::Json<DiagramVo>
) -> Result<CommonResponse, DrawDBError>{
    //开启事务
    let tx = db.begin().await?;
    let diagram_model = diagram.convert_to_active_model();
    let result = diagram_model.update(&tx).await?;
    // TODO：
    // 1、删除与表的关联关系
    // 2、删除与引用的关联关系
    // 3、重新构建与表的关联关系
    // 4、重新构建与引用的关联关系
    // 5、更新图表
    // 6、更新引用
    tx.commit().await?;
    Ok(CommonResponse::new(ResponseCode::Success,
        ResponseMessage::Success,
         Some(serde_json::to_value(result).unwrap())))
}

///删除图表
#[delete("/detele/{id}")]
async fn delete_diagram(
    db: web::Data<DatabaseConnection>,
    id: web::Path<String>
)->Result<CommonResponse, DrawDBError>{
    let tx = db.begin().await?;
    let id = id.into_inner();
    Diagram::delete_by_id(&id).exec(&tx).await?;
    tx.commit().await?;
    Ok(CommonResponse::new(ResponseCode::Success,
        ResponseMessage::Success,
         Some(serde_json::to_value(id).unwrap())))
}


#[cfg(test)]
mod tests{
    use super::*;
    use std::collections::HashMap;
    use itertools::{self, Itertools};
    use crate::entity::{prelude::*, table, task,vo::TableVo,vo::TaskVo};
    use sea_orm::{Database, PaginatorTrait, QueryTrait, Related};

    #[actix_web::test]
    async fn test_query_related(){
        let db = Database::connect("sqlite://test.sqlite").await.unwrap();
        let db = web::Data::new(db);
        let tx = db.get_ref();

        // 查询与Diagram关联的Task、查询与Diagram关联的Table
       let tasks_map = Diagram::find()
       .find_also_related(Task)
       .all(tx)
       .await.unwrap()
       .iter()
       .filter_map(|(diagram, task)| {
           task.as_ref().map(|task| TaskVo::from_option(task, diagram.id.clone()))
       })
       .collect::<Vec<TaskVo>>()
       .into_iter()
       .into_group_map_by(|t| t.diagram_id.clone());


        let tables_map = Diagram::find()
        .find_also_related(Table)
        .all(tx)
        .await.unwrap()
        .iter()
        .filter_map(|(diagram, table)| {
            table.as_ref().map(|table| TableVo::from(table, diagram.id.clone(),None))
        })
        .collect::<Vec<TableVo>>()
        .into_iter()
        .into_group_map_by(|t|{
            t.diagram_id.clone()
        });


        let diagrams = Diagram::find()
        .paginate(tx , 5)
        .fetch().await.unwrap()
        .into_iter()
        .map(|d|{
            let mut dia = DiagramVo::from(&d);
            dia.tables = tables_map.get(&dia.id).cloned();
            dia.tasks = tasks_map.get(&dia.id).cloned();
            dia
        }).collect::<Vec<DiagramVo>>();

        println!("{:?}",diagrams);
    }
}
