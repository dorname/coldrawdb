pub mod diagram;
pub mod task;
mod common;

use diagram::*;
use serde_json::Value;
use task::*;
pub use common::*;
use rusqlite::ToSql;
use crate::db::DB;

const DIAGRAM_TABLE_NAME: &str = "diagram";
const NOTE_TABLE_NAME: &str = "note";
const TASK_TABLE_NAME: &str = "task";
const AREA_TABLE_NAME: &str = "area";
const TYPE_TABLE_NAME: &str = "type";
const ENUM_TABLE_NAME: &str = "enum";

pub enum TableType {
    Diagram,
    Note,
    Task,
    Area,
    Type,
    Enum,
}

impl TableType {
    pub fn get_table_name(&self) -> &str {
        match self {
            TableType::Diagram => DIAGRAM_TABLE_NAME,
            TableType::Note => NOTE_TABLE_NAME,
            TableType::Task => TASK_TABLE_NAME,
            TableType::Area => AREA_TABLE_NAME,
            TableType::Type => TYPE_TABLE_NAME,
            TableType::Enum => ENUM_TABLE_NAME,
        }
    }
    pub fn from_num(num: i64) -> TableType {
        match num {
            0 => TableType::Diagram,
            1 => TableType::Note,
            2 => TableType::Task,
            3 => TableType::Area,
            4 => TableType::Type,
            5 => TableType::Enum,
            _ => panic!("Invalid table type"),
        }
    }
}
// 创建表
pub fn create_diagram_table(db: &DB)->Result<(), rusqlite::Error>{
    let sql = format!("CREATE TABLE IF NOT EXISTS {} (id INTEGER PRIMARY KEY AUTOINCREMENT, last_modified INTEGER, loaded_from_gist_id INTEGER)", DIAGRAM_TABLE_NAME);
    db.execute_sync(&sql, [])?;
    Ok(())
}

pub fn create_note_table(db: &DB)->Result<(), rusqlite::Error>{
    let sql = format!("CREATE TABLE IF NOT EXISTS {} (id INTEGER PRIMARY KEY AUTOINCREMENT, last_modified INTEGER, loaded_from_gist_id INTEGER)", NOTE_TABLE_NAME);
    db.execute_sync(&sql, [])?;
    Ok(())
}

pub fn create_task_table(db: &DB)->Result<(), rusqlite::Error>{
    let sql = format!("CREATE TABLE IF NOT EXISTS {} (id INTEGER PRIMARY KEY AUTOINCREMENT, 
    compele BOOLEAN, 
    details STRING,
    task_order INTEGER,
    priority INTEGER,
    title STRING)", TASK_TABLE_NAME);
    db.execute_sync(&sql, [])?;
    Ok(())
}

pub fn create_area_table(db: &DB)->Result<(), rusqlite::Error>{
    let sql = format!("CREATE TABLE IF NOT EXISTS {} (id INTEGER PRIMARY KEY AUTOINCREMENT, last_modified INTEGER, loaded_from_gist_id INTEGER)", AREA_TABLE_NAME);
    db.execute_sync(&sql, [])?;
    Ok(())
}

pub fn create_type_table(db: &DB)->Result<(), rusqlite::Error>{
    let sql = format!("CREATE TABLE IF NOT EXISTS {} (id INTEGER PRIMARY KEY AUTOINCREMENT, last_modified INTEGER, loaded_from_gist_id INTEGER)", TYPE_TABLE_NAME);
    db.execute_sync(&sql, [])?;
    Ok(())
}

pub fn create_enum_table(db: &DB)->Result<(), rusqlite::Error>{
    let sql = format!("CREATE TABLE IF NOT EXISTS {} (id INTEGER PRIMARY KEY AUTOINCREMENT, last_modified INTEGER, loaded_from_gist_id INTEGER)", ENUM_TABLE_NAME);
    db.execute_sync(&sql, [])?;
    Ok(())
}

/// 初始化diagram
pub fn init_diagram(id: i64, last_modified: i64, loaded_from_gist_id: Option<i64>) -> Diagram {
    Diagram::new(id, last_modified, loaded_from_gist_id)
} 

/// 初始化task
pub fn init_task(id: i64, compele: bool, details: String, order: i64, priority: i64, title: String) -> Task {
    Task::new(id, compele, details, order, priority, title)
}


// select
pub async fn query<M>(
    db: &DB,
    where_clause: &str,
    params: Value,
    model: M,
    table_type: TableType,
) -> Result<Vec<M>, rusqlite::Error>
where
    M: BusinessModel + Send + Sync + 'static
{
    // 先根据 model.get_columns() 和 table_type 生成 SELECT 语句
    let common_model = CommonModel::new(
        table_type.get_table_name().to_string(),
        model.get_columns(),
        model.get_values(),
        where_clause.to_string(),
        model.get_order_by(),
    );
    
    let rows: Vec<M> = db
        .with_connection(move |conn| {
            let mut stmt = conn.prepare(common_model.get_select_sql().as_str())?;
            let params_temp = params.clone();
            let params_boxed = json_value_to_params(&params_temp).map_err(|e| rusqlite::Error::ToSqlConversionFailure(e.into()))?;
            let params_refs: Vec<&dyn ToSql> = params_boxed.iter().map(|b| b.as_ref() as &dyn ToSql).collect();
            // 这里直接调用 trait 里的 from_raw
            let iter = stmt.query_map(&params_refs[..], |row| Ok(M::from_raw(row)))?;
            Ok(iter.collect::<Result<Vec<_>, _>>()?)
        })
        .await?;

    Ok(rows)
}

// insert
pub async fn insert(db: &DB, model: impl BusinessModel, table_type: TableType) -> Result<(), rusqlite::Error> {
    let common_model = model.to_common_model(table_type.get_table_name().to_string(),"".to_string());
    db.execute(common_model.get_insert_sql().as_str(), [])
        .await?;
    Ok(())
}

// update
pub async fn update(db: &DB, model: impl BusinessModel, table_type: TableType,where_clause: String) -> Result<(), rusqlite::Error> {
    let common_model = model.to_common_model(table_type.get_table_name().to_string(),where_clause);
    db.execute(common_model.get_update_sql().as_str(), [])
        .await?;
    Ok(())
}

// delete
pub async fn delete(db: &DB, model: impl BusinessModel, table_type: TableType,where_clause: String) -> Result<(), rusqlite::Error> {
    let common_model = model.to_common_model(table_type.get_table_name().to_string(),where_clause);
    db.execute(common_model.get_delete_sql().as_str(), [])
        .await?;
    Ok(())
}




fn json_value_to_params(value: &serde_json::Value) -> Result<Vec<Box<dyn ToSql>>, String> {
    match value {
        serde_json::Value::Array(arr) => {
            let mut params: Vec<Box<dyn ToSql>> = Vec::new();
            for v in arr {
                match v {
                    serde_json::Value::String(s) => params.push(Box::new(s.clone())),
                    serde_json::Value::Number(n) => {
                        if let Some(i) = n.as_i64() {
                            params.push(Box::new(i));
                        } else if let Some(f) = n.as_f64() {
                            params.push(Box::new(f));
                        } else {
                            return Err("不支持的数字类型".to_string());
                        }
                    }
                    serde_json::Value::Bool(b) => params.push(Box::new(*b as i32)),
                    serde_json::Value::Null => params.push(Box::new(None::<i32>)),
                    _ => return Err("不支持的参数类型".to_string()),
                }
            }
            Ok(params)
        }
        _ => Err("params 必须是数组".to_string()),
    }
}


