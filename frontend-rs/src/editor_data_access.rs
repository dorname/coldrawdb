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

use crate::editor_core::types::{Database, Diagram};
use chrono::{DateTime, Utc};
use gloo_net::http::Request;
use serde::{Deserialize, Serialize};
use thiserror::Error;

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
            s => Err(ApiError::Server(
                s,
                resp.text().await.unwrap_or_default(),
            )),
        }
    }

    /// PUT /api/v1/diagrams/{id}
    ///
    /// Saves the diagram with an expected revision for optimistic locking.
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
                let body: ConflictBody = resp
                    .json()
                    .await
                    .unwrap_or(ConflictBody { details: None });
                let current = body
                    .details
                    .map(|d| d.current_revision)
                    .unwrap_or(0);
                Err(SaveError::Conflict {
                    current_revision: current,
                    expected_revision,
                })
            }
            s => Err(SaveError::Server(
                s,
                resp.text().await.unwrap_or_default(),
            )),
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
            s => Err(ApiError::Server(
                s,
                resp.text().await.unwrap_or_default(),
            )),
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
            s => Err(ApiError::Server(
                s,
                resp.text().await.unwrap_or_default(),
            )),
        }
    }
}

// ---------------------------------------------------------------------------
// Internal types (request/response DTOs)
// ---------------------------------------------------------------------------

/// Matches backend `DiagramOut` (backend/src/diagrams_v1.rs:61-68).
#[derive(Deserialize)]
struct DiagramOut {
    id: String,
    name: Option<String>,
    database: Option<String>,
    pan: Option<String>,
    zoom: Option<String>,
    revision: i64,
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
            tables: Vec::new(),
            references: Vec::new(),
            notes: Vec::new(),
            areas: Vec::new(),
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

/// Lightweight diagram sent on PUT (only fields backend cares about).
#[derive(Serialize)]
struct DiagramForSave<'a> {
    id: &'a str,
    name: &'a str,
    database: &'a str,
    pan: &'a str,
    zoom: &'a str,
}

impl<'a> From<&'a Diagram> for DiagramForSave<'a> {
    fn from(d: &'a Diagram) -> Self {
        Self {
            id: &d.id,
            name: &d.name,
            database: database_str(&d.database),
            pan: "",
            zoom: "",
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
struct SaveReq<'a> {
    expected_revision: i64,
    diagram: DiagramForSave<'a>,
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

#[allow(dead_code)]
pub fn init() {}