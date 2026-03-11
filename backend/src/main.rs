use actix_web::{web, App, HttpServer};
use backend::{app_config, get_config, init, DrawDBError};
use std::result::Result;
use tracing_subscriber::fmt;
use tracing_subscriber::EnvFilter;

fn init_log() {
    fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new("info")),
        )
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

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(db.clone().unwrap()))
            .configure(app_config)
    })
    .bind(format!("{}:{}", host, port))?
    .run()
    .await
    .map_err(DrawDBError::IoError)
}
