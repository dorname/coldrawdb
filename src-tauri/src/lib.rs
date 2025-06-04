use db::models::diagram::Diagram;
use db::{models, DB};
use tauri::State;
mod db;
use db::models::task::Task;
use db::models::TableType;
use serde_json::Value;


#[tauri::command]
async fn insert_diagram(state: State<'_, DB>, diagram: Diagram) -> Result<i64, String> {
    let conn = state;
    models::insert(&conn, diagram, TableType::from_num(0))
        .await
        .map_err(|e| e.to_string())?;
    Ok(0)
}

#[tauri::command]
async fn insert_task(state: State<'_, DB>, task: Task) -> Result<i64, String> {
    let conn = state;
    models::insert(&conn, task, TableType::from_num(2))
        .await
        .map_err(|e| e.to_string())?;
    Ok(0)
}

#[tauri::command]
async fn query_diagram(
    state: State<'_, DB>,
    where_clause: String,
    params: Value,
    diagram: Diagram,
) -> Result<Vec<Diagram>, String> {
    let conn = state;
    let result = models::query(
        &conn,
        &where_clause,
        params,
        diagram,
        TableType::from_num(1),
    )
    .await
    .map_err(|e| e.to_string())?;
    Ok(result)
}

#[tauri::command]
async fn query_task(
    state: State<'_, DB>,
    where_clause: String,
    params: Value,
    task: Task,
) -> Result<Vec<Task>, String> {
    let conn = state;
    let result = models::query(&conn, &where_clause, params, task, TableType::from_num(2))
        .await
        .map_err(|e| e.to_string())?;
    Ok(result)
}

#[tauri::command]
async fn update_task(
    state: State<'_, DB>,
    where_clause: String,
    model: Task,
    table_type: i64,
) -> Result<i64, String> {
    let conn = state;
    models::update(&conn, model, TableType::from_num(table_type), where_clause)
        .await
        .map_err(|e| e.to_string())?;
    Ok(0)
}

#[tauri::command]
async fn delete_task(
    state: State<'_, DB>,
    where_clause: String,
    model: Task,
    table_type: i64,
) -> Result<i64, String> {
    let conn = state;
    models::delete(&conn, model, TableType::from_num(table_type), where_clause)
        .await
        .map_err(|e| e.to_string())?;
    Ok(0)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() -> Result<(), Box<dyn std::error::Error>> {
    let init_db = init_db();
    if let Err(e) = init_db {
        panic!("Failed to initialize database: {}", e);
    }
    let db = init_db.unwrap();
    models::create_task_table(&db)?;
    tauri::Builder::default()
        .setup(|app| {
            if cfg!(debug_assertions) {
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
                        .build(),
                )?;
            }
            Ok(())
        })
        .manage(db)
        .invoke_handler(tauri::generate_handler![
            insert_task,
            query_task,
            update_task,
            delete_task,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
    Ok(())
}

/// 初始化数据库
pub fn init_db() -> Result<DB, rusqlite::Error> {
    let db = DB::init()?;
    Ok(db)
}
