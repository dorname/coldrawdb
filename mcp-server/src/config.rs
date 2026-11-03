use std::collections::HashMap;
use std::time::Duration;

use url::Url;

use crate::error::ToolError;

#[derive(Clone)]
pub struct Config {
    pub base_url: Url,
    pub access_token: Option<String>,
    pub auth_email: Option<String>,
    pub auth_password: Option<String>,
    pub timeout: Duration,
    /// fix-open-issues-26-33（#26）：update_diagram 参数体积软上限（字节）。
    pub payload_soft_limit_bytes: usize,
}

impl std::fmt::Debug for Config {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("Config")
            .field("base_url", &self.base_url)
            .field(
                "access_token",
                &self.access_token.as_ref().map(|_| "[REDACTED]"),
            )
            .field(
                "auth_email",
                &self.auth_email.as_ref().map(|_| "[REDACTED]"),
            )
            .field(
                "auth_password",
                &self.auth_password.as_ref().map(|_| "[REDACTED]"),
            )
            .field("timeout", &self.timeout)
            .field("payload_soft_limit_bytes", &self.payload_soft_limit_bytes)
            .finish()
    }
}

impl Config {
    pub fn from_env() -> Result<Self, ToolError> {
        Self::from_values(std::env::vars().collect())
    }

    pub fn from_values(values: HashMap<String, String>) -> Result<Self, ToolError> {
        let raw = values
            .get("COLDRAWDB_BASE_URL")
            .filter(|value| !value.trim().is_empty())
            .ok_or_else(|| ToolError::config("缺少 COLDRAWDB_BASE_URL"))?;
        let mut base_url =
            Url::parse(raw).map_err(|_| ToolError::config("COLDRAWDB_BASE_URL 不是有效 URI"))?;
        if !matches!(base_url.scheme(), "http" | "https") || base_url.cannot_be_a_base() {
            return Err(ToolError::config("COLDRAWDB_BASE_URL 仅允许 http 或 https"));
        }
        if !base_url.username().is_empty() || base_url.password().is_some() {
            return Err(ToolError::config("COLDRAWDB_BASE_URL 禁止包含 userinfo"));
        }
        base_url.set_query(None);
        base_url.set_fragment(None);
        let trimmed = base_url.path().trim_end_matches('/').to_string();
        base_url.set_path(&trimmed);

        let timeout_secs = values
            .get("COLDRAWDB_REQUEST_TIMEOUT_SECS")
            .map(|value| {
                value
                    .parse::<u64>()
                    .map_err(|_| ToolError::config("请求超时必须是 1～120 的整数"))
            })
            .transpose()?
            .unwrap_or(30);
        if !(1..=120).contains(&timeout_secs) {
            return Err(ToolError::config("请求超时必须是 1～120 的整数"));
        }

        // fix-open-issues-26-33（#26 / UT-MCP-31）：软上限非法值回退默认并 warn（stderr）。
        let (payload_soft_limit_bytes, soft_limit_warn) = crate::payload_limit::parse_soft_limit(
            values
                .get(crate::payload_limit::ENV_SOFT_LIMIT)
                .map(String::as_str),
        );
        if let Some(warn) = soft_limit_warn {
            eprintln!("coldrawdb-mcp config: {warn}");
        }

        let access_token = values
            .get("COLDRAWDB_ACCESS_TOKEN")
            .filter(|value| !value.is_empty())
            .cloned();
        let auth_email = values
            .get("COLDRAWDB_AUTH_EMAIL")
            .filter(|value| !value.trim().is_empty())
            .cloned();
        let auth_password = values
            .get("COLDRAWDB_AUTH_PASSWORD")
            .filter(|value| !value.is_empty())
            .cloned();
        if access_token.is_some() && (auth_email.is_some() || auth_password.is_some()) {
            return Err(ToolError::config(
                "COLDRAWDB_ACCESS_TOKEN 与动态登录凭据互斥，请只配置一种模式",
            ));
        }
        if auth_email.is_some() != auth_password.is_some() {
            return Err(ToolError::config(
                "COLDRAWDB_AUTH_EMAIL 与 COLDRAWDB_AUTH_PASSWORD 必须成对配置",
            ));
        }

        Ok(Self {
            base_url,
            access_token,
            auth_email,
            auth_password,
            timeout: Duration::from_secs(timeout_secs),
            payload_soft_limit_bytes,
        })
    }

    pub fn dynamic_auth_enabled(&self) -> bool {
        self.auth_email.is_some() && self.auth_password.is_some()
    }

    pub fn endpoint(&self, path: &str) -> Result<Url, ToolError> {
        let base = self.base_url.as_str().trim_end_matches('/');
        Url::parse(&format!("{base}{path}")).map_err(|_| ToolError::config("无法构造上游地址"))
    }
}
