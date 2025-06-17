use actix_web::{get, web, App, HttpServer, Responder};
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
use error::DrawDBError;
use init::{get_config, init};
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
    let db = init().await?;
    let server_config = get_config();
    let config = server_config
        .read()
        .map_err(|e| DrawDBError::OtherError(e.to_string()))?;
    let host = config.host.clone();
    let port = config.port.clone();

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(db.clone().unwrap()))
            .service(hello)
            .route("/", web::get().to(index))
            .service(web::scope("/todos").configure(todos::todos_routes))
            .service(web::scope("/tables").configure(tables::tables_routes))
            .service(web::scope("/diagrams").configure(diagrams::diagrams_routes))
     
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
