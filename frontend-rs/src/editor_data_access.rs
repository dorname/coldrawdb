//! editor-data-access: HTTP client for drawdb REST API v1
//!
//! Implements `DiagramClient` using `gloo-net` for WASM-compatible HTTP.
//! Error types follow spec §Round 8 resolution: `SaveError::Conflict` carries
//! `current_revision` so the caller can surface a 409 modal (force-overwrite / reload).
//!
//! Dependencies:
//!   - `gloo-net` for WASM HTTP
//!   - `thiserror` for error enums
//!   - `chrono` for `saved_at` timestamps
//!   - `serde` + `serde_json` for request/response bodies
//!   - `crate::editor_core::types::Diagram` (defined in editor-core, shared across modules)

use crate::editor_core::types::{Area, Database, Diagram, Field, Index, Note, Reference, Table};
use chrono::{DateTime, Utc};
use gloo_net::http::Request;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use web_sys::RequestCredentials;

// ---------------------------------------------------------------------------
// Error types
// ---------------------------------------------------------------------------

/// General API errors from GET / POST / DELETE operations.
#[derive(Debug, Error)]
pub enum ApiError {
    #[error("network: {0}")]
    Network(String),
    #[error("server {0}: {1}")]
    Server(u16, String),
    #[error("parse: {0}")]
    Parse(String),
}

/// S03 auth API errors.
#[derive(Debug, Error)]
pub enum AuthError {
    #[error("network: {0}")]
    Network(String),
    #[error("server {0}: {1}")]
    Server(u16, String),
    #[error("parse: {0}")]
    Parse(String),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct UserProfile {
    pub id: String,
    pub email: String,
    #[serde(rename = "displayName", default)]
    pub display_name: Option<String>,
    #[serde(rename = "emailVerifiedAt", default)]
    pub email_verified_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TokenResponse {
    #[serde(rename = "accessToken")]
    pub access_token: String,
    #[serde(rename = "expiresIn")]
    pub expires_in: i64,
    #[serde(rename = "tokenType")]
    pub token_type: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuthSession {
    pub access_token: String,
    pub expires_in: i64,
    pub token_type: String,
    pub user: Option<UserProfile>,
}

impl AuthSession {
    pub fn from_token(token: TokenResponse) -> Self {
        Self {
            access_token: token.access_token,
            expires_in: token.expires_in,
            token_type: token.token_type,
            user: None,
        }
    }

    pub fn bearer_header(&self) -> String {
        format!("Bearer {}", self.access_token)
    }

    pub fn display_name(&self) -> String {
        self.user
            .as_ref()
            .and_then(|u| u.display_name.clone())
            .or_else(|| self.user.as_ref().map(|u| u.email.clone()))
            .unwrap_or_else(|| "已登录用户".to_string())
    }
}

#[derive(Serialize)]
struct RegisterReq {
    email: String,
    password: String,
    #[serde(rename = "displayName")]
    display_name: String,
}

#[derive(Serialize)]
struct LoginReq {
    email: String,
    password: String,
    #[serde(rename = "rememberDevice")]
    remember_device: bool,
}

#[derive(Deserialize)]
struct ErrorBody {
    code: Option<String>,
    message: Option<String>,
}

pub fn is_token_expired_error(status: u16, body: &str) -> bool {
    if status != 401 {
        return false;
    }
    serde_json::from_str::<ErrorBody>(body)
        .ok()
        .and_then(|b| b.code)
        .map(|code| code == "token_expired" || code == "REFRESH_INVALID")
        .unwrap_or(false)
}

pub fn auth_error_message(status: u16, body: &str) -> String {
    serde_json::from_str::<ErrorBody>(body)
        .ok()
        .and_then(|b| b.message.or(b.code))
        .unwrap_or_else(|| format!("认证请求失败（HTTP {status}）"))
}

#[derive(Clone)]
pub struct AuthClient {
    base_url: String,
}

impl AuthClient {
    pub fn new(base_url: impl Into<String>) -> Self {
        Self {
            base_url: base_url.into(),
        }
    }

    pub async fn register(
        &self,
        email: &str,
        password: &str,
        display_name: &str,
    ) -> Result<(), AuthError> {
        let url = format!("{}/api/v1/auth/register", self.base_url);
        let resp = Request::post(&url)
            .json(&RegisterReq {
                email: email.to_string(),
                password: password.to_string(),
                display_name: display_name.to_string(),
            })
            .map_err(|e| AuthError::Network(e.to_string()))?
            .send()
            .await
            .map_err(|e| AuthError::Network(e.to_string()))?;

        match resp.status() {
            201 => Ok(()),
            s => Err(AuthError::Server(
                s,
                auth_error_message(s, &resp.text().await.unwrap_or_default()),
            )),
        }
    }

    pub async fn login(&self, email: &str, password: &str) -> Result<AuthSession, AuthError> {
        let url = format!("{}/api/v1/auth/login", self.base_url);
        let resp = Request::post(&url)
            .credentials(RequestCredentials::Include)
            .json(&LoginReq {
                email: email.to_string(),
                password: password.to_string(),
                remember_device: true,
            })
            .map_err(|e| AuthError::Network(e.to_string()))?
            .send()
            .await
            .map_err(|e| AuthError::Network(e.to_string()))?;

        match resp.status() {
            200 => {
                let token: TokenResponse = resp
                    .json()
                    .await
                    .map_err(|e| AuthError::Parse(e.to_string()))?;
                let mut session = AuthSession::from_token(token);
                if let Ok(user) = self.me(&session.access_token).await {
                    session.user = Some(user);
                }
                Ok(session)
            }
            s => Err(AuthError::Server(
                s,
                auth_error_message(s, &resp.text().await.unwrap_or_default()),
            )),
        }
    }

    pub async fn refresh(&self) -> Result<TokenResponse, AuthError> {
        let url = format!("{}/api/v1/auth/refresh", self.base_url);
        let resp = Request::post(&url)
            .credentials(RequestCredentials::Include)
            .send()
            .await
            .map_err(|e| AuthError::Network(e.to_string()))?;

        match resp.status() {
            200 => resp
                .json()
                .await
                .map_err(|e| AuthError::Parse(e.to_string())),
            s => Err(AuthError::Server(
                s,
                auth_error_message(s, &resp.text().await.unwrap_or_default()),
            )),
        }
    }

    pub async fn logout(&self, access_token: &str) -> Result<(), AuthError> {
        let url = format!("{}/api/v1/auth/logout", self.base_url);
        let resp = Request::post(&url)
            .credentials(RequestCredentials::Include)
            .header("Authorization", &format!("Bearer {access_token}"))
            .send()
            .await
            .map_err(|e| AuthError::Network(e.to_string()))?;

        match resp.status() {
            204 => Ok(()),
            s => Err(AuthError::Server(
                s,
                auth_error_message(s, &resp.text().await.unwrap_or_default()),
            )),
        }
    }

    pub async fn me(&self, access_token: &str) -> Result<UserProfile, AuthError> {
        let url = format!("{}/api/v1/auth/me", self.base_url);
        let resp = Request::get(&url)
            .header("Authorization", &format!("Bearer {access_token}"))
            .send()
            .await
            .map_err(|e| AuthError::Network(e.to_string()))?;

        match resp.status() {
            200 => resp
                .json()
                .await
                .map_err(|e| AuthError::Parse(e.to_string())),
            s => Err(AuthError::Server(
                s,
                auth_error_message(s, &resp.text().await.unwrap_or_default()),
            )),
        }
    }

    pub async fn refresh_session(&self, current: &AuthSession) -> Result<AuthSession, AuthError> {
        let token = self.refresh().await?;
        let mut next = AuthSession::from_token(token);
        next.user = current.user.clone();
        Ok(next)
    }
}

/// Backend standard success envelope: `{ code: 0, data: T, request_id: String }`.
/// All 200-OK responses from v1 API use this wrapper.
#[derive(Deserialize)]
struct ApiResp<T> {
    code: i32,
    data: T,
}

/// Inner data for POST /diagrams success envelope (`data.id`).
#[derive(Deserialize)]
struct IdData {
    id: String,
}

/// Save (PUT) specific errors, including the 409 revision conflict.
#[derive(Debug, Error)]
pub enum SaveError {
    #[error("conflict: current_revision={current_revision}, expected={expected_revision}")]
    Conflict {
        current_revision: i64,
        expected_revision: i64,
    },
    #[error("server {0}: {1}")]
    Server(u16, String),
    #[error("network: {0}")]
    Network(String),
}

/// Successful save response — includes server-side revision and timestamp.
#[derive(Debug, Clone)]
pub struct SaveResponse {
    pub revision: i64,
    pub saved_at: DateTime<Utc>,
}

// ---------------------------------------------------------------------------
// DiagramClient
// ---------------------------------------------------------------------------

/// HTTP client for the drawdb v1 REST API.
///
/// Construct with `DiagramClient::new("http://127.0.0.1:3000")`.
/// All methods are async and return `Result` with typed error enums.
///
/// # Example
/// ```ignore
/// let client = DiagramClient::new("http://127.0.0.1:3000");
/// let diagram = client.get("diagram-id-123").await?;
/// ```
/// HTTP client to backend, clone-cheap (inner `String` is owned so clone is cheap).
/// 派生 Clone 以支持 schedule_save helper 跨 spawn_local 边界 clone client
/// （fix-add-frontend-stub-leftover 提案 Bug A 修复需要）
#[derive(Clone)]
pub struct DiagramClient {
    base_url: String,
}

impl DiagramClient {
    /// Create a new client pointed at the given base URL.
    pub fn new(base_url: impl Into<String>) -> Self {
        Self {
            base_url: base_url.into(),
        }
    }

    /// GET /api/v1/diagrams/{id}
    ///
    /// Fetches the full diagram by id. Returns the `Diagram` on success.
    pub async fn get(&self, id: &str) -> Result<Diagram, ApiError> {
        let url = format!("{}/api/v1/diagrams/{}", self.base_url, id);
        let resp = Request::get(&url)
            .send()
            .await
            .map_err(|e| ApiError::Network(e.to_string()))?;

        match resp.status() {
            200 => {
                // Backend wraps success in `{ code: 0, data: DiagramOut, request_id }`
                let out: ApiResp<DiagramOut> = resp
                    .json()
                    .await
                    .map_err(|e| ApiError::Parse(e.to_string()))?;
                Ok(out.data.into_diagram())
            }
            s => Err(ApiError::Server(s, resp.text().await.unwrap_or_default())),
        }
    }

    /// PUT /api/v1/diagrams/{id}
    ///
    /// Saves once with an expected revision for optimistic locking.
    /// On 409 Conflict returns `SaveError::Conflict { current_revision, expected_revision }`.
    pub async fn save(
        &self,
        id: &str,
        expected_revision: i64,
        body: &Diagram,
    ) -> Result<SaveResponse, SaveError> {
        let url = format!("{}/api/v1/diagrams/{}", self.base_url, id);
        let req = SaveReq {
            expected_revision,
            diagram: DiagramForSave::from(body),
        };

        let resp = Request::put(&url)
            .json(&req)
            .map_err(|e| SaveError::Network(e.to_string()))?
            .send()
            .await
            .map_err(|e| SaveError::Network(e.to_string()))?;

        match resp.status() {
            200 => {
                // Backend wraps success in `{ code: 0, data: SaveResp, request_id }`
                let out: ApiResp<SaveResp> = resp
                    .json()
                    .await
                    .map_err(|e| SaveError::Network(e.to_string()))?;
                Ok(SaveResponse {
                    revision: out.data.revision.unwrap_or(expected_revision + 1),
                    saved_at: Utc::now(),
                })
            }
            409 => {
                // Parse { "code": 409, "details": { "current_revision": N } }
                #[derive(Deserialize)]
                struct ConflictDetails {
                    current_revision: i64,
                }
                #[derive(Deserialize)]
                struct ConflictBody {
                    details: Option<ConflictDetails>,
                }
                let body: ConflictBody =
                    resp.json().await.unwrap_or(ConflictBody { details: None });
                let current = body.details.map(|d| d.current_revision).unwrap_or(0);
                Err(SaveError::Conflict {
                    current_revision: current,
                    expected_revision,
                })
            }
            s => Err(SaveError::Server(s, resp.text().await.unwrap_or_default())),
        }
    }

    /// POST /api/v1/diagrams
    ///
    /// Creates a new diagram with the given name. Returns the new diagram id.
    pub async fn create(&self, name: &str) -> Result<String, ApiError> {
        let url = format!("{}/api/v1/diagrams", self.base_url);
        let req = CreateReq {
            name: name.to_string(),
            database: None,
        };

        let resp = Request::post(&url)
            .json(&req)
            .map_err(|e| ApiError::Network(e.to_string()))?
            .send()
            .await
            .map_err(|e| ApiError::Network(e.to_string()))?;

        match resp.status() {
            200 => {
                // Backend wraps success in `{ code: 0, data: { id }, request_id }`
                let out: ApiResp<IdData> = resp
                    .json()
                    .await
                    .map_err(|e| ApiError::Parse(e.to_string()))?;
                Ok(out.data.id)
            }
            s => Err(ApiError::Server(s, resp.text().await.unwrap_or_default())),
        }
    }

    /// DELETE /api/v1/diagrams/{id}
    ///
    /// Permanently deletes the diagram. Returns `Ok(())` on success.
    pub async fn delete(&self, id: &str) -> Result<(), ApiError> {
        let url = format!("{}/api/v1/diagrams/{}", self.base_url, id);
        let resp = Request::delete(&url)
            .send()
            .await
            .map_err(|e| ApiError::Network(e.to_string()))?;

        match resp.status() {
            200 => Ok(()),
            s => Err(ApiError::Server(s, resp.text().await.unwrap_or_default())),
        }
    }

    /// POST /api/v1/bridge/import/local
    ///
    /// 从本地 payload 创建新 diagram（Phase C ImportDrawer）。
    pub async fn import_local(
        &self,
        source: &str,
        payload: serde_json::Value,
    ) -> Result<ImportLocalResponse, ApiError> {
        let url = format!("{}/api/v1/bridge/import/local", self.base_url);
        let req = ImportLocalReq {
            source: source.to_string(),
            payload,
        };

        let resp = Request::post(&url)
            .json(&req)
            .map_err(|e| ApiError::Network(e.to_string()))?
            .send()
            .await
            .map_err(|e| ApiError::Network(e.to_string()))?;

        match resp.status() {
            200 => {
                let out: ApiResp<ImportLocalData> = resp
                    .json()
                    .await
                    .map_err(|e| ApiError::Parse(e.to_string()))?;
                Ok(ImportLocalResponse {
                    diagram_id: out.data.diagram_id,
                    log_id: out.data.log_id,
                    status: out.data.status,
                })
            }
            s => Err(ApiError::Server(s, resp.text().await.unwrap_or_default())),
        }
    }

    /// GET /api/v1/bridge/config
    pub async fn get_bridge_config(&self) -> Result<BridgeConfig, ApiError> {
        let url = format!("{}/api/v1/bridge/config", self.base_url);
        let resp = Request::get(&url)
            .send()
            .await
            .map_err(|e| ApiError::Network(e.to_string()))?;

        match resp.status() {
            200 => {
                let out: ApiResp<BridgeConfig> = resp
                    .json()
                    .await
                    .map_err(|e| ApiError::Parse(e.to_string()))?;
                Ok(out.data)
            }
            s => Err(ApiError::Server(s, resp.text().await.unwrap_or_default())),
        }
    }

    /// PUT /api/v1/bridge/config
    pub async fn update_bridge_config(&self, update: &BridgeConfigUpdate) -> Result<(), ApiError> {
        let url = format!("{}/api/v1/bridge/config", self.base_url);
        let resp = Request::put(&url)
            .json(update)
            .map_err(|e| ApiError::Network(e.to_string()))?
            .send()
            .await
            .map_err(|e| ApiError::Network(e.to_string()))?;

        match resp.status() {
            200 => Ok(()),
            s => Err(ApiError::Server(s, resp.text().await.unwrap_or_default())),
        }
    }

    /// GET /api/v1/bridge/import/local/logs
    pub async fn list_import_logs(
        &self,
        status: Option<&str>,
    ) -> Result<Vec<ImportLogEntry>, ApiError> {
        let mut url = format!("{}/api/v1/bridge/import/local/logs", self.base_url);
        if let Some(st) = status {
            url.push_str(&format!("?status={}", st));
        }
        let resp = Request::get(&url)
            .send()
            .await
            .map_err(|e| ApiError::Network(e.to_string()))?;

        match resp.status() {
            200 => {
                let out: ApiResp<Vec<ImportLogEntry>> = resp
                    .json()
                    .await
                    .map_err(|e| ApiError::Parse(e.to_string()))?;
                Ok(out.data)
            }
            s => Err(ApiError::Server(s, resp.text().await.unwrap_or_default())),
        }
    }

    /// POST /api/v1/bridge/import/local/retry/{id}
    pub async fn retry_import_log(&self, log_id: &str) -> Result<RetryImportResponse, ApiError> {
        let url = format!(
            "{}/api/v1/bridge/import/local/retry/{}",
            self.base_url, log_id
        );
        let resp = Request::post(&url)
            .send()
            .await
            .map_err(|e| ApiError::Network(e.to_string()))?;

        match resp.status() {
            200 => {
                let out: ApiResp<RetryImportData> = resp
                    .json()
                    .await
                    .map_err(|e| ApiError::Parse(e.to_string()))?;
                Ok(RetryImportResponse {
                    log_id: out.data.id,
                    status: out.data.status,
                    diagram_id: out.data.diagram_id,
                    retry_count: out.data.retry_count,
                })
            }
            s => Err(ApiError::Server(s, resp.text().await.unwrap_or_default())),
        }
    }
}

// ---------------------------------------------------------------------------
// Internal types (request/response DTOs)
// ---------------------------------------------------------------------------

/// Matches backend full diagram response (`diagram_persistence::DiagramFull`).
#[derive(Deserialize)]
struct DiagramOut {
    id: String,
    name: Option<String>,
    database: Option<String>,
    pan: Option<String>,
    zoom: Option<String>,
    revision: i64,
    #[serde(default)]
    tables: Vec<TableOut>,
    #[serde(default)]
    references: Vec<ReferenceOut>,
    #[serde(default)]
    areas: Vec<AreaOut>,
    #[serde(default)]
    notes: Vec<NoteOut>,
}

#[derive(Deserialize)]
struct TableOut {
    id: String,
    name: String,
    #[serde(default)]
    x: f64,
    #[serde(default)]
    y: f64,
    #[serde(default)]
    color: String,
    #[serde(default)]
    comment: String,
    #[serde(default)]
    fields: Vec<FieldOut>,
    #[serde(default)]
    indices: Vec<IndexOut>,
}

#[derive(Deserialize)]
struct FieldOut {
    id: String,
    name: String,
    #[serde(default, alias = "type")]
    type_: String,
    #[serde(default)]
    default: String,
    #[serde(default)]
    check: String,
    #[serde(default)]
    primary: bool,
    #[serde(default)]
    unique: bool,
    #[serde(default)]
    not_null: bool,
    #[serde(default)]
    increment: bool,
    #[serde(default)]
    comment: String,
}

#[derive(Deserialize)]
struct IndexOut {
    id: String,
    name: String,
    #[serde(default)]
    fields: Vec<String>,
    #[serde(default)]
    unique: bool,
}

#[derive(Deserialize)]
struct ReferenceOut {
    id: String,
    #[serde(default)]
    name: String,
    start_table_id: String,
    end_table_id: String,
    start_field_id: String,
    end_field_id: String,
    #[serde(default, alias = "type")]
    type_: String,
    #[serde(default)]
    on_delete: String,
    #[serde(default)]
    on_update: String,
}

#[derive(Deserialize)]
struct AreaOut {
    id: String,
    #[serde(default)]
    x: f64,
    #[serde(default)]
    y: f64,
    #[serde(default)]
    width: f64,
    #[serde(default)]
    height: f64,
    #[serde(default)]
    color: String,
    #[serde(default)]
    name: String,
}

#[derive(Deserialize)]
struct NoteOut {
    id: String,
    #[serde(default)]
    x: f64,
    #[serde(default)]
    y: f64,
    #[serde(default)]
    content: String,
    #[serde(default)]
    color: String,
}

impl DiagramOut {
    fn into_diagram(self) -> Diagram {
        Diagram {
            id: self.id,
            name: self.name.unwrap_or_default(),
            revision: self.revision,
            database: self
                .database
                .as_ref()
                .and_then(|d| parse_database(d))
                .unwrap_or(Database::Generic),
            tables: self.tables.into_iter().map(Into::into).collect(),
            references: self.references.into_iter().map(Into::into).collect(),
            notes: self.notes.into_iter().map(Into::into).collect(),
            areas: self.areas.into_iter().map(Into::into).collect(),
        }
    }
}

impl From<TableOut> for Table {
    fn from(t: TableOut) -> Self {
        Table {
            id: t.id,
            name: t.name,
            x: t.x,
            y: t.y,
            color: t.color,
            comment: t.comment,
            fields: t.fields.into_iter().map(Into::into).collect(),
            indices: t.indices.into_iter().map(Into::into).collect(),
        }
    }
}

impl From<FieldOut> for Field {
    fn from(f: FieldOut) -> Self {
        Field {
            id: f.id,
            name: f.name,
            type_: f.type_,
            default: f.default,
            check: f.check,
            primary: f.primary,
            unique: f.unique,
            not_null: f.not_null,
            increment: f.increment,
            comment: f.comment,
        }
    }
}

impl From<IndexOut> for Index {
    fn from(i: IndexOut) -> Self {
        Index {
            id: i.id,
            name: i.name,
            fields: i.fields,
            unique: i.unique,
        }
    }
}

impl From<ReferenceOut> for Reference {
    fn from(r: ReferenceOut) -> Self {
        Reference {
            id: r.id,
            name: r.name,
            start_table_id: r.start_table_id,
            end_table_id: r.end_table_id,
            start_field_id: r.start_field_id,
            end_field_id: r.end_field_id,
            type_: r.type_,
            on_delete: r.on_delete,
            on_update: r.on_update,
        }
    }
}

impl From<AreaOut> for Area {
    fn from(a: AreaOut) -> Self {
        Area {
            id: a.id,
            x: a.x,
            y: a.y,
            width: a.width,
            height: a.height,
            color: a.color,
            name: a.name,
        }
    }
}

impl From<NoteOut> for Note {
    fn from(n: NoteOut) -> Self {
        Note {
            id: n.id,
            x: n.x,
            y: n.y,
            content: n.content,
            color: n.color,
        }
    }
}

fn parse_database(s: &str) -> Option<Database> {
    match s {
        "mysql" => Some(Database::Mysql),
        "postgresql" => Some(Database::Postgresql),
        "sqlite" => Some(Database::Sqlite),
        "mssql" => Some(Database::Mssql),
        "oracle" => Some(Database::Oracle),
        _ => Some(Database::Generic),
    }
}

/// Full diagram body for PUT (nested entities included).
#[derive(Serialize)]
struct DiagramForSave {
    id: String,
    name: String,
    database: String,
    pan: String,
    zoom: String,
    tables: Vec<Table>,
    references: Vec<Reference>,
    areas: Vec<Area>,
    notes: Vec<Note>,
}

impl From<&Diagram> for DiagramForSave {
    fn from(d: &Diagram) -> Self {
        Self {
            id: d.id.clone(),
            name: d.name.clone(),
            database: database_str(&d.database).to_string(),
            pan: String::new(),
            zoom: String::new(),
            tables: d.tables.clone(),
            references: d.references.clone(),
            areas: d.areas.clone(),
            notes: d.notes.clone(),
        }
    }
}

fn database_str(d: &Database) -> &'static str {
    match d {
        Database::Mysql => "mysql",
        Database::Postgresql => "postgresql",
        Database::Sqlite => "sqlite",
        Database::Mssql => "mssql",
        Database::Oracle => "oracle",
        Database::Generic => "",
    }
}

/// Backend SaveReq (PUT body).
#[derive(Serialize)]
struct SaveReq {
    expected_revision: i64,
    diagram: DiagramForSave,
}

/// Backend save response.
#[derive(Deserialize)]
struct SaveResp {
    id: Option<String>,
    revision: Option<i64>,
}

/// POST /diagrams body.
#[derive(Serialize)]
struct CreateReq {
    name: String,
    database: Option<String>,
}

/// POST /bridge/import/local body.
#[derive(Serialize)]
struct ImportLocalReq {
    source: String,
    payload: serde_json::Value,
}

#[derive(Deserialize)]
struct ImportLocalData {
    log_id: String,
    diagram_id: String,
    status: String,
}

/// Phase C：bridge 本地导入响应
#[derive(Debug, Clone)]
pub struct ImportLocalResponse {
    pub diagram_id: String,
    pub log_id: String,
    pub status: String,
}

/// Bridge 配置（对齐 `phase3_bridge::BridgeConfig`）
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BridgeConfig {
    pub db_read_preferred: bool,
    pub db_write_enabled: bool,
    pub dual_write_local: bool,
    pub updated_at: String,
}

/// PUT bridge/config 请求体（字段可选）
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct BridgeConfigUpdate {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub db_read_preferred: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub db_write_enabled: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dual_write_local: Option<bool>,
}

/// 导入日志条目
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ImportLogEntry {
    pub id: String,
    #[serde(default)]
    pub source: Option<String>,
    #[serde(default)]
    pub imported_diagram_id: Option<String>,
    pub status: String,
    #[serde(default)]
    pub retry_count: i64,
    #[serde(default)]
    pub error_message: Option<String>,
    #[serde(default)]
    pub created_at: Option<String>,
    #[serde(default)]
    pub updated_at: Option<String>,
}

/// 重试导入响应
#[derive(Debug, Clone)]
pub struct RetryImportResponse {
    pub log_id: String,
    pub status: String,
    pub diagram_id: Option<String>,
    pub retry_count: i64,
}

#[derive(Deserialize)]
struct RetryImportData {
    id: String,
    status: String,
    #[serde(default)]
    diagram_id: Option<String>,
    #[serde(default)]
    retry_count: i64,
}

/// 自动保存重试间隔（ms）：对齐 Phase 2 / Phase 3 S01 — 3s / 6s / 12s，累计封顶 30s。
pub const SAVE_RETRY_DELAYS_MS: [u32; 3] = [3000, 6000, 12000];
pub const SAVE_RETRY_MAX_ELAPSED_MS: u32 = 30_000;

/// 409 冲突不重试；网络 / 5xx 可重试。
pub fn is_retriable_save_error(err: &SaveError) -> bool {
    match err {
        SaveError::Conflict { .. } => false,
        SaveError::Network(_) => true,
        SaveError::Server(status, _) => *status >= 500,
    }
}

/// PUT with exponential backoff (initial attempt + up to 3 retries).
pub async fn save_with_retry(
    client: &DiagramClient,
    id: &str,
    expected_revision: i64,
    body: &Diagram,
) -> Result<SaveResponse, SaveError> {
    let mut attempt = 0usize;
    let mut elapsed_ms = 0u32;
    loop {
        match client.save(id, expected_revision, body).await {
            Ok(resp) => return Ok(resp),
            Err(e) if is_retriable_save_error(&e) => {
                if attempt >= SAVE_RETRY_DELAYS_MS.len() {
                    return Err(e);
                }
                let delay = SAVE_RETRY_DELAYS_MS[attempt];
                if elapsed_ms.saturating_add(delay) > SAVE_RETRY_MAX_ELAPSED_MS {
                    return Err(e);
                }
                gloo_timers::future::TimeoutFuture::new(delay).await;
                elapsed_ms = elapsed_ms.saturating_add(delay);
                attempt += 1;
            }
            Err(e) => return Err(e),
        }
    }
}

/// 预期尝试次数：首次 + 重试次数（用于 UT 断言）。
pub fn save_retry_total_attempts() -> usize {
    1 + SAVE_RETRY_DELAYS_MS.len()
}

#[cfg(test)]
mod bridge_api_tests {
    use super::*;
    use serde::Deserialize;

    #[derive(Deserialize)]
    struct TestApiResp<T> {
        code: i32,
        data: T,
    }

    /// UT-ALIGN-B01: bridge config GET/PUT envelope 解析
    #[test]
    fn ut_align_b01_bridge_config_envelope_parse() {
        let json = r#"{"code":0,"data":{"db_read_preferred":false,"db_write_enabled":true,"dual_write_local":false,"updated_at":"2026-06-21"},"request_id":"r1"}"#;
        let out: TestApiResp<BridgeConfig> = serde_json::from_str(json).unwrap();
        assert_eq!(out.code, 0);
        assert!(!out.data.db_read_preferred);
        assert!(out.data.db_write_enabled);
        assert!(!out.data.dual_write_local);
        assert_eq!(out.data.updated_at, "2026-06-21");
    }

    #[test]
    fn ut_align_b01_bridge_config_update_envelope_parse() {
        let json = r#"{"code":0,"data":{"updated":true},"request_id":"r2"}"#;
        #[derive(Deserialize)]
        struct UpdateResult {
            updated: bool,
        }
        let out: TestApiResp<UpdateResult> = serde_json::from_str(json).unwrap();
        assert!(out.data.updated);
    }

    #[test]
    fn ut_align_b01_bridge_config_update_partial_serialize() {
        let update = BridgeConfigUpdate {
            dual_write_local: Some(true),
            ..Default::default()
        };
        let v = serde_json::to_value(&update).unwrap();
        assert_eq!(v["dual_write_local"], true);
        assert!(v.get("db_read_preferred").is_none());
    }

    /// UT-ALIGN-B02: 导入日志列表与重试 envelope 解析
    #[test]
    fn ut_align_b02_import_logs_envelope_parse() {
        let json = r#"{"code":0,"data":[{"id":"log-1","status":"failed","retry_count":0,"error_message":"parse error"}],"request_id":"r1"}"#;
        let out: TestApiResp<Vec<ImportLogEntry>> = serde_json::from_str(json).unwrap();
        assert_eq!(out.data.len(), 1);
        assert_eq!(out.data[0].id, "log-1");
        assert_eq!(out.data[0].status, "failed");
        assert_eq!(out.data[0].retry_count, 0);
        assert_eq!(out.data[0].error_message.as_deref(), Some("parse error"));
    }

    #[test]
    fn ut_align_b02_retry_import_envelope_parse() {
        let json = r#"{"code":0,"data":{"id":"log-1","status":"success","retry_count":1,"diagram_id":"d-new"},"request_id":"r2"}"#;
        #[derive(Deserialize)]
        struct RetryData {
            id: String,
            status: String,
            retry_count: i64,
            diagram_id: Option<String>,
        }
        let out: TestApiResp<RetryData> = serde_json::from_str(json).unwrap();
        assert_eq!(out.data.id, "log-1");
        assert_eq!(out.data.status, "success");
        assert_eq!(out.data.retry_count, 1);
        assert_eq!(out.data.diagram_id.as_deref(), Some("d-new"));
    }
}

#[cfg(test)]
mod save_retry_tests {
    use super::{save_retry_total_attempts, SAVE_RETRY_DELAYS_MS, SAVE_RETRY_MAX_ELAPSED_MS};

    #[test]
    fn ut_s01_09_retry_delays_match_spec() {
        assert_eq!(SAVE_RETRY_DELAYS_MS, [3000, 6000, 12000]);
        assert_eq!(SAVE_RETRY_MAX_ELAPSED_MS, 30_000);
        assert_eq!(save_retry_total_attempts(), 4);
    }
}

#[cfg(test)]
mod auth_tests {
    use super::{
        auth_error_message, is_token_expired_error, AuthSession, TokenResponse, UserProfile,
    };

    #[test]
    fn ut_fe_s03_02_auth_token_response_parse() {
        let json = r#"{"accessToken":"jwt-1","expiresIn":900,"tokenType":"Bearer"}"#;
        let token: TokenResponse = serde_json::from_str(json).unwrap();
        let session = AuthSession::from_token(token);
        assert_eq!(session.bearer_header(), "Bearer jwt-1");
        assert_eq!(session.expires_in, 900);
    }

    #[test]
    fn ut_fe_s03_03_user_profile_display_name() {
        let session = AuthSession {
            access_token: "jwt-1".into(),
            expires_in: 900,
            token_type: "Bearer".into(),
            user: Some(UserProfile {
                id: "u1".into(),
                email: "dev@example.com".into(),
                display_name: Some("Dev".into()),
                email_verified_at: None,
            }),
        };
        assert_eq!(session.display_name(), "Dev");
    }

    #[test]
    fn ut_fe_s03_04_token_expired_detection() {
        assert!(is_token_expired_error(
            401,
            r#"{"code":"token_expired","message":"Access token expired"}"#
        ));
        assert!(!is_token_expired_error(
            403,
            r#"{"code":"token_expired","message":"Access token expired"}"#
        ));
    }

    #[test]
    fn ut_fe_s03_05_auth_error_message_is_sanitized() {
        assert_eq!(
            auth_error_message(
                401,
                r#"{"code":"INVALID_CREDENTIALS","message":"邮箱或密码错误"}"#
            ),
            "邮箱或密码错误"
        );
        assert_eq!(
            auth_error_message(500, "not-json"),
            "认证请求失败（HTTP 500）"
        );
    }
}

#[allow(dead_code)]
pub fn init() {}
