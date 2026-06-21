//! 完整 diagram 实体持久化（tables / fields / references / areas / notes）
//! 供 `diagrams_v1` GET/PUT 使用。

use sea_orm::{ConnectionTrait, DatabaseBackend, Statement, TransactionTrait};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::error::DrawDBError;
use crate::next_id;

fn esc(s: &str) -> String {
    s.replace('\'', "''")
}

fn sql_opt_str(v: Option<&str>) -> String {
    v.map(|s| format!("'{}'", esc(s)))
        .unwrap_or_else(|| "NULL".to_string())
}

fn sql_bool(v: bool) -> String {
    if v {
        "1".into()
    } else {
        "0".into()
    }
}

fn sql_num(v: f64) -> String {
    format!("{}", v)
}

fn row_str(row: &sea_orm::QueryResult, col: &str) -> Option<String> {
    row.try_get("", col).ok()
}

fn row_f64(row: &sea_orm::QueryResult, col: &str) -> f64 {
    row_str(row, col)
        .and_then(|s| s.parse().ok())
        .unwrap_or(0.0)
}

fn row_bool(row: &sea_orm::QueryResult, col: &str) -> bool {
    row.try_get::<bool>("", col).unwrap_or(false)
        || row_str(row, col).as_deref() == Some("1")
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct FieldDto {
    pub id: String,
    pub name: String,
    #[serde(default, alias = "type")]
    pub type_: String,
    #[serde(default)]
    pub default: String,
    #[serde(default)]
    pub check: String,
    #[serde(default)]
    pub primary: bool,
    #[serde(default)]
    pub unique: bool,
    #[serde(default)]
    pub not_null: bool,
    #[serde(default)]
    pub increment: bool,
    #[serde(default)]
    pub comment: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct TableDto {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub x: f64,
    #[serde(default)]
    pub y: f64,
    #[serde(default)]
    pub color: String,
    #[serde(default)]
    pub comment: String,
    #[serde(default)]
    pub fields: Vec<FieldDto>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct ReferenceDto {
    pub id: String,
    #[serde(default)]
    pub name: String,
    pub start_table_id: String,
    pub end_table_id: String,
    pub start_field_id: String,
    pub end_field_id: String,
    #[serde(default, alias = "type")]
    pub type_: String,
    #[serde(default)]
    pub on_delete: String,
    #[serde(default)]
    pub on_update: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct AreaDto {
    pub id: String,
    #[serde(default)]
    pub x: f64,
    #[serde(default)]
    pub y: f64,
    #[serde(default)]
    pub width: f64,
    #[serde(default)]
    pub height: f64,
    #[serde(default)]
    pub color: String,
    #[serde(default)]
    pub name: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct NoteDto {
    pub id: String,
    #[serde(default)]
    pub x: f64,
    #[serde(default)]
    pub y: f64,
    #[serde(default)]
    pub content: String,
    #[serde(default)]
    pub color: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct DiagramFull {
    pub id: String,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub database: Option<String>,
    #[serde(default)]
    pub pan: Option<String>,
    #[serde(default)]
    pub zoom: Option<String>,
    #[serde(default)]
    pub revision: i64,
    #[serde(default)]
    pub tables: Vec<TableDto>,
    #[serde(default)]
    pub references: Vec<ReferenceDto>,
    #[serde(default)]
    pub areas: Vec<AreaDto>,
    #[serde(default)]
    pub notes: Vec<NoteDto>,
}

pub async fn load_diagram<C: ConnectionTrait>(conn: &C, diagram_id: &str) -> Result<Option<DiagramFull>, DrawDBError> {
    let q = format!(
        "SELECT id, name, database, pan, zoom, revision FROM diagram WHERE id='{}' AND (is_deleted=0 OR is_deleted IS NULL) LIMIT 1",
        esc(diagram_id)
    );
    let row = conn
        .query_one(Statement::from_sql_and_values(DatabaseBackend::Sqlite, q, vec![]))
        .await?;
    let Some(row) = row else {
        return Ok(None);
    };

    let mut diagram = DiagramFull {
        id: row.try_get("", "id")?,
        name: row.try_get("", "name").ok(),
        database: row.try_get("", "database").ok(),
        pan: row.try_get("", "pan").ok(),
        zoom: row.try_get("", "zoom").ok(),
        revision: row.try_get("", "revision").unwrap_or(0),
        tables: Vec::new(),
        references: Vec::new(),
        areas: Vec::new(),
        notes: Vec::new(),
    };

    let links_q = format!(
        "SELECT table_id, reference_id, area_id, note_id FROM diagram_link WHERE diagram_id='{}'",
        esc(diagram_id)
    );
    let links = conn
        .query_all(Statement::from_sql_and_values(DatabaseBackend::Sqlite, links_q, vec![]))
        .await?;

    for link in links {
        if let Some(tid) = row_str(&link, "table_id") {
            if diagram.tables.iter().any(|t| t.id == tid) {
                continue;
            }
            if let Some(table) = load_table(conn, &tid).await? {
                diagram.tables.push(table);
            }
        }
        if let Some(rid) = row_str(&link, "reference_id") {
            if diagram.references.iter().any(|r| r.id == rid) {
                continue;
            }
            if let Some(reference) = load_reference(conn, &rid).await? {
                diagram.references.push(reference);
            }
        }
        if let Some(aid) = row_str(&link, "area_id") {
            if diagram.areas.iter().any(|a| a.id == aid) {
                continue;
            }
            if let Some(area) = load_area(conn, &aid).await? {
                diagram.areas.push(area);
            }
        }
        if let Some(nid) = row_str(&link, "note_id") {
            if diagram.notes.iter().any(|n| n.id == nid) {
                continue;
            }
            if let Some(note) = load_note(conn, &nid).await? {
                diagram.notes.push(note);
            }
        }
    }

    Ok(Some(diagram))
}

async fn load_table<C: ConnectionTrait>(conn: &C, table_id: &str) -> Result<Option<TableDto>, DrawDBError> {
    let q = format!(
        "SELECT id, name, x, y, color, comment FROM \"table\" WHERE id='{}' AND (is_deleted=0 OR is_deleted IS NULL) LIMIT 1",
        esc(table_id)
    );
    let row = conn
        .query_one(Statement::from_sql_and_values(DatabaseBackend::Sqlite, q, vec![]))
        .await?;
    let Some(row) = row else {
        return Ok(None);
    };

    let fields_q = format!(
        "SELECT f.id, f.name, f.type, f.\"default\", f.\"check\", f.\"primary\", f.\"unique\", f.not_null, f.increment, f.comment \
         FROM field f INNER JOIN table_link tl ON tl.field_id = f.id \
         WHERE tl.table_id='{}' AND (f.is_deleted=0 OR f.is_deleted IS NULL) \
         ORDER BY tl.order_no, f.name",
        esc(table_id)
    );
    let field_rows = conn
        .query_all(Statement::from_sql_and_values(DatabaseBackend::Sqlite, fields_q, vec![]))
        .await?;

    let fields = field_rows
        .into_iter()
        .map(|fr| FieldDto {
            id: fr.try_get("", "id").unwrap_or_default(),
            name: row_str(&fr, "name").unwrap_or_default(),
            type_: row_str(&fr, "type").unwrap_or_default(),
            default: row_str(&fr, "default").unwrap_or_default(),
            check: row_str(&fr, "check").unwrap_or_default(),
            primary: row_bool(&fr, "primary"),
            unique: row_bool(&fr, "unique"),
            not_null: row_bool(&fr, "not_null"),
            increment: row_bool(&fr, "increment"),
            comment: row_str(&fr, "comment").unwrap_or_default(),
        })
        .collect();

    Ok(Some(TableDto {
        id: row.try_get("", "id")?,
        name: row_str(&row, "name").unwrap_or_default(),
        x: row_f64(&row, "x"),
        y: row_f64(&row, "y"),
        color: row_str(&row, "color").unwrap_or_default(),
        comment: row_str(&row, "comment").unwrap_or_default(),
        fields,
    }))
}

async fn load_reference<C: ConnectionTrait>(conn: &C, ref_id: &str) -> Result<Option<ReferenceDto>, DrawDBError> {
    let q = format!(
        "SELECT id, name, cardinality, deleteConstraint, updateConstraint, startFieldId, endFieldId, startTableId, endTableId \
         FROM reference WHERE id='{}' AND (is_deleted=0 OR is_deleted IS NULL) LIMIT 1",
        esc(ref_id)
    );
    let row = conn
        .query_one(Statement::from_sql_and_values(DatabaseBackend::Sqlite, q, vec![]))
        .await?;
    let Some(row) = row else {
        return Ok(None);
    };
    Ok(Some(ReferenceDto {
        id: row.try_get("", "id")?,
        name: row_str(&row, "name").unwrap_or_default(),
        start_table_id: row_str(&row, "startTableId").unwrap_or_default(),
        end_table_id: row_str(&row, "endTableId").unwrap_or_default(),
        start_field_id: row_str(&row, "startFieldId").unwrap_or_default(),
        end_field_id: row_str(&row, "endFieldId").unwrap_or_default(),
        type_: row_str(&row, "cardinality").unwrap_or_else(|| "one_to_many".into()),
        on_delete: row_str(&row, "deleteConstraint").unwrap_or_else(|| "RESTRICT".into()),
        on_update: row_str(&row, "updateConstraint").unwrap_or_else(|| "RESTRICT".into()),
    }))
}

async fn load_area<C: ConnectionTrait>(conn: &C, area_id: &str) -> Result<Option<AreaDto>, DrawDBError> {
    let q = format!(
        "SELECT id, name, x, y, width, height, color FROM area WHERE id='{}' AND (is_deleted=0 OR is_deleted IS NULL) LIMIT 1",
        esc(area_id)
    );
    let row = conn
        .query_one(Statement::from_sql_and_values(DatabaseBackend::Sqlite, q, vec![]))
        .await?;
    let Some(row) = row else {
        return Ok(None);
    };
    Ok(Some(AreaDto {
        id: row.try_get("", "id")?,
        name: row_str(&row, "name").unwrap_or_default(),
        x: row_f64(&row, "x"),
        y: row_f64(&row, "y"),
        width: row_f64(&row, "width"),
        height: row_f64(&row, "height"),
        color: row_str(&row, "color").unwrap_or_default(),
    }))
}

async fn load_note<C: ConnectionTrait>(conn: &C, note_id: &str) -> Result<Option<NoteDto>, DrawDBError> {
    let q = format!(
        "SELECT id, x, y, content, color, title FROM note WHERE id='{}' AND (is_deleted=0 OR is_deleted IS NULL) LIMIT 1",
        esc(note_id)
    );
    let row = conn
        .query_one(Statement::from_sql_and_values(DatabaseBackend::Sqlite, q, vec![]))
        .await?;
    let Some(row) = row else {
        return Ok(None);
    };
    let content = row_str(&row, "content")
        .or_else(|| row_str(&row, "title"))
        .unwrap_or_default();
    Ok(Some(NoteDto {
        id: row.try_get("", "id")?,
        x: row_f64(&row, "x"),
        y: row_f64(&row, "y"),
        content,
        color: row_str(&row, "color").unwrap_or_default(),
    }))
}

async fn purge_diagram_entities<C: ConnectionTrait>(conn: &C, diagram_id: &str) -> Result<(), sea_orm::DbErr> {
    let links_q = format!(
        "SELECT table_id, reference_id, area_id, note_id FROM diagram_link WHERE diagram_id='{}'",
        esc(diagram_id)
    );
    let links = conn
        .query_all(Statement::from_sql_and_values(DatabaseBackend::Sqlite, links_q, vec![]))
        .await?;

    for link in &links {
        if let Some(tid) = row_str(link, "table_id") {
            let fq = format!(
                "SELECT field_id FROM table_link WHERE table_id='{}'",
                esc(&tid)
            );
            if let Ok(field_rows) = conn
                .query_all(Statement::from_sql_and_values(DatabaseBackend::Sqlite, fq, vec![]))
                .await
            {
                for fr in field_rows {
                    if let Some(fid) = row_str(&fr, "field_id") {
                        let _ = conn
                            .execute(Statement::from_sql_and_values(
                                DatabaseBackend::Sqlite,
                                format!("DELETE FROM field WHERE id='{}'", esc(&fid)),
                                vec![],
                            ))
                            .await;
                    }
                }
            }
            let _ = conn
                .execute(Statement::from_sql_and_values(
                    DatabaseBackend::Sqlite,
                    format!("DELETE FROM table_link WHERE table_id='{}'", esc(&tid)),
                    vec![],
                ))
                .await;
            let _ = conn
                .execute(Statement::from_sql_and_values(
                    DatabaseBackend::Sqlite,
                    format!("DELETE FROM \"table\" WHERE id='{}'", esc(&tid)),
                    vec![],
                ))
                .await;
        }
        if let Some(rid) = row_str(link, "reference_id") {
            let _ = conn
                .execute(Statement::from_sql_and_values(
                    DatabaseBackend::Sqlite,
                    format!("DELETE FROM reference WHERE id='{}'", esc(&rid)),
                    vec![],
                ))
                .await;
        }
        if let Some(aid) = row_str(link, "area_id") {
            let _ = conn
                .execute(Statement::from_sql_and_values(
                    DatabaseBackend::Sqlite,
                    format!("DELETE FROM area WHERE id='{}'", esc(&aid)),
                    vec![],
                ))
                .await;
        }
        if let Some(nid) = row_str(link, "note_id") {
            let _ = conn
                .execute(Statement::from_sql_and_values(
                    DatabaseBackend::Sqlite,
                    format!("DELETE FROM note WHERE id='{}'", esc(&nid)),
                    vec![],
                ))
                .await;
        }
    }

    let _ = conn
        .execute(Statement::from_sql_and_values(
            DatabaseBackend::Sqlite,
            format!("DELETE FROM diagram_link WHERE diagram_id='{}'", esc(diagram_id)),
            vec![],
        ))
        .await;
    Ok(())
}

pub async fn save_diagram<C: ConnectionTrait + TransactionTrait>(
    conn: &C,
    diagram_id: &str,
    expected_revision: i64,
    diagram: &DiagramFull,
) -> Result<i64, SaveDiagramError> {
    if diagram.id != diagram_id {
        return Err(SaveDiagramError::BadRequest("path id and body id mismatch".into()));
    }

    let tx = conn.begin().await?;
    let q = format!(
        "SELECT revision FROM diagram WHERE id='{}' AND (is_deleted=0 OR is_deleted IS NULL) LIMIT 1",
        esc(diagram_id)
    );
    let row = tx
        .query_one(Statement::from_sql_and_values(DatabaseBackend::Sqlite, q, vec![]))
        .await?;
    let Some(row) = row else {
        return Err(SaveDiagramError::NotFound);
    };
    let cur: i64 = row.try_get("", "revision").unwrap_or(0);
    if cur != expected_revision {
        return Err(SaveDiagramError::Conflict { current_revision: cur });
    }

    purge_diagram_entities(&tx, diagram_id).await?;

    let up = format!(
        "UPDATE diagram SET name={}, database={}, pan={}, zoom={}, revision=revision+1, updated_at=datetime('now') WHERE id='{}'",
        sql_opt_str(diagram.name.as_deref()),
        sql_opt_str(diagram.database.as_deref()),
        sql_opt_str(diagram.pan.as_deref()),
        sql_opt_str(diagram.zoom.as_deref()),
        esc(diagram_id)
    );
    tx.execute(Statement::from_sql_and_values(DatabaseBackend::Sqlite, up, vec![]))
        .await?;

    for (ti, table) in diagram.tables.iter().enumerate() {
        let ins_t = format!(
            "INSERT INTO \"table\"(id, name, x, y, color, comment, locked, is_deleted) VALUES('{}','{}',{},{},{},{},0,0)",
            esc(&table.id),
            esc(&table.name),
            sql_num(table.x),
            sql_num(table.y),
            sql_opt_str(Some(table.color.as_str())),
            sql_opt_str(Some(table.comment.as_str())),
        );
        tx.execute(Statement::from_sql_and_values(DatabaseBackend::Sqlite, ins_t, vec![]))
            .await?;
        let ins_dl = format!(
            "INSERT INTO diagram_link(id, diagram_id, table_id) VALUES('{}','{}','{}')",
            esc(&next_id()),
            esc(diagram_id),
            esc(&table.id),
        );
        tx.execute(Statement::from_sql_and_values(DatabaseBackend::Sqlite, ins_dl, vec![]))
            .await?;

        for (fi, field) in table.fields.iter().enumerate() {
            let ins_f = format!(
                "INSERT INTO field(id, name, type, \"default\", \"check\", \"primary\", \"unique\", not_null, increment, comment, is_deleted) \
                 VALUES('{}','{}','{}',{},{},{},{},{},{},{},0)",
                esc(&field.id),
                esc(&field.name),
                esc(&field.type_),
                sql_opt_str(Some(field.default.as_str())),
                sql_opt_str(Some(field.check.as_str())),
                sql_bool(field.primary),
                sql_bool(field.unique),
                sql_bool(field.not_null),
                sql_bool(field.increment),
                sql_opt_str(Some(field.comment.as_str())),
            );
            tx.execute(Statement::from_sql_and_values(DatabaseBackend::Sqlite, ins_f, vec![]))
                .await?;
            let ins_tl = format!(
                "INSERT INTO table_link(id, table_id, field_id, order_no) VALUES('{}','{}','{}',{})",
                esc(&next_id()),
                esc(&table.id),
                esc(&field.id),
                fi as i64,
            );
            tx.execute(Statement::from_sql_and_values(DatabaseBackend::Sqlite, ins_tl, vec![]))
                .await?;
        }
        let _ = ti;
    }

    for reference in &diagram.references {
        let ins_r = format!(
            "INSERT INTO reference(id, name, cardinality, deleteConstraint, updateConstraint, startFieldId, endFieldId, startTableId, endTableId, is_deleted) \
             VALUES('{}',{},'{}','{}','{}','{}','{}','{}','{}',0)",
            esc(&reference.id),
            sql_opt_str(Some(reference.name.as_str())),
            esc(if reference.type_.is_empty() { "one_to_many" } else { &reference.type_ }),
            esc(if reference.on_delete.is_empty() { "RESTRICT" } else { &reference.on_delete }),
            esc(if reference.on_update.is_empty() { "RESTRICT" } else { &reference.on_update }),
            esc(&reference.start_field_id),
            esc(&reference.end_field_id),
            esc(&reference.start_table_id),
            esc(&reference.end_table_id),
        );
        tx.execute(Statement::from_sql_and_values(DatabaseBackend::Sqlite, ins_r, vec![]))
            .await?;
        let ins_dl = format!(
            "INSERT INTO diagram_link(id, diagram_id, reference_id) VALUES('{}','{}','{}')",
            esc(&next_id()),
            esc(diagram_id),
            esc(&reference.id),
        );
        tx.execute(Statement::from_sql_and_values(DatabaseBackend::Sqlite, ins_dl, vec![]))
            .await?;
    }

    for area in &diagram.areas {
        let ins_a = format!(
            "INSERT INTO area(id, name, x, y, width, height, color, is_deleted) VALUES('{}','{}',{},{},{},{},{},0)",
            esc(&area.id),
            esc(&area.name),
            sql_num(area.x),
            sql_num(area.y),
            sql_num(area.width),
            sql_num(area.height),
            sql_opt_str(Some(area.color.as_str())),
        );
        tx.execute(Statement::from_sql_and_values(DatabaseBackend::Sqlite, ins_a, vec![]))
            .await?;
        let ins_dl = format!(
            "INSERT INTO diagram_link(id, diagram_id, area_id) VALUES('{}','{}','{}')",
            esc(&next_id()),
            esc(diagram_id),
            esc(&area.id),
        );
        tx.execute(Statement::from_sql_and_values(DatabaseBackend::Sqlite, ins_dl, vec![]))
            .await?;
    }

    for note in &diagram.notes {
        let ins_n = format!(
            "INSERT INTO note(id, x, y, content, color, title, is_deleted) VALUES('{}',{},{},{},{},NULL,0)",
            esc(&note.id),
            sql_num(note.x),
            sql_num(note.y),
            sql_opt_str(Some(note.content.as_str())),
            sql_opt_str(Some(note.color.as_str())),
        );
        tx.execute(Statement::from_sql_and_values(DatabaseBackend::Sqlite, ins_n, vec![]))
            .await?;
        let ins_dl = format!(
            "INSERT INTO diagram_link(id, diagram_id, note_id) VALUES('{}','{}','{}')",
            esc(&next_id()),
            esc(diagram_id),
            esc(&note.id),
        );
        tx.execute(Statement::from_sql_and_values(DatabaseBackend::Sqlite, ins_dl, vec![]))
            .await?;
    }

    let new_rev = cur + 1;
    tx.commit().await?;
    Ok(new_rev)
}

/// 从 bridge / import API 的 JSON payload 构建完整 diagram（用于首次 persist）。
pub fn diagram_from_import_payload(diagram_id: &str, payload: &Value) -> DiagramFull {
    let name = payload
        .get("name")
        .and_then(|v| v.as_str())
        .map(String::from);
    let tables: Vec<TableDto> = payload
        .get("tables")
        .and_then(|v| serde_json::from_value(v.clone()).ok())
        .unwrap_or_default();
    let references: Vec<ReferenceDto> = payload
        .get("references")
        .and_then(|v| serde_json::from_value(v.clone()).ok())
        .unwrap_or_default();
    let areas: Vec<AreaDto> = payload
        .get("areas")
        .and_then(|v| serde_json::from_value(v.clone()).ok())
        .unwrap_or_default();
    let notes: Vec<NoteDto> = payload
        .get("notes")
        .and_then(|v| serde_json::from_value(v.clone()).ok())
        .unwrap_or_default();
    DiagramFull {
        id: diagram_id.to_string(),
        name,
        database: payload
            .get("database")
            .and_then(|v| v.as_str())
            .map(String::from),
        pan: None,
        zoom: None,
        revision: 0,
        tables,
        references,
        areas,
        notes,
    }
}

/// 创建空 diagram 后写入 import payload 中的嵌套实体。
pub async fn persist_import_payload<C: ConnectionTrait + TransactionTrait>(
    conn: &C,
    diagram_id: &str,
    payload: &Value,
) -> Result<i64, SaveDiagramError> {
    let diagram = diagram_from_import_payload(diagram_id, payload);
    save_diagram(conn, diagram_id, 0, &diagram).await
}

#[derive(Debug)]
pub enum SaveDiagramError {
    NotFound,
    BadRequest(String),
    Conflict { current_revision: i64 },
    Db(sea_orm::DbErr),
}

impl From<sea_orm::DbErr> for SaveDiagramError {
    fn from(e: sea_orm::DbErr) -> Self {
        SaveDiagramError::Db(e)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sea_orm::Database;
    use crate::init::{apply_migrations, init_table};

    async fn build_db() -> sea_orm::DatabaseConnection {
        let db_path = format!(
            "{}/drawdb_persist_{}.sqlite",
            std::env::temp_dir().display(),
            uuid::Uuid::new_v4()
        );
        if std::path::Path::new(&db_path).exists() {
            let _ = std::fs::remove_file(&db_path);
        }
        std::fs::File::create(&db_path).unwrap();
        let db = Database::connect(format!("sqlite://{}?", db_path))
            .await
            .unwrap();
        init_table("init.sql", &db).await.unwrap();
        apply_migrations("migrations", &db).await.unwrap();
        db
    }

    #[tokio::test]
    async fn round_trip_table_and_field() {
        let db = build_db().await;
        let id = next_id();
        let sql = format!(
            "INSERT INTO diagram(id, name, database, pan, zoom, revision, updated_at, is_deleted) VALUES('{}','test',NULL,'','',0,datetime('now'),0)",
            esc(&id)
        );
        db.execute(Statement::from_sql_and_values(DatabaseBackend::Sqlite, sql, vec![]))
            .await
            .unwrap();

        let diagram = DiagramFull {
            id: id.clone(),
            name: Some("test".into()),
            database: None,
            pan: Some("".into()),
            zoom: Some("".into()),
            revision: 0,
            tables: vec![TableDto {
                id: "t1".into(),
                name: "users".into(),
                x: 100.0,
                y: 200.0,
                color: String::new(),
                comment: String::new(),
                fields: vec![FieldDto {
                    id: "f1".into(),
                    name: "id".into(),
                    type_: "INT".into(),
                    default: String::new(),
                    check: String::new(),
                    primary: true,
                    unique: false,
                    not_null: true,
                    increment: true,
                    comment: String::new(),
                }],
            }],
            references: vec![],
            areas: vec![],
            notes: vec![],
        };

        let new_rev = save_diagram(&db, &id, 0, &diagram).await.unwrap();
        assert_eq!(new_rev, 1);

        let loaded = load_diagram(&db, &id).await.unwrap().unwrap();
        assert_eq!(loaded.revision, 1);
        assert_eq!(loaded.tables.len(), 1);
        assert_eq!(loaded.tables[0].name, "users");
        assert_eq!(loaded.tables[0].fields.len(), 1);
        assert_eq!(loaded.tables[0].fields[0].name, "id");
    }
}
