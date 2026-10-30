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

fn fields(table: &Value) -> &[Value] {
    table
        .get("fields")
        .and_then(Value::as_array)
        .map(Vec::as_slice)
        .unwrap_or(&[])
}

fn references(diagram: &Value) -> &[Value] {
    diagram
        .get("references")
        .and_then(Value::as_array)
        .map(Vec::as_slice)
        .unwrap_or(&[])
}

fn bool_value(value: &Value, key: &str) -> bool {
    value.get(key).and_then(Value::as_bool).unwrap_or(false)
}

fn text_value<'a>(value: &'a Value, key: &str) -> Option<&'a str> {
    value
        .get(key)
        .and_then(Value::as_str)
        .filter(|text| !text.trim().is_empty())
}

fn field_type(field: &Value) -> Option<&str> {
    field
        .get("type_")
        .or_else(|| field.get("type"))
        .and_then(Value::as_str)
}

fn resolve_reference<'a>(
    tables: &'a [Value],
    reference: &Value,
) -> Option<(&'a Value, &'a Value, &'a Value, &'a Value)> {
    let start_table_id = reference.get("start_table_id")?.as_str()?;
    let start_field_id = reference.get("start_field_id")?.as_str()?;
    let end_table_id = reference.get("end_table_id")?.as_str()?;
    let end_field_id = reference.get("end_field_id")?.as_str()?;
    let start_table = tables
        .iter()
        .find(|table| table.get("id").and_then(Value::as_str) == Some(start_table_id))?;
    let end_table = tables
        .iter()
        .find(|table| table.get("id").and_then(Value::as_str) == Some(end_table_id))?;
    let start_field = fields(start_table)
        .iter()
        .find(|field| field.get("id").and_then(Value::as_str) == Some(start_field_id))?;
    let end_field = fields(end_table)
        .iter()
        .find(|field| field.get("id").and_then(Value::as_str) == Some(end_field_id))?;
    Some((start_table, start_field, end_table, end_field))
}

fn escape_dbml_string(value: &str) -> String {
    value
        .replace('\\', "\\\\")
        .replace('\'', "\\'")
        .replace('\r', "\\r")
        .replace('\n', "\\n")
}

fn dbml_default(value: &str) -> String {
    let value = value.trim();
    if value.parse::<f64>().is_ok()
        || matches!(
            value.to_ascii_lowercase().as_str(),
            "true" | "false" | "null"
        )
    {
        value.to_string()
    } else {
        format!("`{}`", value.replace('\\', "\\\\").replace('`', "\\`"))
    }
}

fn to_dbml(diagram: &Value) -> String {
    let tables = tables(diagram);
    let mut output = String::new();
    for table in tables {
        let name = table.get("name").and_then(Value::as_str).unwrap_or("table");
        let fields = fields(table);
        let primary_fields: Vec<&Value> = fields
            .iter()
            .filter(|field| bool_value(field, "primary"))
            .collect();
        let composite_primary = primary_fields.len() > 1;
        output.push_str(&format!("Table {name} {{\n"));
        for field in fields {
            let field_name = field.get("name").and_then(Value::as_str).unwrap_or("field");
            let mut attributes = Vec::new();
            if bool_value(field, "primary") && !composite_primary {
                attributes.push("pk".to_string());
            }
            if bool_value(field, "not_null") {
                attributes.push("not null".to_string());
            }
            if bool_value(field, "unique") {
                attributes.push("unique".to_string());
            }
            if let Some(default) = text_value(field, "default") {
                attributes.push(format!("default: {}", dbml_default(default)));
            }
            if let Some(comment) = text_value(field, "comment") {
                attributes.push(format!("note: '{}'", escape_dbml_string(comment)));
            }
            output.push_str(&format!(
                "  {field_name} {}",
                field_type(field).unwrap_or("text")
            ));
            if !attributes.is_empty() {
                output.push_str(&format!(" [{}]", attributes.join(", ")));
            }
            output.push('\n');
        }
        if composite_primary {
            let columns = primary_fields
                .iter()
                .map(|field| field.get("name").and_then(Value::as_str).unwrap_or("field"))
                .collect::<Vec<_>>()
                .join(", ");
            output.push_str(&format!("\n  indexes {{\n    ({columns}) [pk]\n  }}\n"));
        }
        if let Some(comment) = text_value(table, "comment") {
            output.push_str(&format!("\n  Note: '{}'\n", escape_dbml_string(comment)));
        }
        output.push_str("}\n\n");
    }
    for reference in references(diagram) {
        let Some((start_table, start_field, end_table, end_field)) =
            resolve_reference(tables, reference)
        else {
            continue;
        };
        let start_table_name = start_table
            .get("name")
            .and_then(Value::as_str)
            .unwrap_or("table");
        let start_field_name = start_field
            .get("name")
            .and_then(Value::as_str)
            .unwrap_or("field");
        let end_table_name = end_table
            .get("name")
            .and_then(Value::as_str)
            .unwrap_or("table");
        let end_field_name = end_field
            .get("name")
            .and_then(Value::as_str)
            .unwrap_or("field");
        output.push_str(&format!(
            "Ref: {start_table_name}.{start_field_name} > {end_table_name}.{end_field_name}"
        ));
        let mut actions = Vec::new();
        if let Some(action) = text_value(reference, "on_delete") {
            actions.push(format!(
                "delete: {}",
                action.trim().to_ascii_lowercase().replace('_', " ")
            ));
        }
        if let Some(action) = text_value(reference, "on_update") {
            actions.push(format!(
                "update: {}",
                action.trim().to_ascii_lowercase().replace('_', " ")
            ));
        }
        if !actions.is_empty() {
            output.push_str(&format!(" [{}]", actions.join(", ")));
        }
        output.push('\n');
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

fn escape_sql_string(value: &str) -> String {
    value.replace('\'', "''")
}

fn to_sql(diagram: &Value, format: &str) -> String {
    let tables = tables(diagram);
    let mut output = format!("-- coldrawdb export: {format}\n");
    for table in tables {
        let name = table.get("name").and_then(Value::as_str).unwrap_or("table");
        let fields = fields(table);
        let primary_fields: Vec<&Value> = fields
            .iter()
            .filter(|field| bool_value(field, "primary"))
            .collect();
        let composite_primary = primary_fields.len() > 1;
        let mut definitions: Vec<String> = fields
            .iter()
            .map(|field| {
                let name = field.get("name").and_then(Value::as_str).unwrap_or("field");
                let mut definition = format!(
                    "  {} {}",
                    quote(name, format),
                    field_type(field).unwrap_or("TEXT")
                );
                if bool_value(field, "primary") && !composite_primary {
                    definition.push_str(" PRIMARY KEY");
                }
                if bool_value(field, "not_null") {
                    definition.push_str(" NOT NULL");
                }
                if bool_value(field, "unique") {
                    definition.push_str(" UNIQUE");
                }
                if let Some(default) = text_value(field, "default") {
                    definition.push_str(&format!(" DEFAULT {default}"));
                }
                if matches!(format, "mysql" | "mariadb") {
                    if let Some(comment) = text_value(field, "comment") {
                        definition.push_str(&format!(" COMMENT '{}'", escape_sql_string(comment)));
                    }
                }
                definition
            })
            .collect();
        if composite_primary {
            let columns = primary_fields
                .iter()
                .map(|field| {
                    quote(
                        field.get("name").and_then(Value::as_str).unwrap_or("field"),
                        format,
                    )
                })
                .collect::<Vec<_>>()
                .join(", ");
            definitions.push(format!("  PRIMARY KEY ({columns})"));
        }
        for reference in references(diagram) {
            let Some((start_table, start_field, end_table, end_field)) =
                resolve_reference(tables, reference)
            else {
                continue;
            };
            if start_table.get("id") != table.get("id") {
                continue;
            }
            let start_field_name = start_field
                .get("name")
                .and_then(Value::as_str)
                .unwrap_or("field");
            let end_table_name = end_table
                .get("name")
                .and_then(Value::as_str)
                .unwrap_or("table");
            let end_field_name = end_field
                .get("name")
                .and_then(Value::as_str)
                .unwrap_or("field");
            let mut definition = format!(
                "  FOREIGN KEY ({}) REFERENCES {} ({})",
                quote(start_field_name, format),
                quote(end_table_name, format),
                quote(end_field_name, format)
            );
            if let Some(action) = text_value(reference, "on_delete") {
                definition.push_str(&format!(
                    " ON DELETE {}",
                    action.trim().to_ascii_uppercase().replace('_', " ")
                ));
            }
            if let Some(action) = text_value(reference, "on_update") {
                definition.push_str(&format!(
                    " ON UPDATE {}",
                    action.trim().to_ascii_uppercase().replace('_', " ")
                ));
            }
            definitions.push(definition);
        }
        let table_comment = text_value(table, "comment");
        let table_suffix = if matches!(format, "mysql" | "mariadb") {
            table_comment
                .map(|comment| format!(" COMMENT='{}'", escape_sql_string(comment)))
                .unwrap_or_default()
        } else {
            String::new()
        };
        output.push_str(&format!(
            "CREATE TABLE {} (\n{}\n){};\n",
            quote(name, format),
            definitions.join(",\n"),
            table_suffix
        ));
        if matches!(format, "postgresql" | "oracle" | "generic") {
            if let Some(comment) = table_comment {
                output.push_str(&format!(
                    "COMMENT ON TABLE {} IS '{}';\n",
                    quote(name, format),
                    escape_sql_string(comment)
                ));
            }
            for field in fields {
                if let Some(comment) = text_value(field, "comment") {
                    let field_name = field.get("name").and_then(Value::as_str).unwrap_or("field");
                    output.push_str(&format!(
                        "COMMENT ON COLUMN {}.{} IS '{}';\n",
                        quote(name, format),
                        quote(field_name, format),
                        escape_sql_string(comment)
                    ));
                }
            }
        }
        output.push('\n');
    }
    output
}
