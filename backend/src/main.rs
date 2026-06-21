use actix_web::{get, web, App, HttpServer, Responder};
use actix_cors::Cors;
mod common;
mod entity;
mod error;
mod init;
mod todos;
mod diagrams;
mod references;
mod areas;
mod notes;
mod tables;
mod indices;
mod fields;
mod repository;
mod diagrams_v1;
mod phase3_bridge;
mod auth;
mod auth_v1;
mod rooms;
mod rooms_v1;
mod collab;
mod collab_v1;
mod verify_reporter;
mod diagram_persistence;
use error::DrawDBError;
use init::{get_config, init};
use collab::CollabHub;
use tracing_subscriber::fmt;
use std::result::Result;
use snowflake::{SnowflakeIdGenerator};
use std::sync::Mutex;
use tracing_subscriber::EnvFilter;

// 全局单例生成器，假设机器 ID 为 1
lazy_static::lazy_static! {
    static ref ID_GEN: Mutex<SnowflakeIdGenerator> = Mutex::new(
       SnowflakeIdGenerator::new(1, 1)
    );
}

/// 取一个雪花 ID
pub fn next_id() -> String {
    let mut g = ID_GEN.lock().unwrap();
    g.generate().to_string()
}

/// 批量生成雪花 ID
pub fn next_ids(count: usize) -> Vec<String> {
    let mut g = ID_GEN.lock().unwrap();
    (0..count).map(|_| g.generate().to_string()).collect()
}


/// 初始化日志
fn init_log() {
    // 1) 初始化 env filter
    // 2) 初始化 fmt subscriber
    fmt()
    .with_env_filter(EnvFilter::try_from_default_env()
    .unwrap_or_else(|_| EnvFilter::new("info")))
    .with_file(true)
    .with_line_number(true)
    .compact()
    .pretty()
    .init();
}

#[actix_web::main]
async fn main() -> Result<(), DrawDBError> {
    init_log();
    let db = init(false).await?;
    let server_config = get_config();
    let config = server_config
        .read()
        .map_err(|e| DrawDBError::OtherError(e.to_string()))?;
    let host = config.host.clone();
    let port = config.port.clone();
    let collab_hub = CollabHub::new();

    HttpServer::new(move || {
        App::new()
            .wrap(Cors::permissive())
            .app_data(web::Data::new(db.clone().unwrap()))
            .app_data(web::Data::new(collab_hub.clone()))
            .service(hello)
            .route("/", web::get().to(index))
            .service(web::scope("/todos").configure(todos::todos_routes))
            .service(web::scope("/tables").configure(tables::tables_routes))
            .service(web::scope("/diagrams").configure(diagrams::diagrams_routes))
            .service(web::scope("/api/v1").configure(diagrams_v1::diagrams_v1_routes))
            .service(web::scope("/api/v1").configure(auth_v1::auth_v1_routes))
            .service(web::scope("/api/v1").configure(rooms_v1::rooms_v1_routes))
            .service(web::scope("/api/v1").configure(collab_v1::collab_rest_routes))
            .service(web::scope("/api/v1").configure(phase3_bridge::phase3_bridge_routes))
            .route("/ws/rooms/{room_id}", web::get().to(collab::collab_ws_handler))

    })
    .bind(format!("{}:{}", host, port))?
    .run()
    .await
    .map_err(DrawDBError::IoError)
}

/// 例子
async fn index() -> impl Responder {
    "Hello, world!"
}

/// 测试
#[get("/hello/{name}")]
async fn hello(name: web::Path<String>) -> impl Responder {
    format!("Hello, {}!", name)
}
