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
mod tests {
    use super::*;
    use crate::entity::vo::{TableVo, TaskVo};
    use itertools::Itertools;
    use sea_orm::{Database, PaginatorTrait};

    #[actix_web::test]
    async fn ut_pu_20_query_related_with_isolated_database() {
        let db_path = std::env::temp_dir().join(format!(
            "coldrawdb_query_related_{}.sqlite",
            uuid::Uuid::new_v4()
        ));
        std::fs::File::create(&db_path).unwrap();
        let db = Database::connect(format!("sqlite://{}?", db_path.display()))
            .await
            .unwrap();
        crate::init::init_table("init.sql", &db).await.unwrap();

        // 查询与Diagram关联的Task、查询与Diagram关联的Table
        let tasks_map = Diagram::find()
            .find_also_related(Task)
            .all(&db)
            .await
            .unwrap()
            .iter()
            .filter_map(|(diagram, task)| {
                task.as_ref()
                    .map(|task| TaskVo::from_option(task, diagram.id.clone()))
            })
            .collect::<Vec<TaskVo>>()
            .into_iter()
            .into_group_map_by(|task| task.diagram_id.clone());

        let tables_map = Diagram::find()
            .find_also_related(Table)
            .all(&db)
            .await
            .unwrap()
            .iter()
            .filter_map(|(diagram, table)| {
                table
                    .as_ref()
                    .map(|table| TableVo::from(table, diagram.id.clone(), None))
            })
            .collect::<Vec<TableVo>>()
            .into_iter()
            .into_group_map_by(|table| table.diagram_id.clone());

        let diagrams = Diagram::find()
            .paginate(&db, 5)
            .fetch()
            .await
            .unwrap()
            .into_iter()
            .map(|diagram| {
                let mut diagram_vo = DiagramVo::from(&diagram);
                diagram_vo.tables = tables_map.get(&diagram_vo.id).cloned();
                diagram_vo.tasks = tasks_map.get(&diagram_vo.id).cloned();
                diagram_vo
            })
            .collect::<Vec<DiagramVo>>();

        assert!(tasks_map.is_empty());
        assert!(tables_map.is_empty());
        assert!(diagrams.is_empty());
        db.close().await.unwrap();
        std::fs::remove_file(&db_path).unwrap();
        crate::verify_reporter::report_pass("UT-PU-20", 0);
    }
}
