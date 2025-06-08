use actix_web::{get, web, App, HttpServer, Responder};
mod common;
mod entity;
mod error;
mod init;
mod todos;
use error::DrawDBError;
use init::{get_config, init};
use std::result::Result;

#[actix_web::main]
async fn main() -> Result<(), DrawDBError> {
    let db = init().await?;
    let server_config = get_config();
    let config = server_config
        .read()
        .map_err(|e| DrawDBError::OtherError(e.to_string()))?;
    let host = config.host.clone();
    let port = config.port.clone();

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(db.clone()))
            .service(hello)
            .route("/", web::get().to(index))
            .service(web::scope("/todos").configure(todos::todos_routes))
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
