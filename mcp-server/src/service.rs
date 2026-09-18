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
            // mcp-canvas-tools（2026-09-18）：画布编辑工具
            "update_table" => self.update_table(arguments).await,
            "update_field" => self.update_field(arguments).await,
            "update_reference" => self.update_reference(arguments).await,
            "layout_diagram" => self.layout_diagram(arguments).await,
            _ => Err(ToolError::validation("未知工具")),
        }
    }

    // ── mcp-canvas-tools（2026-09-18）：画布编辑工具实现 ──────────────────

    async fn update_table(&self, arguments: Value) -> Result<Value, ToolError> {
        let id = required_str(&arguments, "id")?;
        let table_id = required_str(&arguments, "table_id")?;

        if let Some(name) = arguments.get("name").and_then(Value::as_str) {
            if name.is_empty() || name.chars().count() > 64 {
                return Err(ToolError::validation("name 必须在 1～64 字符之间"));
            }
        }

        let result = self.api.get(id).await?;
        let mut diagram = result
            .get("diagram")
            .cloned()
            .ok_or_else(|| ToolError::new("UPSTREAM_ERROR", "响应缺少 diagram", false))?;
        let revision = diagram
            .get("revision")
            .and_then(Value::as_i64)
            .ok_or_else(|| ToolError::new("UPSTREAM_ERROR", "响应缺少 revision", false))?;

        let tables = diagram
            .get_mut("tables")
            .and_then(Value::as_array_mut)
            .ok_or_else(|| ToolError::new("UPSTREAM_ERROR", "diagram 缺少 tables 数组", false))?;
        let table = tables
            .iter_mut()
            .find(|t| t.get("id").and_then(Value::as_str) == Some(table_id))
            .ok_or_else(|| ToolError::validation(format!("table_id {table_id} 不存在")))?;

        if let Some(name) = arguments.get("name").and_then(Value::as_str) {
            table["name"] = json!(name);
        }
        if let Some(comment) = arguments.get("comment").and_then(Value::as_str) {
            table["comment"] = json!(comment);
        }
        if let Some(x) = arguments.get("x").and_then(Value::as_f64) {
            table["x"] = json!(x);
        }
        if let Some(y) = arguments.get("y").and_then(Value::as_f64) {
            table["y"] = json!(y);
        }
        if let Some(color) = arguments.get("color").and_then(Value::as_str) {
            table["color"] = json!(color);
        }
        if let Some(locked) = arguments.get("locked").and_then(Value::as_bool) {
            table["locked"] = json!(locked);
        }

        self.api.update(id, revision, diagram).await
    }

    async fn update_field(&self, arguments: Value) -> Result<Value, ToolError> {
        let id = required_str(&arguments, "id")?;
        let table_id = required_str(&arguments, "table_id")?;
        let field_id = required_str(&arguments, "field_id")?;

        if let Some(name) = arguments.get("name").and_then(Value::as_str) {
            if name.is_empty() || name.chars().count() > 64 {
                return Err(ToolError::validation("name 必须在 1～64 字符之间"));
            }
        }

        let result = self.api.get(id).await?;
        let mut diagram = result
            .get("diagram")
            .cloned()
            .ok_or_else(|| ToolError::new("UPSTREAM_ERROR", "响应缺少 diagram", false))?;
        let revision = diagram
            .get("revision")
            .and_then(Value::as_i64)
            .ok_or_else(|| ToolError::new("UPSTREAM_ERROR", "响应缺少 revision", false))?;

        let tables = diagram
            .get_mut("tables")
            .and_then(Value::as_array_mut)
            .ok_or_else(|| ToolError::new("UPSTREAM_ERROR", "diagram 缺少 tables 数组", false))?;
        let table = tables
            .iter_mut()
            .find(|t| t.get("id").and_then(Value::as_str) == Some(table_id))
            .ok_or_else(|| ToolError::validation(format!("table_id {table_id} 不存在")))?;
        let fields = table
            .get_mut("fields")
            .and_then(Value::as_array_mut)
            .ok_or_else(|| ToolError::new("UPSTREAM_ERROR", "table 缺少 fields 数组", false))?;
        let field = fields
            .iter_mut()
            .find(|f| f.get("id").and_then(Value::as_str) == Some(field_id))
            .ok_or_else(|| ToolError::validation(format!("field_id {field_id} 不存在")))?;

        if let Some(name) = arguments.get("name").and_then(Value::as_str) {
            field["name"] = json!(name);
        }
        if let Some(comment) = arguments.get("comment").and_then(Value::as_str) {
            field["comment"] = json!(comment);
        }
        if let Some(type_) = arguments.get("type").and_then(Value::as_str) {
            field["type"] = json!(type_);
        }
        if let Some(not_null) = arguments.get("not_null").and_then(Value::as_bool) {
            field["not_null"] = json!(not_null);
        }
        if let Some(primary) = arguments.get("primary").and_then(Value::as_bool) {
            field["primary"] = json!(primary);
        }
        if let Some(default) = arguments.get("default").and_then(Value::as_str) {
            field["default"] = json!(default);
        }

        self.api.update(id, revision, diagram).await
    }

    async fn update_reference(&self, arguments: Value) -> Result<Value, ToolError> {
        let id = required_str(&arguments, "id")?;
        let action = required_str(&arguments, "action")?;

        // 先校验 action 及必需参数，再调 API
        match action {
            "create" => {
                required_str(&arguments, "start_table_id")?;
                required_str(&arguments, "start_field_id")?;
                required_str(&arguments, "end_table_id")?;
                required_str(&arguments, "end_field_id")?;
            }
            "update" | "delete" => {
                required_str(&arguments, "ref_id")?;
            }
            _ => return Err(ToolError::validation("action 必须是 create/update/delete")),
        }

        let result = self.api.get(id).await?;
        let mut diagram = result
            .get("diagram")
            .cloned()
            .ok_or_else(|| ToolError::new("UPSTREAM_ERROR", "响应缺少 diagram", false))?;
        let revision = diagram
            .get("revision")
            .and_then(Value::as_i64)
            .ok_or_else(|| ToolError::new("UPSTREAM_ERROR", "响应缺少 revision", false))?;

        let references = diagram
            .get_mut("references")
            .and_then(Value::as_array_mut)
            .ok_or_else(|| ToolError::new("UPSTREAM_ERROR", "diagram 缺少 references 数组", false))?;

        match action {
            "create" => {
                let start_table_id = required_str(&arguments, "start_table_id")?;
                let start_field_id = required_str(&arguments, "start_field_id")?;
                let end_table_id = required_str(&arguments, "end_table_id")?;
                let end_field_id = required_str(&arguments, "end_field_id")?;

                let mut new_ref = json!({
                    "id": format!("ref-{}", uuid_simple()),
                    "start_table_id": start_table_id,
                    "start_field_id": start_field_id,
                    "end_table_id": end_table_id,
                    "end_field_id": end_field_id,
                    "cardinality": arguments.get("cardinality").and_then(Value::as_str).unwrap_or("many_to_one"),
                    "on_delete": arguments.get("on_delete").and_then(Value::as_str).unwrap_or("no_action"),
                    "on_update": arguments.get("on_update").and_then(Value::as_str).unwrap_or("no_action"),
                });
                if let Some(name) = arguments.get("name").and_then(Value::as_str) {
                    new_ref["name"] = json!(name);
                }
                references.push(new_ref);
            }
            "update" => {
                let ref_id = required_str(&arguments, "ref_id")?;
                let existing = references
                    .iter_mut()
                    .find(|r| r.get("id").and_then(Value::as_str) == Some(ref_id))
                    .ok_or_else(|| ToolError::validation(format!("ref_id {ref_id} 不存在")))?;
                if let Some(cardinality) = arguments.get("cardinality").and_then(Value::as_str) {
                    existing["cardinality"] = json!(cardinality);
                }
                if let Some(on_delete) = arguments.get("on_delete").and_then(Value::as_str) {
                    existing["on_delete"] = json!(on_delete);
                }
                if let Some(on_update) = arguments.get("on_update").and_then(Value::as_str) {
                    existing["on_update"] = json!(on_update);
                }
                if let Some(name) = arguments.get("name").and_then(Value::as_str) {
                    existing["name"] = json!(name);
                }
            }
            "delete" => {
                let ref_id = required_str(&arguments, "ref_id")?;
                let before = references.len();
                references.retain(|r| r.get("id").and_then(Value::as_str) != Some(ref_id));
                if references.len() == before {
                    return Err(ToolError::validation(format!("ref_id {ref_id} 不存在")));
                }
            }
            _ => return Err(ToolError::validation("action 必须是 create/update/delete")),
        }

        self.api.update(id, revision, diagram).await
    }

    async fn layout_diagram(&self, arguments: Value) -> Result<Value, ToolError> {
        let id = required_str(&arguments, "id")?;

        let result = self.api.get(id).await?;
        let mut diagram = result
            .get("diagram")
            .cloned()
            .ok_or_else(|| ToolError::new("UPSTREAM_ERROR", "响应缺少 diagram", false))?;
        let revision = diagram
            .get("revision")
            .and_then(Value::as_i64)
            .ok_or_else(|| ToolError::new("UPSTREAM_ERROR", "响应缺少 revision", false))?;

        let tables = diagram
            .get("tables")
            .and_then(Value::as_array)
            .cloned()
            .unwrap_or_default();
        let references = diagram
            .get("references")
            .and_then(Value::as_array)
            .cloned()
            .unwrap_or_default();

        let params = crate::layout::LayoutParams {
            iterations: arguments
                .get("iterations")
                .and_then(Value::as_u64)
                .map(|v| v as usize)
                .unwrap_or(100),
            spacing: arguments
                .get("spacing")
                .and_then(Value::as_f64)
                .unwrap_or(180.0),
        };

        let repositioned = crate::layout::force_directed_layout(&tables, &references, &params);
        let count = repositioned.len();
        diagram["tables"] = json!(repositioned);

        let mut resp = self.api.update(id, revision, diagram).await?;
        resp["tables_repositioned"] = json!(count);
        Ok(resp)
    }
}

fn uuid_simple() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    format!("{:x}", nanos)
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
