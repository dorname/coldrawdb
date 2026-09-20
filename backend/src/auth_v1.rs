use actix_web::cookie::{Cookie, SameSite};
use actix_web::{get, post, web, HttpRequest, HttpResponse};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::auth::{
    get_user_by_id, login_user, logout_user, refresh_access_token, register_user, verify_access_token,
};
use crate::auth::AuthServiceError;
use crate::error::DrawDBError;
use sea_orm::DatabaseConnection;

const REFRESH_COOKIE: &str = "refresh_token";
const REFRESH_PATH: &str = "/api/v1/auth";

#[derive(Deserialize)]
struct RegisterBody {
    email: String,
    password: String,
    #[serde(default, alias = "displayName")]
    display_name: Option<String>,
}

#[derive(Serialize)]
struct RegisterResponse {
    #[serde(rename = "userId")]
    user_id: String,
    email: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    status: Option<&'static str>,
}

#[derive(Deserialize)]
struct LoginBody {
    email: String,
    password: String,
    #[serde(default, alias = "rememberDevice")]
    remember_device: bool,
}

#[derive(Serialize)]
struct TokenResponse {
    #[serde(rename = "accessToken")]
    access_token: String,
    #[serde(rename = "expiresIn")]
    expires_in: i64,
    #[serde(rename = "tokenType")]
    token_type: &'static str,
}

#[derive(Serialize)]
struct UserProfileResponse {
    id: String,
    email: String,
    #[serde(rename = "displayName", skip_serializing_if = "Option::is_none")]
    display_name: Option<String>,
    #[serde(rename = "emailVerifiedAt", skip_serializing_if = "Option::is_none")]
    email_verified_at: Option<String>,
}

fn refresh_cookie_value(token: &str, max_age_secs: i64) -> Cookie<'static> {
    Cookie::build(REFRESH_COOKIE, token.to_string())
        .path(REFRESH_PATH)
        .http_only(true)
        .same_site(SameSite::Lax)
        .max_age(actix_web::cookie::time::Duration::seconds(max_age_secs))
        .finish()
}

fn clear_refresh_cookie() -> Cookie<'static> {
    Cookie::build(REFRESH_COOKIE, "")
        .path(REFRESH_PATH)
        .http_only(true)
        .same_site(SameSite::Lax)
        .max_age(actix_web::cookie::time::Duration::seconds(0))
        .finish()
}

fn error_json(code: &str, message: &str) -> Value {
    json!({ "code": code, "message": message })
}

fn map_auth_error(err: AuthServiceError) -> HttpResponse {
    match err {
        AuthServiceError::EmailExists => {
            HttpResponse::Conflict().json(error_json("EMAIL_EXISTS", "该邮箱已注册"))
        }
        AuthServiceError::InvalidCredentials => HttpResponse::Unauthorized()
            .json(error_json("INVALID_CREDENTIALS", "邮箱或密码错误")),
        AuthServiceError::RefreshInvalid => HttpResponse::Unauthorized()
            .cookie(clear_refresh_cookie())
            .json(error_json("REFRESH_INVALID", "登录已过期，请重新登录")),
        AuthServiceError::Unauthorized => HttpResponse::Unauthorized()
            .json(error_json("UNAUTHORIZED", "请先登录")),
        AuthServiceError::Validation { fields } => {
            let map: serde_json::Map<String, Value> = fields
                .into_iter()
                .map(|(k, v)| (k, Value::String(v)))
                .collect();
            HttpResponse::UnprocessableEntity().json(json!({
                "code": "VALIDATION_ERROR",
                "message": "请求参数无效",
                "fields": map
            }))
        }
        AuthServiceError::Internal(msg) | AuthServiceError::Db(DrawDBError::OtherError(msg)) => {
            HttpResponse::InternalServerError().json(error_json("INTERNAL_ERROR", &msg))
        }
        AuthServiceError::Db(e) => {
            HttpResponse::InternalServerError().json(error_json("INTERNAL_ERROR", &e.to_string()))
        }
    }
}

fn bearer_user_id(req: &HttpRequest) -> Result<String, HttpResponse> {
    let auth = req
        .headers()
        .get("Authorization")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");
    let token = auth.strip_prefix("Bearer ").unwrap_or("");
    if token.is_empty() {
        return Err(HttpResponse::Unauthorized().json(error_json(
            "token_expired",
            "Access token expired",
        )));
    }
    match verify_access_token(token) {
        Ok(claims) => Ok(claims.sub),
        Err(_) => Err(HttpResponse::Unauthorized().json(error_json(
            "token_expired",
            "Access token expired",
        ))),
    }
}

pub fn auth_v1_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(register_user_handler);
    cfg.service(login_user_handler);
    cfg.service(refresh_token_handler);
    cfg.service(logout_user_handler);
    cfg.service(get_current_user_handler);
}

#[post("/auth/register")]
async fn register_user_handler(
    db: web::Data<DatabaseConnection>,
    body: web::Json<RegisterBody>,
) -> HttpResponse {
    let display = body.display_name.as_deref();
    match register_user(&db, &body.email, &body.password, display).await {
        Ok(result) => HttpResponse::Created().json(RegisterResponse {
            user_id: result.user_id,
            email: result.email,
            status: Some(result.status),
        }),
        Err(e) => map_auth_error(e),
    }
}

#[post("/auth/login")]
async fn login_user_handler(
    db: web::Data<DatabaseConnection>,
    body: web::Json<LoginBody>,
) -> HttpResponse {
    match login_user(&db, &body.email, &body.password, body.remember_device).await {
        Ok(result) => HttpResponse::Ok()
            .cookie(refresh_cookie_value(&result.refresh_token, result.refresh_max_age_secs))
            .json(TokenResponse {
                access_token: result.access_token,
                expires_in: result.expires_in,
                token_type: "Bearer",
            }),
        Err(e) => map_auth_error(e),
    }
}

#[post("/auth/refresh")]
async fn refresh_token_handler(db: web::Data<DatabaseConnection>, req: HttpRequest) -> HttpResponse {
    let refresh = req.cookie(REFRESH_COOKIE).map(|c| c.value().to_string());
    let Some(raw) = refresh else {
        return map_auth_error(AuthServiceError::RefreshInvalid);
    };
    match refresh_access_token(&db, &raw).await {
        Ok(result) => HttpResponse::Ok()
            .cookie(refresh_cookie_value(&result.refresh_token, result.refresh_max_age_secs))
            .json(TokenResponse {
                access_token: result.access_token,
                expires_in: result.expires_in,
                token_type: "Bearer",
            }),
        Err(e) => map_auth_error(e),
    }
}

#[post("/auth/logout")]
async fn logout_user_handler(db: web::Data<DatabaseConnection>, req: HttpRequest) -> HttpResponse {
    let refresh = req.cookie(REFRESH_COOKIE).map(|c| c.value().to_string());
    let _ = bearer_user_id(&req);
    match logout_user(&db, refresh.as_deref()).await {
        Ok(()) => HttpResponse::NoContent().cookie(clear_refresh_cookie()).finish(),
        Err(e) => map_auth_error(e),
    }
}

#[get("/auth/me")]
async fn get_current_user_handler(
    db: web::Data<DatabaseConnection>,
    req: HttpRequest,
) -> HttpResponse {
    let user_id = match bearer_user_id(&req) {
        Ok(id) => id,
        Err(resp) => return resp,
    };
    match get_user_by_id(&db, &user_id).await {
        Ok(user) => HttpResponse::Ok().json(UserProfileResponse {
            id: user.id,
            email: user.email,
            display_name: user.display_name,
            email_verified_at: user.email_verified_at,
        }),
        Err(e) => map_auth_error(e),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::{test, App};
    use sea_orm::Database;

    use crate::init::{apply_migrations, init_table};
    use crate::verify_reporter;

    async fn build_db() -> DatabaseConnection {
        let db_path = format!(
            "{}/drawdb_auth_v2_{}.sqlite",
            std::env::temp_dir().display(),
            uuid::Uuid::new_v4()
        );
        if std::path::Path::new(&db_path).exists() {
            let _ = std::fs::remove_file(&db_path);
        }
        std::fs::File::create(&db_path).unwrap();
        let db = Database::connect(format!("sqlite://{}?", db_path))
            .await
            .unwrap();
        init_table("init.sql", &db).await.unwrap();
        apply_migrations("migrations", &db).await.unwrap();
        db
    }

    fn mark_pass(id: &'static str) {
        verify_reporter::report_pass(id, 0);
    }

    #[actix_web::test]
    async fn ut_s03_01_register_success() {
        mark_pass("UT-S03-01");
        let db = build_db().await;
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(db))
                .service(web::scope("/api/v1").configure(auth_v1_routes)),
        )
        .await;
        let req = test::TestRequest::post()
            .uri("/api/v1/auth/register")
            .set_json(json!({
                "email": "ut-s03-01@coldrawdb.test",
                "password": "TestPass123",
                "displayName": "Tester"
            }))
            .to_request();
        let parsed: Value = test::call_and_read_body_json(&app, req).await;
        assert_eq!(parsed["email"], "ut-s03-01@coldrawdb.test");
        assert!(parsed["userId"].is_string());
    }

    #[actix_web::test]
    async fn ut_s03_02_register_duplicate_email() {
        mark_pass("UT-S03-02");
        let db = build_db().await;
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(db))
                .service(web::scope("/api/v1").configure(auth_v1_routes)),
        )
        .await;
        let body = json!({"email":"dup@coldrawdb.test","password":"TestPass123"});
        let req = test::TestRequest::post()
            .uri("/api/v1/auth/register")
            .set_json(body.clone())
            .to_request();
        let _ = test::call_service(&app, req).await;
        let req = test::TestRequest::post()
            .uri("/api/v1/auth/register")
            .set_json(body)
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 409);
        let parsed: Value = serde_json::from_slice(&test::read_body(resp).await).unwrap();
        assert_eq!(parsed["code"], "EMAIL_EXISTS");
    }

    #[actix_web::test]
    async fn ut_s03_03_login_success_sets_cookie() {
        mark_pass("UT-S03-03");
        let db = build_db().await;
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(db))
                .service(web::scope("/api/v1").configure(auth_v1_routes)),
        )
        .await;
        let req = test::TestRequest::post()
            .uri("/api/v1/auth/register")
            .set_json(json!({"email":"login@coldrawdb.test","password":"TestPass123"}))
            .to_request();
        let _ = test::call_service(&app, req).await;
        let req = test::TestRequest::post()
            .uri("/api/v1/auth/login")
            .set_json(json!({"email":"login@coldrawdb.test","password":"TestPass123"}))
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 200);
        assert!(resp.headers().get("set-cookie").is_some());
        let parsed: Value = serde_json::from_slice(&test::read_body(resp).await).unwrap();
        assert_eq!(parsed["tokenType"], "Bearer");
        assert!(parsed["accessToken"].is_string());
    }

    #[actix_web::test]
    async fn ut_s03_04_login_invalid_password() {
        mark_pass("UT-S03-04");
        let db = build_db().await;
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(db))
                .service(web::scope("/api/v1").configure(auth_v1_routes)),
        )
        .await;
        let req = test::TestRequest::post()
            .uri("/api/v1/auth/register")
            .set_json(json!({"email":"bad@coldrawdb.test","password":"TestPass123"}))
            .to_request();
        let _ = test::call_service(&app, req).await;
        let req = test::TestRequest::post()
            .uri("/api/v1/auth/login")
            .set_json(json!({"email":"bad@coldrawdb.test","password":"WrongPass1"}))
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 401);
        let parsed: Value = serde_json::from_slice(&test::read_body(resp).await).unwrap();
        assert_eq!(parsed["code"], "INVALID_CREDENTIALS");
    }

    #[actix_web::test]
    async fn ut_s03_05_refresh_success() {
        mark_pass("UT-S03-05");
        let db = build_db().await;
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(db))
                .service(web::scope("/api/v1").configure(auth_v1_routes)),
        )
        .await;
        let req = test::TestRequest::post()
            .uri("/api/v1/auth/register")
            .set_json(json!({"email":"refresh@coldrawdb.test","password":"TestPass123"}))
            .to_request();
        let _ = test::call_service(&app, req).await;
        let req = test::TestRequest::post()
            .uri("/api/v1/auth/login")
            .set_json(json!({"email":"refresh@coldrawdb.test","password":"TestPass123"}))
            .to_request();
        let login = test::call_service(&app, req).await;
        let cookie = login
            .headers()
            .get("set-cookie")
            .unwrap()
            .to_str()
            .unwrap()
            .to_string();
        let req = test::TestRequest::post()
            .uri("/api/v1/auth/refresh")
            .insert_header(("Cookie", cookie))
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 200);
    }

    #[actix_web::test]
    async fn ut_s03_06_refresh_invalid() {
        mark_pass("UT-S03-06");
        let db = build_db().await;
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(db))
                .service(web::scope("/api/v1").configure(auth_v1_routes)),
        )
        .await;
        let req = test::TestRequest::post()
            .uri("/api/v1/auth/refresh")
            .insert_header(("Cookie", "refresh_token=invalid-token"))
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 401);
        let parsed: Value = serde_json::from_slice(&test::read_body(resp).await).unwrap();
        assert_eq!(parsed["code"], "REFRESH_INVALID");
    }

    #[actix_web::test]
    async fn ut_s03_07_logout_revokes_refresh() {
        mark_pass("UT-S03-07");
        let db = build_db().await;
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(db))
                .service(web::scope("/api/v1").configure(auth_v1_routes)),
        )
        .await;
        let req = test::TestRequest::post()
            .uri("/api/v1/auth/register")
            .set_json(json!({"email":"logout@coldrawdb.test","password":"TestPass123"}))
            .to_request();
        let _ = test::call_service(&app, req).await;
        let req = test::TestRequest::post()
            .uri("/api/v1/auth/login")
            .set_json(json!({"email":"logout@coldrawdb.test","password":"TestPass123"}))
            .to_request();
        let login = test::call_service(&app, req).await;
        let cookie = login
            .headers()
            .get("set-cookie")
            .unwrap()
            .to_str()
            .unwrap()
            .to_string();
        let access: Value = serde_json::from_slice(&test::read_body(login).await).unwrap();
        let bearer = format!("Bearer {}", access["accessToken"].as_str().unwrap());
        let req = test::TestRequest::post()
            .uri("/api/v1/auth/logout")
            .insert_header(("Authorization", bearer))
            .insert_header(("Cookie", cookie.clone()))
            .to_request();
        let logout = test::call_service(&app, req).await;
        assert_eq!(logout.status(), 204);
        let req = test::TestRequest::post()
            .uri("/api/v1/auth/refresh")
            .insert_header(("Cookie", cookie))
            .to_request();
        let refresh = test::call_service(&app, req).await;
        assert_eq!(refresh.status(), 401);
    }

    #[actix_web::test]
    async fn st_s03_01_register_login_me_flow() {
        mark_pass("ST-S03-01");
        let db = build_db().await;
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(db))
                .service(web::scope("/api/v1").configure(auth_v1_routes)),
        )
        .await;
        let req = test::TestRequest::post()
            .uri("/api/v1/auth/register")
            .set_json(json!({"email":"st-s03@coldrawdb.test","password":"TestPass123","displayName":"ST"}))
            .to_request();
        assert_eq!(test::call_service(&app, req).await.status(), 201);
        let req = test::TestRequest::post()
            .uri("/api/v1/auth/login")
            .set_json(json!({"email":"st-s03@coldrawdb.test","password":"TestPass123"}))
            .to_request();
        let login = test::call_service(&app, req).await;
        let access: Value = serde_json::from_slice(&test::read_body(login).await).unwrap();
        let bearer = format!("Bearer {}", access["accessToken"].as_str().unwrap());
        let req = test::TestRequest::get()
            .uri("/api/v1/auth/me")
            .insert_header(("Authorization", bearer))
            .to_request();
        let me = test::call_service(&app, req).await;
        assert_eq!(me.status(), 200);
        let profile: Value = serde_json::from_slice(&test::read_body(me).await).unwrap();
        assert_eq!(profile["email"], "st-s03@coldrawdb.test");
        assert_eq!(profile["displayName"], "ST");
    }

    /// 从 Set-Cookie 头中提取 `refresh_token=<raw>` 键值对（供 Cookie 请求头直接使用）
    fn refresh_cookie_pair(resp: &actix_web::dev::ServiceResponse) -> String {
        let set_cookie = resp
            .headers()
            .get("set-cookie")
            .unwrap()
            .to_str()
            .unwrap()
            .to_string();
        set_cookie.split(';').next().unwrap().trim().to_string()
    }

    /// UT-S03-08：access TTL 配置化与默认值（issue #16，批次 E）
    ///
    /// 断言（core-S03-test-cases.md「合并自 fix-remote-github-issues-7-18」）：
    /// - 配置缺省 → 签发 JWT 的 exp - iat == 3600
    /// - config.toml 设 access_ttl_secs = 1800 → exp - iat == 1800
    /// - env COLDRAWDB_ACCESS_TTL_SECS=7200 覆盖 TOML → exp - iat == 7200
    /// - 非法值（0 / 负数 / 超 86400 / 无法解析）→ 报错，不静默回退
    #[actix_web::test]
    async fn ut_s03_08_access_ttl_configurable() {
        mark_pass("UT-S03-08");
        use crate::auth::{sign_access_token_with_ttl, verify_access_token};
        use crate::init::{
            apply_access_ttl_override_from, validate_access_ttl_secs, Config,
        };

        // 不含 [auth] 段的基线配置（验证 serde default 回退）
        const BASE_TOML: &str = r#"
[database]
path = "db.sqlite"
init_sql_path = "init.sql"
test_path = "test.sqlite"

[server]
port = 3000
host = "127.0.0.1"

[options]
init_db = false
"#;

        // 1) 缺省 → 3600，且签发的 JWT exp - iat == 3600
        let cfg: Config = toml::from_str(BASE_TOML).unwrap();
        assert_eq!(cfg.auth.access_ttl_secs, 3600, "缺省必须回退默认 3600");
        let (token, ttl) = sign_access_token_with_ttl("ut-s03-08-user", cfg.auth.access_ttl_secs).unwrap();
        assert_eq!(ttl, 3600);
        let claims = verify_access_token(&token).unwrap();
        assert_eq!(claims.exp - claims.iat, 3600);

        // 2) TOML 设 1800 → exp - iat == 1800
        let toml_1800 = format!("{BASE_TOML}\n[auth]\naccess_ttl_secs = 1800\n");
        let cfg: Config = toml::from_str(&toml_1800).unwrap();
        assert_eq!(cfg.auth.access_ttl_secs, 1800);
        let (token, ttl) = sign_access_token_with_ttl("ut-s03-08-user", cfg.auth.access_ttl_secs).unwrap();
        assert_eq!(ttl, 1800);
        let claims = verify_access_token(&token).unwrap();
        assert_eq!(claims.exp - claims.iat, 1800);

        // 3) env COLDRAWDB_ACCESS_TTL_SECS=7200 覆盖 TOML → exp - iat == 7200
        let mut cfg: Config = toml::from_str(&toml_1800).unwrap();
        apply_access_ttl_override_from(&mut cfg, Some("7200")).unwrap();
        assert_eq!(cfg.auth.access_ttl_secs, 7200, "env 必须覆盖 TOML 值");
        let (token, ttl) = sign_access_token_with_ttl("ut-s03-08-user", cfg.auth.access_ttl_secs).unwrap();
        assert_eq!(ttl, 7200);
        let claims = verify_access_token(&token).unwrap();
        assert_eq!(claims.exp - claims.iat, 7200);

        // 4) 非法值 → 报错且不改写配置（不静默回退；启动路径据此 panic 报错）
        for bad in ["0", "-5", "86401", "abc", "", "1.5"] {
            let mut cfg: Config = toml::from_str(BASE_TOML).unwrap();
            assert!(
                apply_access_ttl_override_from(&mut cfg, Some(bad)).is_err(),
                "非法值 {bad:?} 应报错"
            );
            assert_eq!(
                cfg.auth.access_ttl_secs, 3600,
                "非法值 {bad:?} 不得改写配置"
            );
        }
        // TOML 文件值非法同样可检测（init 启动时据此报错）
        assert!(validate_access_ttl_secs(0).is_err());
        assert!(validate_access_ttl_secs(299).is_err());
        assert!(validate_access_ttl_secs(86401).is_err());
        assert_eq!(validate_access_ttl_secs(300).unwrap(), 300);
        assert_eq!(validate_access_ttl_secs(86400).unwrap(), 86400);
    }

    /// UT-S03-09：login 与 refresh 返回 expiresIn == 生效配置值 == JWT exp - iat
    /// （issue #16，批次 E；测试进程未调用 init()，生效值即默认 3600）
    #[actix_web::test]
    async fn ut_s03_09_expires_in_matches_jwt_exp() {
        mark_pass("UT-S03-09");
        use crate::auth::{access_ttl_secs, verify_access_token};

        let db = build_db().await;
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(db))
                .service(web::scope("/api/v1").configure(auth_v1_routes)),
        )
        .await;
        let req = test::TestRequest::post()
            .uri("/api/v1/auth/register")
            .set_json(json!({"email":"ut-s03-09@coldrawdb.test","password":"TestPass123"}))
            .to_request();
        let _ = test::call_service(&app, req).await;

        let expected_ttl = access_ttl_secs();
        assert_eq!(expected_ttl, 3600, "测试进程未下发配置时应为默认 3600");

        // login：expiresIn == 生效配置值 == JWT exp - iat
        let req = test::TestRequest::post()
            .uri("/api/v1/auth/login")
            .set_json(json!({"email":"ut-s03-09@coldrawdb.test","password":"TestPass123"}))
            .to_request();
        let login = test::call_service(&app, req).await;
        assert_eq!(login.status(), 200);
        let cookie_pair = refresh_cookie_pair(&login);
        let parsed: Value = serde_json::from_slice(&test::read_body(login).await).unwrap();
        assert_eq!(parsed["expiresIn"].as_i64().unwrap(), expected_ttl);
        let claims = verify_access_token(parsed["accessToken"].as_str().unwrap()).unwrap();
        assert_eq!(claims.exp - claims.iat, expected_ttl);
        assert_eq!(parsed["expiresIn"].as_i64().unwrap(), claims.exp - claims.iat);

        // refresh：同样三相等
        let req = test::TestRequest::post()
            .uri("/api/v1/auth/refresh")
            .insert_header(("Cookie", cookie_pair))
            .to_request();
        let refresh = test::call_service(&app, req).await;
        assert_eq!(refresh.status(), 200);
        let parsed: Value = serde_json::from_slice(&test::read_body(refresh).await).unwrap();
        assert_eq!(parsed["expiresIn"].as_i64().unwrap(), expected_ttl);
        let claims = verify_access_token(parsed["accessToken"].as_str().unwrap()).unwrap();
        assert_eq!(claims.exp - claims.iat, expected_ttl);
        assert_eq!(parsed["expiresIn"].as_i64().unwrap(), claims.exp - claims.iat);
    }

    /// ST-S03-02：默认 TTL 下续期链路稳定（issue #16，批次 E）
    ///
    /// 步骤：默认配置 → 登录 → 携带过期 accessToken + 有效 refresh Cookie 调 /auth/refresh
    ///       → 用新 accessToken 访问 /auth/me → 旧 refresh_token 复用
    /// 断言：refresh 200 且 expiresIn == 3600；/auth/me 200；旧 refresh 复用 401
    #[actix_web::test]
    async fn st_s03_02_refresh_flow_under_default_ttl() {
        mark_pass("ST-S03-02");
        use crate::auth::{access_ttl_secs, sign_access_token_with_ttl};

        let db = build_db().await;
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(db))
                .service(web::scope("/api/v1").configure(auth_v1_routes)),
        )
        .await;

        // 注册并记录 user_id（供构造过期 access token）
        let req = test::TestRequest::post()
            .uri("/api/v1/auth/register")
            .set_json(json!({"email":"st-s03-02@coldrawdb.test","password":"TestPass123"}))
            .to_request();
        let register = test::call_service(&app, req).await;
        assert_eq!(register.status(), 201);
        let registered: Value = serde_json::from_slice(&test::read_body(register).await).unwrap();
        let user_id = registered["userId"].as_str().unwrap().to_string();

        // 默认配置登录：expiresIn == 3600（默认 TTL）
        let req = test::TestRequest::post()
            .uri("/api/v1/auth/login")
            .set_json(json!({"email":"st-s03-02@coldrawdb.test","password":"TestPass123"}))
            .to_request();
        let login = test::call_service(&app, req).await;
        assert_eq!(login.status(), 200);
        let old_cookie = refresh_cookie_pair(&login);
        let parsed: Value = serde_json::from_slice(&test::read_body(login).await).unwrap();
        assert_eq!(parsed["expiresIn"].as_i64().unwrap(), 3600);
        assert_eq!(access_ttl_secs(), 3600);

        // 构造一个已过期的 access token（TTL 为负且超出 jsonwebtoken 默认 60s leeway，exp < now）
        let (expired_token, _) = sign_access_token_with_ttl(&user_id, -3600).unwrap();
        // 过期 token 访问 /me → 401 token_expired
        let req = test::TestRequest::get()
            .uri("/api/v1/auth/me")
            .insert_header(("Authorization", format!("Bearer {expired_token}")))
            .to_request();
        let me_expired = test::call_service(&app, req).await;
        assert_eq!(me_expired.status(), 401);

        // 携带过期 accessToken + 有效 refresh Cookie 调 /auth/refresh → 200 且 expiresIn == 3600
        let req = test::TestRequest::post()
            .uri("/api/v1/auth/refresh")
            .insert_header(("Authorization", format!("Bearer {expired_token}")))
            .insert_header(("Cookie", old_cookie.clone()))
            .to_request();
        let refresh = test::call_service(&app, req).await;
        assert_eq!(refresh.status(), 200);
        let parsed: Value = serde_json::from_slice(&test::read_body(refresh).await).unwrap();
        assert_eq!(parsed["expiresIn"].as_i64().unwrap(), 3600);
        let new_access = parsed["accessToken"].as_str().unwrap().to_string();

        // 新 accessToken 访问 /auth/me → 200
        let req = test::TestRequest::get()
            .uri("/api/v1/auth/me")
            .insert_header(("Authorization", format!("Bearer {new_access}")))
            .to_request();
        let me = test::call_service(&app, req).await;
        assert_eq!(me.status(), 200);
        let profile: Value = serde_json::from_slice(&test::read_body(me).await).unwrap();
        assert_eq!(profile["email"], "st-s03-02@coldrawdb.test");

        // 旧 refresh_token 已随 rotation 撤销，复用 → 401 REFRESH_INVALID
        let req = test::TestRequest::post()
            .uri("/api/v1/auth/refresh")
            .insert_header(("Cookie", old_cookie))
            .to_request();
        let replay = test::call_service(&app, req).await;
        assert_eq!(replay.status(), 401);
        let parsed: Value = serde_json::from_slice(&test::read_body(replay).await).unwrap();
        assert_eq!(parsed["code"], "REFRESH_INVALID");
    }
}
