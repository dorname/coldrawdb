use reqwest::StatusCode;
use serde::Serialize;
use serde_json::{Map, Value};

#[derive(Clone, Debug, Serialize, PartialEq)]
pub struct ToolError {
    pub code: String,
    pub message: String,
    pub retryable: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub request_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<Value>,
}

impl ToolError {
    pub fn new(code: &str, message: impl Into<String>, retryable: bool) -> Self {
        Self {
            code: code.into(),
            message: message.into(),
            retryable,
            request_id: None,
            details: None,
        }
    }

    pub fn config(message: impl Into<String>) -> Self {
        Self::new("CONFIG_INVALID", message, false)
    }

    pub fn validation(message: impl Into<String>) -> Self {
        Self::new("VALIDATION_ERROR", message, false)
    }

    pub fn upstream(status: StatusCode, body: &Value) -> Self {
        let (code, retryable) = match status.as_u16() {
            400 | 422 => ("VALIDATION_ERROR", false),
            401 => ("UNAUTHENTICATED", false),
            403 => ("PERMISSION_DENIED", false),
            404 => ("NOT_FOUND", false),
            409 => ("REVISION_CONFLICT", false),
            500..=599 => ("UPSTREAM_ERROR", true),
            _ => ("UPSTREAM_ERROR", false),
        };
        let message = body
            .get("message")
            .and_then(Value::as_str)
            .unwrap_or("上游请求失败");
        let request_id = body
            .get("request_id")
            .and_then(Value::as_str)
            .map(str::to_owned);
        let details = body
            .get("details")
            .and_then(Value::as_object)
            .map(|source| {
                let mut safe = Map::new();
                for key in ["current_revision", "field"] {
                    if let Some(value) = source.get(key) {
                        safe.insert(key.into(), value.clone());
                    }
                }
                Value::Object(safe)
            });
        Self {
            code: code.into(),
            message: message.into(),
            retryable,
            request_id,
            details,
        }
    }

    pub fn transport(error: &reqwest::Error) -> Self {
        if error.is_timeout() {
            Self::new("UPSTREAM_TIMEOUT", "上游请求超时", true)
        } else {
            Self::new("UPSTREAM_UNAVAILABLE", "无法连接上游服务", true)
        }
    }
}
