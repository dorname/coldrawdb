//! 房间图全量 JSON → collab op。语义对齐 frontend-rs `diff_snapshots`。
//! 落地目标：`mcp-server/src/collab_diff.rs`（guard 释放后复制，勿在 rerelease 提案内提交）。

use std::collections::BTreeMap;

use serde_json::{json, Value};

#[derive(Default)]
struct Snap {
    tables: BTreeMap<String, Value>,
    fields: BTreeMap<String, (String, Value)>,
    field_order: BTreeMap<String, Vec<String>>,
    references: BTreeMap<String, Value>,
    areas: BTreeMap<String, Value>,
    notes: BTreeMap<String, Value>,
    dictionaries: BTreeMap<String, Value>,
    full_tables: BTreeMap<String, Value>,
}

fn snapshot(diagram: &Value) -> Snap {
    let mut snap = Snap::default();
    let empty = Vec::new();
    let tables = diagram.get("tables").and_then(Value::as_array).unwrap_or(&empty);
    for table in tables {
        let Some(id) = table.get("id").and_then(Value::as_str) else {
            continue;
        };
        snap.full_tables.insert(id.to_string(), table.clone());
        let mut scalars = table.clone();
        if let Some(obj) = scalars.as_object_mut() {
            obj.remove("fields");
        }
        snap.tables.insert(id.to_string(), scalars);
        let fields = table.get("fields").and_then(Value::as_array).unwrap_or(&empty);
        let mut order = Vec::new();
        for field in fields {
            let Some(fid) = field.get("id").and_then(Value::as_str) else {
                continue;
            };
            order.push(fid.to_string());
            snap.fields
                .insert(fid.to_string(), (id.to_string(), field.clone()));
        }
        snap.field_order.insert(id.to_string(), order);
    }
    for (key, dest) in [
        ("references", &mut snap.references),
        ("areas", &mut snap.areas),
        ("notes", &mut snap.notes),
        ("dictionaries", &mut snap.dictionaries),
    ] {
        let items = diagram.get(key).and_then(Value::as_array).unwrap_or(&empty);
        for item in items {
            if let Some(id) = item.get("id").and_then(Value::as_str) {
                dest.insert(id.to_string(), item.clone());
            }
        }
    }
    snap
}

fn op_create(kind: &str, target_id: &str, parent_id: Option<&str>, changes: Value) -> Value {
    let mut op = json!({ "type": format!("{kind}.create"), "targetId": target_id, "changes": changes });
    if let Some(parent) = parent_id {
        op["parentId"] = json!(parent);
    }
    op
}

fn op_update(kind: &str, target_id: &str, parent_id: Option<&str>, changes: Value) -> Value {
    let mut op = json!({ "type": format!("{kind}.update"), "targetId": target_id, "changes": changes });
    if let Some(parent) = parent_id {
        op["parentId"] = json!(parent);
    }
    op
}

fn op_delete(kind: &str, target_id: &str, parent_id: Option<&str>) -> Value {
    let mut op = json!({ "type": format!("{kind}.delete"), "targetId": target_id });
    if let Some(parent) = parent_id {
        op["parentId"] = json!(parent);
    }
    op
}

/// `base` 为当前物化文档，`desired` 为写工具准备提交的全量 diagram。
pub fn diff_diagrams(base: &Value, desired: &Value) -> Vec<Value> {
    let prev = snapshot(base);
    let next = snapshot(desired);
    let mut ops = Vec::new();

    for (id, next_v) in &next.tables {
        match prev.tables.get(id) {
            None => {
                let full = next.full_tables.get(id).cloned().unwrap_or_else(|| next_v.clone());
                ops.push(op_create("table", id, None, full));
            }
            Some(prev_v) if prev_v != next_v => ops.push(op_update("table", id, None, next_v.clone())),
            _ => {}
        }
    }
    let mut deleted_tables = Vec::new();
    for id in prev.tables.keys() {
        if !next.tables.contains_key(id) {
            ops.push(op_delete("table", id, None));
            deleted_tables.push(id.clone());
        }
    }
    let created_table = |tid: &str| !prev.tables.contains_key(tid) && next.tables.contains_key(tid);
    let deleted_table = |tid: &str| deleted_tables.iter().any(|item| item == tid);

    for (fid, (tid, next_v)) in &next.fields {
        if created_table(tid) {
            continue;
        }
        match prev.fields.get(fid) {
            None => ops.push(op_create("field", fid, Some(tid), next_v.clone())),
            Some((_, prev_v)) if prev_v != next_v => {
                ops.push(op_update("field", fid, Some(tid), next_v.clone()))
            }
            _ => {}
        }
    }
    for (fid, (tid, _)) in &prev.fields {
        if next.fields.contains_key(fid) || created_table(tid) || deleted_table(tid) {
            continue;
        }
        ops.push(op_delete("field", fid, Some(tid)));
    }

    for (tid, next_order) in &next.field_order {
        if created_table(tid) || deleted_table(tid) {
            continue;
        }
        let Some(prev_order) = prev.field_order.get(tid) else {
            continue;
        };
        if prev_order != next_order {
            ops.push(json!({
                "type": "fields.reorder",
                "targetId": tid,
                "changes": { "fieldIds": next_order },
            }));
        }
    }

    for (kind, prev_map, next_map) in [
        ("reference", &prev.references, &next.references),
        ("area", &prev.areas, &next.areas),
        ("note", &prev.notes, &next.notes),
        ("dict", &prev.dictionaries, &next.dictionaries),
    ] {
        for (id, next_v) in next_map {
            match prev_map.get(id) {
                None => ops.push(op_create(kind, id, None, next_v.clone())),
                Some(prev_v) if prev_v != next_v => ops.push(op_update(kind, id, None, next_v.clone())),
                _ => {}
            }
        }
        for id in prev_map.keys() {
            if !next_map.contains_key(id) {
                ops.push(op_delete(kind, id, None));
            }
        }
    }
    ops
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn table_rename_is_update_not_rewrite() {
        let base = json!({"tables":[{"id":"t1","name":"users","x":1,"fields":[{"id":"f1","name":"id"}]}]});
        let desired = json!({"tables":[{"id":"t1","name":"accounts","x":1,"fields":[{"id":"f1","name":"id"}]}]});
        let ops = diff_diagrams(&base, &desired);
        assert_eq!(ops.len(), 1);
        assert_eq!(ops[0]["type"], "table.update");
        assert_eq!(ops[0]["targetId"], "t1");
        assert!(ops[0]["changes"].get("fields").is_none());
        assert_eq!(ops[0]["changes"]["name"], "accounts");
    }

    #[test]
    fn new_table_carries_fields_without_field_ops() {
        let base = json!({"tables":[]});
        let desired = json!({"tables":[{"id":"t1","name":"users","fields":[{"id":"f1","name":"id"}]}]});
        let ops = diff_diagrams(&base, &desired);
        assert_eq!(ops.len(), 1);
        assert_eq!(ops[0]["type"], "table.create");
        assert_eq!(ops[0]["changes"]["fields"][0]["id"], "f1");
    }

    #[test]
    fn field_update_and_reorder() {
        let base = json!({"tables":[{"id":"t1","name":"users","fields":[{"id":"f1","name":"id"},{"id":"f2","name":"email"}]}]});
        let desired = json!({"tables":[{"id":"t1","name":"users","fields":[{"id":"f2","name":"mail"},{"id":"f1","name":"id"}]}]});
        let ops = diff_diagrams(&base, &desired);
        assert_eq!(ops[0]["type"], "field.update");
        assert_eq!(ops[0]["parentId"], "t1");
        assert_eq!(ops[1]["type"], "fields.reorder");
        assert_eq!(ops[1]["changes"]["fieldIds"], json!(["f2","f1"]));
    }
}
