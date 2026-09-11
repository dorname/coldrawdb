//! feat-docker-compose-deploy（core-01 部署方案 §4/§12.3）：单容器静态文件服务。
//!
//! - `resolve_static_dir`：`COLDRAWDB_STATIC_DIR`（默认 `/var/www`）下存在 index.html 才启用；
//!   否则后端退化为纯 API 模式（`GET /` 返回 200 存活文案，不 panic）。
//! - SPA 回源：非 API 的未知路径一律回源 `index.html`（§12.3 列出的 `/invite/*`、`/editor/*`、
//!   `/rooms*`、`/login*` 等前端路由不得 404）。
//! - 缓存头（§3.4.6）：`*.wasm` / `*.js` → immutable 一年；`/` / `index.html` / 无扩展名前端
//!   路由 → `no-cache`；`/api/` / `/ws/` 不干预。

use actix_web::body::MessageBody;
use actix_web::dev::{ServiceRequest, ServiceResponse};
use actix_web::http::header::{HeaderValue, CACHE_CONTROL};
use actix_web::middleware::Next;
use actix_web::{web, Error, HttpResponse};

const IMMUTABLE: &str = "public, max-age=31536000, immutable";
const NO_CACHE: &str = "no-cache";

/// 缓存头决策纯函数（UT-DP-02）。
pub fn cache_control_for_path(path: &str) -> Option<&'static str> {
    if path.starts_with("/api/") || path.starts_with("/ws/") {
        return None;
    }
    if path.ends_with(".wasm") || path.ends_with(".js") {
        return Some(IMMUTABLE);
    }
    let seg = path.rsplit('/').next().unwrap_or(path);
    if path == "/" || seg == "index.html" || !seg.contains('.') {
        return Some(NO_CACHE);
    }
    None
}

/// wrap_fn 中间件：按 `cache_control_for_path` 给响应补 Cache-Control。
pub async fn cache_headers<B: MessageBody + 'static>(
    req: ServiceRequest,
    next: Next<B>,
) -> Result<ServiceResponse<B>, Error> {
    let path = req.path().to_string();
    let mut res = next.call(req).await?;
    if let Some(value) = cache_control_for_path(&path) {
        res.headers_mut()
            .insert(CACHE_CONTROL, HeaderValue::from_static(value));
    }
    Ok(res)
}

/// 静态目录就绪判定（存在 index.html 才算可用）。
pub fn static_dir_ready(dir: &str) -> bool {
    std::path::Path::new(dir).join("index.html").is_file()
}

/// 从环境解析静态目录：`COLDRAWDB_STATIC_DIR`，默认 `/var/www`；未就绪返回 None。
pub fn resolve_static_dir() -> Option<String> {
    let dir = std::env::var("COLDRAWDB_STATIC_DIR").unwrap_or_else(|_| "/var/www".to_string());
    if static_dir_ready(&dir) {
        Some(dir)
    } else {
        None
    }
}

/// 纯 API 模式下的存活文案（不依赖任何静态文件）。
async fn index_alive() -> HttpResponse {
    HttpResponse::Ok().body("coldrawdb backend alive (static dir not configured)")
}

/// 挂载静态服务或纯 API 降级。
///
/// 必须在所有 API scope 之后注册：`Files::new("/", ...)` 是兜底服务，仅当既有路由
/// （`/api/v1`、`/todos`、`/ws/...` 等）未匹配时才接管。
pub fn configure_static(cfg: &mut web::ServiceConfig, static_dir: Option<String>) {
    match static_dir {
        Some(dir) => {
            let index = std::path::Path::new(&dir).join("index.html");
            cfg.service(
                actix_files::Files::new("/", dir)
                    .index_file("index.html")
                    // SPA 回源：文件未命中时返回 index.html（§12.3）
                    .default_handler(move |req: ServiceRequest| {
                        let index = index.clone();
                        async move {
                            let (req, _) = req.into_parts();
                            let file = actix_files::NamedFile::open_async(&index).await?;
                            let res = file.into_response(&req);
                            Ok(ServiceResponse::new(req, res))
                        }
                    }),
            );
        }
        None => {
            cfg.route("/", web::get().to(index_alive));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::http::header;
    use actix_web::{test, App};

    /// UT-DP-02：缓存头决策纯函数（core-PV §5）
    #[actix_web::test]
    async fn ut_dp_02_cache_control_for_path() {
        assert_eq!(cache_control_for_path("/app.wasm"), Some(IMMUTABLE));
        assert_eq!(
            cache_control_for_path("/assets/editor-abc123.js"),
            Some(IMMUTABLE)
        );
        assert_eq!(cache_control_for_path("/"), Some(NO_CACHE));
        assert_eq!(cache_control_for_path("/index.html"), Some(NO_CACHE));
        assert_eq!(cache_control_for_path("/editor"), Some(NO_CACHE));
        assert_eq!(cache_control_for_path("/invite/abc123"), Some(NO_CACHE));
        assert_eq!(cache_control_for_path("/rooms"), Some(NO_CACHE));
        assert_eq!(cache_control_for_path("/api/v1/diagrams"), None);
        assert_eq!(cache_control_for_path("/ws/rooms/1"), None);
        assert_eq!(cache_control_for_path("/favicon.ico"), None);
        assert_eq!(cache_control_for_path("/styles/main.css"), None);
        crate::verify_reporter::report_pass("UT-DP-02", 0);
    }

    fn temp_static_dir() -> std::path::PathBuf {
        let dir = std::env::temp_dir().join(format!("coldrawdb_static_{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(
            dir.join("index.html"),
            "<!doctype html><html><body><div id=\"root\"></div><script type=\"module\"></script></body></html>",
        )
        .unwrap();
        std::fs::write(dir.join("app.wasm"), b"\0asm fake wasm payload").unwrap();
        dir
    }

    /// UT-DP-04：静态服务 + SPA 回源 + 缓存头（core-PV §5）
    #[actix_web::test]
    async fn ut_dp_04_static_spa_and_cache_headers() {
        let dir = temp_static_dir();
        let dir_str = dir.to_string_lossy().to_string();
        assert!(static_dir_ready(&dir_str));

        let app = test::init_service(
            App::new()
                .wrap(actix_web::middleware::from_fn(cache_headers))
                .configure(move |cfg| configure_static(cfg, Some(dir_str.clone()))),
        )
        .await;

        // GET / → index.html + no-cache
        let req = test::TestRequest::get().uri("/").to_request();
        let res = test::call_service(&app, req).await;
        assert_eq!(res.status(), 200);
        assert_eq!(
            res.headers().get(header::CACHE_CONTROL).unwrap(),
            "no-cache"
        );
        let body = test::read_body(res).await;
        assert!(String::from_utf8_lossy(&body).contains("<div id=\"root\">"));

        // GET /editor → SPA 回源同一 index.html + no-cache
        let req = test::TestRequest::get().uri("/editor").to_request();
        let res = test::call_service(&app, req).await;
        assert_eq!(res.status(), 200);
        assert_eq!(
            res.headers().get(header::CACHE_CONTROL).unwrap(),
            "no-cache"
        );
        let body = test::read_body(res).await;
        assert!(String::from_utf8_lossy(&body).contains("<div id=\"root\">"));

        // GET /app.wasm → 200 + immutable
        let req = test::TestRequest::get().uri("/app.wasm").to_request();
        let res = test::call_service(&app, req).await;
        assert_eq!(res.status(), 200);
        assert_eq!(
            res.headers().get(header::CACHE_CONTROL).unwrap(),
            "public, max-age=31536000, immutable"
        );

        std::fs::remove_dir_all(&dir).unwrap();
        crate::verify_reporter::report_pass("UT-DP-04", 0);
    }

    /// UT-DP-05：纯 API 模式降级（core-PV §5）
    #[actix_web::test]
    async fn ut_dp_05_api_only_fallback() {
        assert!(!static_dir_ready("/definitely/not/exist"));

        let app = test::init_service(
            App::new()
                .wrap(actix_web::middleware::from_fn(cache_headers))
                .configure(|cfg| configure_static(cfg, None)),
        )
        .await;

        let req = test::TestRequest::get().uri("/").to_request();
        let res = test::call_service(&app, req).await;
        assert_eq!(res.status(), 200);
        let body = test::read_body(res).await;
        assert!(String::from_utf8_lossy(&body).contains("coldrawdb"));

        let req = test::TestRequest::get().uri("/definitely-missing").to_request();
        let res = test::call_service(&app, req).await;
        assert_eq!(res.status(), 404);

        crate::verify_reporter::report_pass("UT-DP-05", 0);
    }
}
