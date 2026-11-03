use std::sync::{Arc, Mutex};

use reqwest::{Client, Method, Response};
use serde_json::{json, Value};

use crate::collab;
use crate::config::Config;
use crate::error::ToolError;

#[derive(Default)]
struct DynamicSession {
    access_token: Option<String>,
    refresh_cookie: Option<String>,
}

#[derive(Clone)]
pub struct ApiClient {
    config: Config,
    client: Client,
    // 动态 access token 与 refresh cookie 仅存在于当前 MCP 进程内。
    dynamic_session: Arc<Mutex<DynamicSession>>,
}

impl ApiClient {
    pub fn new(config: Config) -> Result<Self, ToolError> {
        let client = Client::builder()
            .timeout(config.timeout)
            // fix-mcp-server-test-proxy: 显式禁用 reqwest 环境代理。
            // 测试(mock_response 直连 127.0.0.1:0)与生产(自托管部署走本地
            // 后端 127.0.0.1)均不应走环境代理;reqwest 默认读 HTTPS_PROXY/
            // HTTP_PROXY 且 NO_PROXY=127.* 不被识别为合法 CIDR。
            .no_proxy()
            .build()
            .map_err(|_| ToolError::config("无法初始化 HTTP 客户端"))?;
        Ok(Self {
            config,
            client,
            dynamic_session: Arc::new(Mutex::new(DynamicSession::default())),
        })
    }

    /// fix-open-issues-26-33（#26）：update_diagram 参数体积软上限（字节）。
    pub fn payload_soft_limit_bytes(&self) -> usize {
        self.config.payload_soft_limit_bytes
    }

    fn session(&self) -> std::sync::MutexGuard<'_, DynamicSession> {
        self.dynamic_session
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    fn secrets(&self) -> Vec<String> {
        let session = self.session();
        let mut secrets = vec![
            self.config.access_token.clone(),
            self.config.auth_password.clone(),
            session.access_token.clone(),
            session.refresh_cookie.clone(),
        ];
        secrets.push(self.config.auth_email.clone());
        let mut secrets: Vec<String> = secrets
            .into_iter()
            .flatten()
            .filter(|secret| !secret.is_empty())
            .collect();
        secrets.sort_by_key(|secret| std::cmp::Reverse(secret.len()));
        secrets.dedup();
        secrets
    }

    fn redact_text(text: &str, secrets: &[String]) -> String {
        let mut redacted = text.to_string();
        for secret in secrets {
            redacted = redacted.replace(secret, "[REDACTED]");
        }
        redacted
    }

    fn redact_value(value: &mut Value, secrets: &[String]) {
        match value {
            Value::String(text) => *text = Self::redact_text(text, secrets),
            Value::Array(items) => {
                for item in items {
                    Self::redact_value(item, secrets);
                }
            }
            Value::Object(items) => {
                for item in items.values_mut() {
                    Self::redact_value(item, secrets);
                }
            }
            _ => {}
        }
    }

    fn redact_sensitive_fields(value: &mut Value) {
        match value {
            Value::Array(items) => {
                for item in items {
                    Self::redact_sensitive_fields(item);
                }
            }
            Value::Object(items) => {
                for (key, item) in items {
                    let normalized = key.to_ascii_lowercase().replace('-', "_");
                    if matches!(
                        normalized.as_str(),
                        "password"
                            | "authorization"
                            | "cookie"
                            | "set_cookie"
                            | "token"
                            | "access_token"
                            | "accesstoken"
                            | "refresh_token"
                            | "refreshtoken"
                    ) {
                        *item = json!("[REDACTED]");
                    } else {
                        Self::redact_sensitive_fields(item);
                    }
                }
            }
            _ => {}
        }
    }

    fn redact_error(&self, mut error: ToolError) -> ToolError {
        let secrets = self.secrets();
        error.message = Self::redact_text(&error.message, &secrets);
        if let Some(request_id) = &mut error.request_id {
            *request_id = Self::redact_text(request_id, &secrets);
        }
        if let Some(details) = &mut error.details {
            Self::redact_value(details, &secrets);
        }
        error
    }

    fn refresh_cookie(response: &Response) -> Option<String> {
        response
            .headers()
            .get_all(reqwest::header::SET_COOKIE)
            .iter()
            .filter_map(|value| value.to_str().ok())
            .find_map(|value| {
                value
                    .split(';')
                    .next()
                    .and_then(|pair| pair.strip_prefix("refresh_token="))
                    .map(str::to_string)
            })
    }

    async fn decode_response(&self, response: Response) -> Result<Value, ToolError> {
        let status = response.status();
        // fix-issues-42-44（#42 R-MCPERR-01）：先取原文再解析——非 JSON 响应（网关
        // 502/504 HTML 页、代理拦截页等）必须保留 status/Content-Type/body 摘要供诊断。
        let content_type = response
            .headers()
            .get(reqwest::header::CONTENT_TYPE)
            .and_then(|value| value.to_str().ok())
            .unwrap_or("")
            .to_string();
        let text = response.text().await.unwrap_or_default();
        let parsed = serde_json::from_str::<Value>(&text).ok();
        let value = parsed.clone().unwrap_or_else(|| {
            json!({"message":format!("上游返回非 JSON 响应（HTTP {}，Content-Type: {}）", status.as_u16(), content_type)})
        });
        if !status.is_success() {
            let mut safe_value = value;
            Self::redact_sensitive_fields(&mut safe_value);
            let mut error = ToolError::upstream(status, &safe_value);
            let mut details = error
                .details
                .take()
                .and_then(|value| value.as_object().cloned())
                .unwrap_or_default();
            details.insert("http_status".into(), json!(status.as_u16()));
            details.insert("content_type".into(), json!(content_type));
            // 上游可能回显秘密：先替换所有已知凭据，再截取安全摘要。
            let safe_text = if parsed.is_some() {
                serde_json::to_string(&safe_value).unwrap_or_default()
            } else {
                let lower = text.to_ascii_lowercase();
                if ["password", "authorization", "cookie", "token"]
                    .iter()
                    .any(|marker| lower.contains(marker))
                {
                    "[REDACTED: 非 JSON 响应含凭据样字段]".to_string()
                } else {
                    Self::redact_text(&text, &self.secrets())
                }
            };
            let excerpt: String = safe_text.chars().take(200).collect();
            details.insert("body_excerpt".into(), json!(excerpt));
            error.details = Some(Value::Object(details));
            // R-MCPERR-02：409 且 body 非 JSON 但文本含 USE_OP_CHANNEL 字样时，
            // 仍识别为房间图收编 → 走 WS op 通道回退。
            if status.as_u16() == 409
                && error.code != "USE_OP_CHANNEL"
                && text.contains("USE_OP_CHANNEL")
            {
                error.code = "USE_OP_CHANNEL".into();
                error.message = "房间图表请通过协作 op 通道写入".into();
            }
            return Err(self.redact_error(error));
        }
        Ok(value)
    }

    async fn send_once(
        &self,
        method: Method,
        path: &str,
        body: Option<&Value>,
        token: Option<&str>,
    ) -> Result<Value, ToolError> {
        let mut request = self.client.request(method, self.config.endpoint(path)?);
        if let Some(token) = token.filter(|token| !token.is_empty()) {
            request = request.bearer_auth(token);
        }
        if let Some(body) = body {
            request = request.json(body);
        }
        let response = request
            .send()
            .await
            .map_err(|error| ToolError::transport(&error, &self.config.base_url))?;
        self.decode_response(response).await
    }

    fn save_auth_tokens(&self, access_token: String, refresh_cookie: Option<String>) {
        let mut session = self.session();
        session.access_token = Some(access_token);
        if let Some(cookie) = refresh_cookie {
            session.refresh_cookie = if cookie.is_empty() {
                None
            } else {
                Some(cookie)
            };
        }
    }

    async fn login(&self) -> Result<(), ToolError> {
        let (Some(email), Some(password)) = (
            self.config.auth_email.as_deref(),
            self.config.auth_password.as_deref(),
        ) else {
            return Ok(());
        };
        let response = self
            .client
            .post(self.config.endpoint("/api/v1/auth/login")?)
            .json(&json!({"email":email,"password":password}))
            .send()
            .await
            .map_err(|error| ToolError::transport(&error, &self.config.base_url))?;
        let cookie = Self::refresh_cookie(&response);
        let value = self.decode_response(response).await?;
        let access_token = value
            .get("accessToken")
            .and_then(Value::as_str)
            .filter(|token| !token.is_empty())
            .ok_or_else(|| ToolError::new("UPSTREAM_ERROR", "login 响应缺少 accessToken", false))?;
        let Some(cookie) = cookie.filter(|cookie| !cookie.is_empty()) else {
            return Err(ToolError::new(
                "UPSTREAM_ERROR",
                "login 响应缺少 refresh cookie",
                false,
            ));
        };
        self.save_auth_tokens(access_token.to_string(), Some(cookie));
        Ok(())
    }

    async fn refresh(&self) -> Result<(), ToolError> {
        let cookie = self.session().refresh_cookie.clone().ok_or_else(|| {
            ToolError::new(
                "UNAUTHENTICATED",
                "动态会话缺少内存 refresh cookie，请重启 MCP 并重新登录",
                false,
            )
        })?;
        // 固定 refresh 端点绝不经过 request()，避免 refresh 自身 401 再次触发 refresh。
        let response = self
            .client
            .post(self.config.endpoint("/api/v1/auth/refresh")?)
            .header(reqwest::header::COOKIE, format!("refresh_token={cookie}"))
            .send()
            .await
            .map_err(|error| ToolError::transport(&error, &self.config.base_url))?;
        let rotated_cookie = Self::refresh_cookie(&response);
        let value = self.decode_response(response).await?;
        let access_token = value
            .get("accessToken")
            .and_then(Value::as_str)
            .filter(|token| !token.is_empty())
            .ok_or_else(|| {
                ToolError::new("UPSTREAM_ERROR", "refresh 响应缺少 accessToken", false)
            })?;
        self.save_auth_tokens(access_token.to_string(), rotated_cookie);
        Ok(())
    }

    async fn access_token(&self) -> Result<Option<String>, ToolError> {
        if let Some(token) = &self.config.access_token {
            return Ok(Some(token.clone()));
        }
        if !self.config.dynamic_auth_enabled() {
            return Ok(None);
        }
        if let Some(token) = self.session().access_token.clone() {
            return Ok(Some(token));
        }
        self.login().await?;
        Ok(self.session().access_token.clone())
    }

    async fn request(
        &self,
        method: Method,
        path: &str,
        body: Option<Value>,
    ) -> Result<Value, ToolError> {
        let mut refreshed = false;
        loop {
            let token = self.access_token().await?;
            let result = self
                .send_once(method.clone(), path, body.as_ref(), token.as_deref())
                .await;
            match result {
                Err(error)
                    if error.code == "UNAUTHENTICATED"
                        && self.config.dynamic_auth_enabled()
                        && !refreshed =>
                {
                    self.refresh().await?;
                    refreshed = true;
                }
                result => return result,
            }
        }
    }

    pub async fn list(&self, query: Option<&str>, limit: usize) -> Result<Value, ToolError> {
        let response = self
            .request(Method::GET, "/diagrams/queryAll", None)
            .await?;
        normalize_list(&response, query, limit)
    }

    pub async fn get(&self, id: &str) -> Result<Value, ToolError> {
        let response = self
            .request(Method::GET, &format!("/api/v1/diagrams/{id}"), None)
            .await?;
        let diagram = response
            .get("data")
            .cloned()
            .ok_or_else(|| ToolError::new("UPSTREAM_ERROR", "上游响应缺少 data", false))?;
        Ok(json!({"diagram": diagram}))
    }

    pub async fn create(&self, name: &str, database: Option<&str>) -> Result<Value, ToolError> {
        let mut body = json!({"name": name});
        if let Some(database) = database {
            body["database"] = json!(database);
        }
        let response = self
            .request(Method::POST, "/api/v1/diagrams", Some(body))
            .await?;
        response
            .get("data")
            .cloned()
            .ok_or_else(|| ToolError::new("UPSTREAM_ERROR", "上游响应缺少 data", false))
    }

    pub async fn update(
        &self,
        id: &str,
        expected_revision: i64,
        diagram: Value,
    ) -> Result<Value, ToolError> {
        let response = self
            .request(
                Method::PUT,
                &format!("/api/v1/diagrams/{id}"),
                Some(json!({"expected_revision":expected_revision,"diagram":diagram.clone()})),
            )
            .await;
        match response {
            Ok(response) => response
                .get("data")
                .cloned()
                .ok_or_else(|| ToolError::new("UPSTREAM_ERROR", "上游响应缺少 data", false)),
            Err(error) if error.code == "USE_OP_CHANNEL" => {
                self.write_room_diagram(id, expected_revision, diagram)
                    .await
            }
            Err(error) => Err(error),
        }
    }

    fn rooms_auth_required(reason: &str, source: Option<ToolError>) -> ToolError {
        let mut details = source
            .as_ref()
            .and_then(|error| error.details.clone())
            .and_then(|value| value.as_object().cloned())
            .unwrap_or_default();
        details.insert("reason".into(), json!(reason));
        if let Some(source) = &source {
            details.insert("upstream_code".into(), json!(source.code));
        }
        let mut error = ToolError::new(
            "ROOMS_AUTH_REQUIRED",
            if reason == "TOKEN_MISSING" {
                "房间图 MCP 写入需要 COLDRAWDB_ACCESS_TOKEN，或成对配置 COLDRAWDB_AUTH_EMAIL / COLDRAWDB_AUTH_PASSWORD"
            } else {
                "房间图 MCP 写入凭据无效或已过期，请更新静态 token 或检查动态 login/refresh 配置"
            },
            false,
        );
        error.details = Some(Value::Object(details));
        error.request_id = source.and_then(|error| error.request_id);
        error
    }

    fn map_room_auth_error(error: ToolError) -> ToolError {
        if matches!(
            error.code.as_str(),
            "UNAUTHENTICATED" | "WS_HANDSHAKE_UNAUTHORIZED"
        ) {
            Self::rooms_auth_required("TOKEN_INVALID_OR_EXPIRED", Some(error))
        } else {
            error
        }
    }

    /// 房间图 PUT 被收编后，按当前物化文档 diff，经 WebSocket 提交 op。
    async fn write_room_diagram(
        &self,
        id: &str,
        expected_revision: i64,
        desired: Value,
    ) -> Result<Value, ToolError> {
        // 匿名回退在 rooms 请求前失败；动态 login 失败也归入房间鉴权上下文。
        let token = self
            .access_token()
            .await
            .map_err(Self::map_room_auth_error)?;
        if token.as_deref().unwrap_or("").is_empty() {
            return Err(Self::rooms_auth_required("TOKEN_MISSING", None));
        }
        self.write_room_diagram_authenticated(id, expected_revision, desired)
            .await
            .map_err(Self::map_room_auth_error)
    }

    async fn write_room_diagram_authenticated(
        &self,
        id: &str,
        expected_revision: i64,
        desired: Value,
    ) -> Result<Value, ToolError> {
        let rooms = self.request(Method::GET, "/api/v1/rooms", None).await?;
        let current = self
            .request(Method::GET, &format!("/api/v1/diagrams/{id}"), None)
            .await?;
        let base = current
            .get("data")
            .cloned()
            .ok_or_else(|| ToolError::new("UPSTREAM_ERROR", "上游响应缺少 data", false))?;
        let base_revision = base
            .get("revision")
            .and_then(Value::as_i64)
            .ok_or_else(|| ToolError::new("UPSTREAM_ERROR", "物化文档缺少 revision", false))?;
        if base_revision != expected_revision {
            let mut error = ToolError::new("REVISION_CONFLICT", "revision 与服务端不一致", false);
            error.details = Some(json!({"current_revision": base_revision}));
            return Err(error);
        }
        let token = self.access_token().await?;
        let (room_id, ops, revision) = collab::plan_room_save(
            self.config.base_url.as_str(),
            token.as_deref(),
            &rooms,
            id,
            base_revision,
            &base,
            &desired,
        )
        .map_err(|(code, message)| ToolError::new(&code, message, false))?;
        if ops.is_empty() {
            return Ok(json!({"id": id, "revision": revision}));
        }
        let token = token.as_deref().unwrap_or("");
        let ws_url = collab::collab_ws_url(self.config.base_url.as_str(), &room_id, token)
            .map_err(|code| ToolError::new(&code, "无法构造协作 WebSocket 地址", false))?;
        let mut result = collab::transport::submit_over_websocket(&ws_url, &ops, revision).await;
        if matches!(&result, Err((code, _)) if code == "WS_HANDSHAKE_UNAUTHORIZED")
            && self.config.dynamic_auth_enabled()
        {
            // 只有 connect_async 会返回该内部码；op 已发送后的失败不会进入此恢复分支。
            self.refresh().await?;
            let token = self.access_token().await?;
            let ws_url = collab::collab_ws_url(
                self.config.base_url.as_str(),
                &room_id,
                token.as_deref().unwrap_or(""),
            )
            .map_err(|code| ToolError::new(&code, "无法构造协作 WebSocket 地址", false))?;
            result = collab::transport::submit_over_websocket(&ws_url, &ops, revision).await;
        }
        let server_rev = result.map_err(|(code, message)| {
            ToolError::new(&code, message, code == "UPSTREAM_UNAVAILABLE")
        })?;
        Ok(json!({"id": id, "revision": server_rev}))
    }

    pub async fn delete(&self, id: &str) -> Result<Value, ToolError> {
        let response = self
            .request(Method::DELETE, &format!("/api/v1/diagrams/{id}"), None)
            .await?;
        let id = response
            .pointer("/data/id")
            .cloned()
            .unwrap_or_else(|| json!(id));
        Ok(json!({"id":id,"deleted":true}))
    }

    pub async fn import(&self, source: Option<&str>, payload: Value) -> Result<Value, ToolError> {
        let response = self
            .request(
                Method::POST,
                "/api/v1/diagrams/import",
                Some(json!({"source":source.unwrap_or("mcp"),"payload":payload})),
            )
            .await?;
        response
            .get("data")
            .cloned()
            .ok_or_else(|| ToolError::new("UPSTREAM_ERROR", "上游响应缺少 data", false))
    }
}

pub fn normalize_list(
    response: &Value,
    query: Option<&str>,
    limit: usize,
) -> Result<Value, ToolError> {
    let rows = response
        .get("data")
        .and_then(Value::as_array)
        .ok_or_else(|| ToolError::new("UPSTREAM_ERROR", "上游列表响应缺少 data 数组", false))?;
    let needle = query.unwrap_or("").to_lowercase();
    let mut items: Vec<Value> = rows.iter().filter_map(|row| {
        let id = row.get("id")?.as_str()?;
        let name = row.get("name").and_then(Value::as_str).unwrap_or("");
        if !needle.is_empty() && !name.to_lowercase().contains(&needle) { return None; }
        Some(json!({
            "id": id,
            "name": name,
            "database": row.get("database").cloned().unwrap_or(Value::Null),
            "revision": row.get("revision").and_then(Value::as_i64).unwrap_or(0),
            "updated_at": row.get("updated_at").or_else(|| row.get("lastModified")).cloned().unwrap_or(Value::Null)
        }))
    }).collect();
    items.sort_by(|a, b| {
        a.get("name")
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_lowercase()
            .cmp(
                &b.get("name")
                    .and_then(Value::as_str)
                    .unwrap_or("")
                    .to_lowercase(),
            )
    });
    items.truncate(limit.min(100));
    Ok(json!({"count":items.len(),"items":items}))
}
