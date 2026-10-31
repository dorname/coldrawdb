use actix_web::{delete, post};
use actix_web::{get, web};
use sea_orm::{ActiveModelTrait, ConnectionTrait, DatabaseConnection, TransactionTrait};
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
///
/// fix-select-bg-diagram-delete-and-log-format（core-S04 图表行）：
/// queryAll 只返回 is_deleted=0 的图表，避免回收站/已删除图表混入创建空间的关联图表候选。
/// is_deleted 列由 migration 0001 提供（NOT NULL DEFAULT 0），实体 Model 未含该列，
/// 故用 sea_query Expr 按裸列名过滤。
#[get("/queryAll")]
async fn query_all_diagrams(
    db: web::Data<DatabaseConnection>,
) -> Result<CommonResponse, DrawDBError> {
    let diagram_vos = list_active_diagrams(db.get_ref()).await?;
    Ok(CommonResponse::new(
        ResponseCode::Success,
        ResponseMessage::Success,
        Some(serde_json::to_value(diagram_vos).unwrap()),
    ))
}

/// 查询全部未删除图表（UT-S04-15 直接锚定本函数）
async fn list_active_diagrams(conn: &DatabaseConnection) -> Result<Vec<DiagramVo>, DrawDBError> {
    // 旧 diagram::Model 不含迁移后的 revision/updated_at；列表直接读取真实列，
    // 避免 MCP 与创建房间候选把已有元数据回退成 0/null。
    let rows = conn.query_all(sea_orm::Statement::from_sql_and_values(
        sea_orm::DbBackend::Sqlite,
        "SELECT d.id, d.zoom, d.name, d.database, d.last_modified, d.pan, d.revision, d.updated_at, \
         (SELECT COUNT(*) FROM diagram_link dl JOIN \"table\" t ON t.id = dl.table_id \
          WHERE dl.diagram_id = d.id AND t.is_deleted = 0) AS table_count, \
         (SELECT COUNT(*) FROM diagram_link dl JOIN \"reference\" r ON r.id = dl.reference_id \
          WHERE dl.diagram_id = d.id AND r.is_deleted = 0) AS reference_count \
         FROM diagram d WHERE d.is_deleted = 0 \
         ORDER BY COALESCE(d.updated_at, d.last_modified) DESC, \
                  COALESCE(d.name, '') COLLATE NOCASE ASC, d.id ASC",
        vec![],
    )).await?;
    let mut diagrams = Vec::with_capacity(rows.len());
    for row in rows {
        let last_modified = row.try_get::<Option<String>>("", "last_modified")?;
        let updated_at = row
            .try_get::<Option<String>>("", "updated_at")?
            .or_else(|| last_modified.clone());
        diagrams.push(DiagramVo {
            id: row.try_get("", "id")?,
            zoom: row.try_get("", "zoom")?,
            database: row.try_get("", "database")?,
            name: row.try_get("", "name")?,
            tables: None,
            areas: None,
            references: None,
            indices: None,
            notes: None,
            tasks: None,
            pan: row.try_get("", "pan")?,
            revision: row.try_get("", "revision")?,
            updated_at,
            last_modified,
            table_count: row.try_get("", "table_count")?,
            reference_count: row.try_get("", "reference_count")?,
        });
    }
    Ok(diagrams)
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

    /// UT-S04-15：queryAll 只返回 is_deleted=0 的图表
    /// （fix-select-bg-diagram-delete-and-log-format，core-S04 图表行）
    #[actix_web::test]
    async fn ut_s04_15_query_all_excludes_deleted() {
        let db_path = std::env::temp_dir().join(format!(
            "coldrawdb_query_all_{}.sqlite",
            uuid::Uuid::new_v4()
        ));
        std::fs::File::create(&db_path).unwrap();
        let db = Database::connect(format!("sqlite://{}?", db_path.display()))
            .await
            .unwrap();
        crate::init::init_table("init.sql", &db).await.unwrap();
        crate::init::apply_migrations("migrations", &db).await.unwrap();

        // 一条正常图表 + 一条已软删除图表
        for (id, name, deleted) in [
            ("d-active", "活跃图表", 0),
            ("d-deleted", "已删图表", 1),
        ] {
            sea_orm::ConnectionTrait::execute(
                &db,
                sea_orm::Statement::from_sql_and_values(
                    sea_orm::DbBackend::Sqlite,
                    "INSERT INTO diagram(id, name, is_deleted) VALUES(?, ?, ?)",
                    vec![id.into(), name.into(), deleted.into()],
                ),
            )
            .await
            .unwrap();
        }

        let vos = list_active_diagrams(&db).await.unwrap();
        assert_eq!(vos.len(), 1, "UT-S04-15: 已删除图表不应出现在 queryAll 结果");
        assert_eq!(vos[0].id, "d-active", "UT-S04-15: 仅返回未删除图表");

        db.close().await.unwrap();
        std::fs::remove_file(&db_path).unwrap();
        crate::verify_reporter::report_pass("UT-S04-15", 0);
    }

    /// UT-S04-20：queryAll 返回真实 revision/updated_at，并保留 lastModified 与软删过滤。
    #[actix_web::test]
    async fn ut_s04_20_query_all_returns_real_metadata() {
        let db_path = std::env::temp_dir().join(format!(
            "coldrawdb_query_all_metadata_{}.sqlite",
            uuid::Uuid::new_v4()
        ));
        std::fs::File::create(&db_path).unwrap();
        let db = Database::connect(format!("sqlite://{}?", db_path.display()))
            .await
            .unwrap();
        crate::init::init_table("init.sql", &db).await.unwrap();
        crate::init::apply_migrations("migrations", &db).await.unwrap();

        for (id, deleted) in [("d-current", 0), ("d-deleted", 1)] {
            sea_orm::ConnectionTrait::execute(
                &db,
                sea_orm::Statement::from_sql_and_values(
                    sea_orm::DbBackend::Sqlite,
                    "INSERT INTO diagram(id, name, last_modified, revision, updated_at, is_deleted) VALUES(?, ?, ?, ?, ?, ?)",
                    vec![
                        id.into(),
                        "同名图".into(),
                        "2020-01-01T00:00:00Z".into(),
                        8.into(),
                        "2026-10-29T12:34:56Z".into(),
                        deleted.into(),
                    ],
                ),
            )
            .await
            .unwrap();
        }

        let vos = list_active_diagrams(&db).await.unwrap();
        assert_eq!(vos.len(), 1);
        assert_eq!(vos[0].id, "d-current");
        assert_eq!(vos[0].revision, 8);
        assert_eq!(vos[0].updated_at.as_deref(), Some("2026-10-29T12:34:56Z"));
        assert_eq!(vos[0].last_modified.as_deref(), Some("2020-01-01T00:00:00Z"));
        let serialized = serde_json::to_value(&vos[0]).unwrap();
        assert_eq!(serialized["lastModified"], "2020-01-01T00:00:00Z");
        assert_eq!(serialized["updated_at"], "2026-10-29T12:34:56Z");

        db.close().await.unwrap();
        std::fs::remove_file(&db_path).unwrap();
        crate::verify_reporter::report_pass("UT-S04-20", 0);
    }

    /// UT-S04-21：queryAll 在同一条摘要查询中聚合真实表/关系数。
    #[actix_web::test]
    async fn ut_s04_21_query_all_aggregates_content_counts() {
        let db_path = std::env::temp_dir().join(format!(
            "coldrawdb_query_all_counts_{}.sqlite",
            uuid::Uuid::new_v4()
        ));
        std::fs::File::create(&db_path).unwrap();
        let db = Database::connect(format!("sqlite://{}?", db_path.display()))
            .await
            .unwrap();
        crate::init::init_table("init.sql", &db).await.unwrap();
        crate::init::apply_migrations("migrations", &db).await.unwrap();

        let statements = [
            "INSERT INTO diagram(id, name, last_modified, revision, updated_at, is_deleted) VALUES('d-full', '完整模型', '2020-01-01T00:00:00Z', 8, '2026-10-29T12:34:56Z', 0)",
            "INSERT INTO diagram(id, name, is_deleted) VALUES('d-empty', '空模型', 0)",
            "INSERT INTO diagram(id, name, is_deleted) VALUES('d-deleted', '已删模型', 1)",
            "INSERT INTO \"table\"(id, name, is_deleted) VALUES('t-1', 'users', 0)",
            "INSERT INTO \"table\"(id, name, is_deleted) VALUES('t-2', 'orders', 0)",
            "INSERT INTO \"table\"(id, name, is_deleted) VALUES('t-deleted', 'ignored', 1)",
            "INSERT INTO \"table\"(id, name, is_deleted) VALUES('t-deleted-diagram', 'ignored', 0)",
            "INSERT INTO diagram_link(id, diagram_id, table_id) VALUES('dl-t-1', 'd-full', 't-1')",
            "INSERT INTO diagram_link(id, diagram_id, table_id) VALUES('dl-t-2', 'd-full', 't-2')",
            "INSERT INTO diagram_link(id, diagram_id, table_id) VALUES('dl-t-deleted', 'd-full', 't-deleted')",
            "INSERT INTO diagram_link(id, diagram_id, table_id) VALUES('dl-t-deleted-diagram', 'd-deleted', 't-deleted-diagram')",
            "INSERT INTO \"reference\"(id, name, is_deleted) VALUES('r-1', 'orders_user', 0)",
            "INSERT INTO \"reference\"(id, name, is_deleted) VALUES('r-deleted', 'ignored', 1)",
            "INSERT INTO \"reference\"(id, name, is_deleted) VALUES('r-deleted-diagram', 'ignored', 0)",
            "INSERT INTO diagram_link(id, diagram_id, reference_id) VALUES('dl-r-1', 'd-full', 'r-1')",
            "INSERT INTO diagram_link(id, diagram_id, reference_id) VALUES('dl-r-deleted', 'd-full', 'r-deleted')",
            "INSERT INTO diagram_link(id, diagram_id, reference_id) VALUES('dl-r-deleted-diagram', 'd-deleted', 'r-deleted-diagram')",
        ];
        for sql in statements {
            sea_orm::ConnectionTrait::execute(
                &db,
                sea_orm::Statement::from_sql_and_values(
                    sea_orm::DbBackend::Sqlite,
                    sql,
                    vec![],
                ),
            )
            .await
            .unwrap();
        }

        let vos = list_active_diagrams(&db).await.unwrap();
        assert_eq!(vos.len(), 2, "UT-S04-21: 软删除图及其内容不得进入摘要");
        let full = vos.iter().find(|diagram| diagram.id == "d-full").unwrap();
        assert_eq!(full.table_count, 2);
        assert_eq!(full.reference_count, 1);
        assert_eq!(full.revision, 8);
        assert_eq!(full.updated_at.as_deref(), Some("2026-10-29T12:34:56Z"));
        assert_eq!(full.last_modified.as_deref(), Some("2020-01-01T00:00:00Z"));
        let empty = vos.iter().find(|diagram| diagram.id == "d-empty").unwrap();
        assert_eq!(empty.table_count, 0);
        assert_eq!(empty.reference_count, 0);
        let serialized = serde_json::to_value(full).unwrap();
        assert_eq!(serialized["table_count"], 2);
        assert_eq!(serialized["reference_count"], 1);
        assert_eq!(serialized["lastModified"], "2020-01-01T00:00:00Z");

        db.close().await.unwrap();
        std::fs::remove_file(&db_path).unwrap();
        crate::verify_reporter::report_pass("UT-S04-21", 0);
    }
}
