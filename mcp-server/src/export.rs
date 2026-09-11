use serde_json::{json, Value};

use crate::error::ToolError;

const SQL_FORMATS: &[&str] = &[
    "mysql",
    "postgresql",
    "sqlite",
    "mariadb",
    "mssql",
    "oracle",
    "generic",
];

pub fn export_diagram(diagram: &Value, format: &str) -> Result<Value, ToolError> {
    let id = diagram
        .get("id")
        .and_then(Value::as_str)
        .ok_or_else(|| ToolError::validation("diagram 缺少 id"))?;
    let revision = diagram.get("revision").and_then(Value::as_i64).unwrap_or(0);
    let (mime_type, content) = match format {
        "json" => (
            "application/json",
            serde_json::to_string_pretty(diagram)
                .map_err(|_| ToolError::new("INTERNAL_ERROR", "JSON 序列化失败", false))?,
        ),
        "dbml" => ("text/plain", to_dbml(diagram)),
        value if SQL_FORMATS.contains(&value) => ("text/sql", to_sql(diagram, value)),
        _ => return Err(ToolError::validation("不支持的导出格式")),
    };
    Ok(
        json!({"diagram_id":id,"revision":revision,"format":format,"mime_type":mime_type,"content":content}),
    )
}

fn tables(diagram: &Value) -> &[Value] {
    diagram
        .get("tables")
        .and_then(Value::as_array)
        .map(Vec::as_slice)
        .unwrap_or(&[])
}

fn to_dbml(diagram: &Value) -> String {
    let mut output = String::new();
    for table in tables(diagram) {
        let name = table.get("name").and_then(Value::as_str).unwrap_or("table");
        output.push_str(&format!("Table {name} {{\n"));
        if let Some(fields) = table.get("fields").and_then(Value::as_array) {
            for field in fields {
                let field_name = field.get("name").and_then(Value::as_str).unwrap_or("field");
                let field_type = field
                    .get("type_")
                    .or_else(|| field.get("type"))
                    .and_then(Value::as_str)
                    .unwrap_or("text");
                let primary = if field
                    .get("primary")
                    .and_then(Value::as_bool)
                    .unwrap_or(false)
                {
                    " [pk]"
                } else {
                    ""
                };
                output.push_str(&format!("  {field_name} {field_type}{primary}\n"));
            }
        }
        output.push_str("}\n\n");
    }
    output
}

fn quote(name: &str, format: &str) -> String {
    match format {
        "mysql" | "mariadb" => format!("`{}`", name.replace('`', "``")),
        "mssql" => format!("[{}]", name.replace(']', "]]")),
        _ => format!("\"{}\"", name.replace('"', "\"\"")),
    }
}

fn to_sql(diagram: &Value, format: &str) -> String {
    let mut output = format!("-- coldrawdb export: {format}\n");
    for table in tables(diagram) {
        let name = table.get("name").and_then(Value::as_str).unwrap_or("table");
        let fields = table
            .get("fields")
            .and_then(Value::as_array)
            .cloned()
            .unwrap_or_default();
        let definitions: Vec<String> = fields
            .iter()
            .map(|field| {
                let name = field.get("name").and_then(Value::as_str).unwrap_or("field");
                let ty = field
                    .get("type_")
                    .or_else(|| field.get("type"))
                    .and_then(Value::as_str)
                    .unwrap_or("TEXT");
                let mut definition = format!("  {} {}", quote(name, format), ty);
                if field
                    .get("primary")
                    .and_then(Value::as_bool)
                    .unwrap_or(false)
                {
                    definition.push_str(" PRIMARY KEY");
                }
                if field
                    .get("not_null")
                    .and_then(Value::as_bool)
                    .unwrap_or(false)
                {
                    definition.push_str(" NOT NULL");
                }
                definition
            })
            .collect();
        output.push_str(&format!(
            "CREATE TABLE {} (\n{}\n);\n\n",
            quote(name, format),
            definitions.join(",\n")
        ));
    }
    output
}
