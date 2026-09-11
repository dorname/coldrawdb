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
mod bridge_introspect;
mod auth;
mod auth_v1;
mod rooms;
mod rooms_v1;
mod collab;
mod collab_v1;
mod verify_reporter;
mod static_serve;
#[cfg(test)]
mod embedded_pg;
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


/// 统一挂载所有 V1 API 路由。
///
/// 禁止在 App::service 中重复注册 `web::scope("/api/v1")`；actix-web 遇到相同前缀的多个
/// scope 时只会匹配第一个，导致 auth / rooms / collab / bridge 端点全部 404。
pub fn api_v1_routes(cfg: &mut web::ServiceConfig) {
    diagrams_v1::diagrams_v1_routes(cfg);
    auth_v1::auth_v1_routes(cfg);
    rooms_v1::rooms_v1_routes(cfg);
    collab_v1::collab_rest_routes(cfg);
    phase3_bridge::phase3_bridge_routes(cfg);
}

/// 初始化日志
fn init_log() {
    // fix-select-bg-diagram-delete-and-log-format（core-PV UT-PU-21）：
    // 1) 去掉 pretty 多行格式（与 compact 冲突且多行缩进落盘可读性差）
    // 2) 禁 ANSI 转义（with_ansi(false)）：编辑器查看日志不再出现 ESC[0m/ESC[32m 字符
    // 3) 默认 filter 降噪 sqlx::query=warn（逐条 SQL 的 INFO 刷屏），RUST_LOG 可覆盖
    // feat-docker-compose-deploy：COLDRAWDB_LOG_LEVEL 覆盖默认级别（§4.3），RUST_LOG 仍最优先
    let default_filter = match std::env::var("COLDRAWDB_LOG_LEVEL") {
        Ok(level) => format!("{level},sqlx::query=warn"),
        Err(_) => "info,sqlx::query=warn".to_string(),
    };
    fmt()
    .with_env_filter(EnvFilter::try_from_default_env()
    .unwrap_or_else(|_| EnvFilter::new(default_filter)))
    .with_file(true)
    .with_line_number(true)
    .with_ansi(false)
    .compact()
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

    // feat-docker-compose-deploy（§4.3）：静态目录未就绪时退化为纯 API 模式
    let static_dir = static_serve::resolve_static_dir();
    match &static_dir {
        Some(dir) => tracing::info!(static_dir = %dir, "static file serving enabled"),
        None => tracing::warn!("COLDRAWDB_STATIC_DIR not ready; running in API-only mode"),
    }

    HttpServer::new(move || {
        App::new()
            .wrap(Cors::permissive())
            .wrap(actix_web::middleware::from_fn(static_serve::cache_headers))
            .app_data(web::Data::new(db.clone().unwrap()))
            .app_data(web::Data::new(collab_hub.clone()))
            .service(hello)
            .service(web::scope("/todos").configure(todos::todos_routes))
            .service(web::scope("/tables").configure(tables::tables_routes))
            .service(web::scope("/diagrams").configure(diagrams::diagrams_routes))
            .service(web::scope("/api/v1").configure(api_v1_routes))
            .route("/ws/rooms/{room_id}", web::get().to(collab::collab_ws_handler))
            // 兜底：静态服务（含 SPA 回源）或纯 API 存活文案，必须最后注册
            .configure({
                let static_dir = static_dir.clone();
                move |cfg| static_serve::configure_static(cfg, static_dir.clone())
            })

    })
    .bind(format!("{}:{}", host, port))?
    .run()
    .await
    .map_err(DrawDBError::IoError)
}

/// 测试
#[get("/hello/{name}")]
async fn hello(name: web::Path<String>) -> impl Responder {
    format!("Hello, {}!", name)
}

#[cfg(test)]
mod tests {
    /// UT-PU-21：日志格式锚点（fix-select-bg-diagram-delete-and-log-format，core-PV §1）
    ///
    /// 断言后端 init_log 与前端启动脚本的日志格式条款：
    /// 禁 ANSI、去 pretty、sqlx 降噪、trunk 强制 NO_COLOR。
    #[test]
    fn ut_pu_21_log_format_anchor() {
        let main_src = include_str!("main.rs");
        let init_idx = main_src
            .find("fn init_log()")
            .expect("UT-PU-21: init_log 函数存在");
        let init_block = &main_src[init_idx..init_idx + 900];

        assert!(
            init_block.contains(".with_ansi(false)"),
            "UT-PU-21: init_log 必须 .with_ansi(false)（禁 ANSI 转义）"
        );
        assert!(
            !init_block.contains(".pretty()"),
            "UT-PU-21: init_log 不得使用 .pretty()（与 compact 冲突且多行落盘可读性差）"
        );
        assert!(
            init_block.contains("sqlx::query=warn"),
            "UT-PU-21: 默认 EnvFilter 应含 sqlx::query=warn 降噪"
        );

        let start_sh = std::fs::read_to_string("../scripts/start-local.sh")
            .expect("UT-PU-21: 可读 scripts/start-local.sh");
        let trunk_line = start_sh
            .lines()
            .find(|line| line.contains("trunk serve") && line.contains("exec env"))
            .expect("UT-PU-21: trunk 启动行存在");
        assert!(
            trunk_line.contains("NO_COLOR=true"),
            "UT-PU-21: trunk 启动行应含 NO_COLOR=true（trunk 仅接受 bool），实际：{trunk_line}"
        );
        assert!(
            !trunk_line.contains("-u NO_COLOR"),
            "UT-PU-21: trunk 启动行不得再 -u NO_COLOR，实际：{trunk_line}"
        );
        crate::verify_reporter::report_pass("UT-PU-21", 0);
    }
}
