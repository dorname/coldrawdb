use std::collections::HashMap;
use std::time::Duration;

use url::Url;

use crate::error::ToolError;

#[derive(Clone)]
pub struct Config {
    pub base_url: Url,
    pub access_token: Option<String>,
    pub timeout: Duration,
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
            .field("timeout", &self.timeout)
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

        Ok(Self {
            base_url,
            access_token: values
                .get("COLDRAWDB_ACCESS_TOKEN")
                .filter(|value| !value.is_empty())
                .cloned(),
            timeout: Duration::from_secs(timeout_secs),
        })
    }

    pub fn endpoint(&self, path: &str) -> Result<Url, ToolError> {
        let base = self.base_url.as_str().trim_end_matches('/');
        Url::parse(&format!("{base}{path}")).map_err(|_| ToolError::config("无法构造上游地址"))
    }
}
