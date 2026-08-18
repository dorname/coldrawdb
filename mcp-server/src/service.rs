use serde_json::{json, Value};

use crate::api::ApiClient;
use crate::error::ToolError;
use crate::export::export_diagram;

#[derive(Clone)]
pub struct McpService {
    api: ApiClient,
}

impl McpService {
    pub fn new(api: ApiClient) -> Self {
        Self { api }
    }

    pub fn tools() -> Result<Vec<Value>, ToolError> {
        let contract: serde_yaml::Value =
            serde_yaml::from_str(include_str!("../../logos/resources/api/mcp-tools.yaml"))
                .map_err(|_| ToolError::new("INTERNAL_ERROR", "无法读取 MCP 工具契约", false))?;
        let json = serde_json::to_value(contract)
            .map_err(|_| ToolError::new("INTERNAL_ERROR", "无法转换 MCP 工具契约", false))?;
        let tools = json
            .get("tools")
            .and_then(Value::as_array)
            .ok_or_else(|| ToolError::new("INTERNAL_ERROR", "MCP 工具契约缺少 tools", false))?;
        let schemas = json
            .get("schemas")
            .and_then(Value::as_object)
            .ok_or_else(|| ToolError::new("INTERNAL_ERROR", "MCP 工具契约缺少 schemas", false))?;
        Ok(tools
            .iter()
            .map(|tool| {
                let mut exposed = json!({
                    "name": tool.get("name"),
                    "title": tool.get("title"),
                    "description": tool.get("description"),
                    "inputSchema": tool.get("inputSchema"),
                    "outputSchema": tool.get("outputSchema"),
                    "annotations": tool.get("annotations")
                });
                resolve_refs(&mut exposed, schemas);
                exposed
            })
            .collect())
    }

    pub async fn call(&self, name: &str, arguments: Value) -> Result<Value, ToolError> {
        match name {
            "list_diagrams" => {
                let limit = arguments
                    .get("limit")
                    .and_then(Value::as_u64)
                    .unwrap_or(100) as usize;
                if !(1..=100).contains(&limit) {
                    return Err(ToolError::validation("limit 必须在 1～100 之间"));
                }
                self.api
                    .list(arguments.get("query").and_then(Value::as_str), limit)
                    .await
            }
            "get_diagram" => self.api.get(required_str(&arguments, "id")?).await,
            "export_schema" => {
                let result = self.api.get(required_str(&arguments, "id")?).await?;
                export_diagram(
                    result.get("diagram").unwrap_or(&Value::Null),
                    required_str(&arguments, "format")?,
                )
            }
            "delete_diagram" if arguments.get("confirm") != Some(&Value::Bool(true)) => {
                Err(ToolError::validation("delete_diagram 需要 confirm=true"))
            }
            "create_diagram" => {
                let name = required_str(&arguments, "name")?;
                if name.chars().count() > 64 {
                    return Err(ToolError::validation("name 最长 64 个字符"));
                }
                self.api
                    .create(name, arguments.get("database").and_then(Value::as_str))
                    .await
            }
            "update_diagram" => {
                let id = required_str(&arguments, "id")?;
                let expected_revision = arguments
                    .get("expected_revision")
                    .and_then(Value::as_i64)
                    .filter(|revision| *revision >= 0)
                    .ok_or_else(|| ToolError::validation("expected_revision 必须是非负整数"))?;
                let diagram = arguments
                    .get("diagram")
                    .filter(|value| value.is_object())
                    .cloned()
                    .ok_or_else(|| ToolError::validation("diagram 必须是 object"))?;
                self.api.update(id, expected_revision, diagram).await
            }
            "delete_diagram" => self.api.delete(required_str(&arguments, "id")?).await,
            "import_schema" => {
                if arguments.get("format").and_then(Value::as_str) != Some("drawdb_json") {
                    return Err(ToolError::validation("MVP 仅支持 drawdb_json 导入"));
                }
                let payload = arguments
                    .get("payload")
                    .filter(|value| value.is_object())
                    .cloned()
                    .ok_or_else(|| ToolError::validation("payload 必须是 object"))?;
                self.api
                    .import(arguments.get("source").and_then(Value::as_str), payload)
                    .await
            }
            _ => Err(ToolError::validation("未知工具")),
        }
    }
}

fn resolve_refs(value: &mut Value, schemas: &serde_json::Map<String, Value>) {
    match value {
        Value::Object(object) => {
            if let Some(reference) = object.get("$ref").and_then(Value::as_str) {
                if let Some(name) = reference.strip_prefix("#/schemas/") {
                    if let Some(schema) = schemas.get(name) {
                        *value = schema.clone();
                        resolve_refs(value, schemas);
                        return;
                    }
                }
            }
            for child in object.values_mut() {
                resolve_refs(child, schemas);
            }
        }
        Value::Array(items) => {
            for item in items {
                resolve_refs(item, schemas);
            }
        }
        _ => {}
    }
}

pub fn required_str<'a>(arguments: &'a Value, field: &str) -> Result<&'a str, ToolError> {
    arguments
        .get(field)
        .and_then(Value::as_str)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| ToolError::validation(format!("缺少 {field}")))
}
