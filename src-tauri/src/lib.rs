use db::models::diagram::Diagram;
use db::{models, DB};
use tauri::State;
mod db;
use db::models::task::Task;
use db::models::template::Template;
use db::models::TableType;
use serde_json::Value;


#[tauri::command]
async fn insert_diagram(state: State<'_, DB>, diagram: Diagram) -> Result<i64, String> {
    let conn = state;
    models::insert(&conn, diagram, TableType::Diagrams)
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
        TableType::Diagrams,
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
async fn update_diagram(
    state: State<'_, DB>,
    where_clause: String,
    model: Diagram,
) -> Result<i64, String> {
    let conn = state;
    models::update(&conn, model, TableType::Diagrams, where_clause)
        .await
        .map_err(|e| e.to_string())?;
    Ok(0)
}

#[tauri::command]
async fn delete_diagram(
    state: State<'_, DB>,
    where_clause: String,
    model: Diagram,
) -> Result<i64, String> {
    let conn = state;
    models::delete(&conn, model, TableType::Diagrams, where_clause)
        .await
        .map_err(|e| e.to_string())?;
    Ok(0)
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

#[tauri::command]
async fn insert_template(state: State<'_, DB>, template: Template) -> Result<i64, String> {
    let conn = state;
    models::insert(&conn, template, TableType::Templates)
        .await
        .map_err(|e| e.to_string())?;
    Ok(0)
}

#[tauri::command]
async fn query_template(
    state: State<'_, DB>,
    where_clause: String,
    params: Value,
    template: Template,
) -> Result<Vec<Template>, String> {
    let conn = state;
    let result = models::query(&conn, &where_clause, params, template, TableType::Templates)
        .await
        .map_err(|e| e.to_string())?;
    Ok(result)
}

#[tauri::command]
async fn update_template(
    state: State<'_, DB>,
    where_clause: String,
    template: Template,
) -> Result<i64, String> {
    let conn = state;
    models::update(&conn, template, TableType::Templates, where_clause)
        .await
        .map_err(|e| e.to_string())?;
    Ok(0)
}

#[tauri::command]
async fn delete_template(
    state: State<'_, DB>,
    where_clause: String,
    template: Template,
) -> Result<i64, String> {
    let conn = state;
    models::delete(&conn, template, TableType::Templates, where_clause)
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
    models::init_tables(&db)?;
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
            insert_diagram,
            query_diagram,
            update_diagram,
            delete_diagram,
            insert_template,
            query_template,
            update_template,
            delete_template,
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
