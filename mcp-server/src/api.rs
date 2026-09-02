use reqwest::{Client, Method};
use serde_json::{json, Value};

use crate::config::Config;
use crate::error::ToolError;

#[derive(Clone)]
pub struct ApiClient {
    config: Config,
    client: Client,
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
        Ok(Self { config, client })
    }

    async fn request(
        &self,
        method: Method,
        path: &str,
        body: Option<Value>,
    ) -> Result<Value, ToolError> {
        let mut request = self.client.request(method, self.config.endpoint(path)?);
        if let Some(token) = &self.config.access_token {
            request = request.bearer_auth(token);
        }
        if let Some(body) = body {
            request = request.json(&body);
        }
        let response = request
            .send()
            .await
            .map_err(|error| ToolError::transport(&error))?;
        let status = response.status();
        let value = response
            .json::<Value>()
            .await
            .unwrap_or_else(|_| json!({"message":"上游返回非 JSON 响应"}));
        if !status.is_success() {
            return Err(ToolError::upstream(status, &value));
        }
        Ok(value)
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
                Some(json!({"expected_revision":expected_revision,"diagram":diagram})),
            )
            .await?;
        response
            .get("data")
            .cloned()
            .ok_or_else(|| ToolError::new("UPSTREAM_ERROR", "上游响应缺少 data", false))
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
