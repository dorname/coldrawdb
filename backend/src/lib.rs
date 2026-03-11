pub mod common;
pub mod entity;
pub mod error;
pub mod init;
pub mod repository;
pub mod templates;
pub mod tables;
pub mod references;
pub mod diagrams;
pub mod todos;

pub use error::DrawDBError;
pub use init::{get_config, init};

use actix_web::{get, web, Responder};
use std::sync::Mutex;

lazy_static::lazy_static! {
    static ref ID_GEN: Mutex<snowflake::SnowflakeIdGenerator> = Mutex::new(
        snowflake::SnowflakeIdGenerator::new(1, 1)
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

async fn index() -> impl Responder {
    "Hello, world!"
}

#[get("/hello/{name}")]
async fn hello(name: web::Path<String>) -> impl Responder {
    format!("Hello, {}!", name)
}

/// 统一路由配置，供 main 与集成测试共用
pub fn app_config(cfg: &mut web::ServiceConfig) {
    cfg.service(hello)
        .route("/", web::get().to(index))
        .service(web::scope("/todos").configure(todos::todos_routes))
        .service(web::scope("/tables").configure(tables::tables_routes))
        .service(web::scope("/diagrams").configure(diagrams::diagrams_routes))
        .service(web::scope("/references").configure(references::references_routes))
        .service(web::scope("/templates").configure(templates::templates_routes));
}
