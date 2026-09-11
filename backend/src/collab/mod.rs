mod hub;
mod ws;

pub use hub::CollabHub;
pub use ws::collab_ws_handler;

use sea_orm::{ConnectionTrait, DatabaseBackend, DatabaseConnection, Statement, TransactionTrait};
use serde_json::Value;
use sha2::{Digest, Sha256};
use uuid::Uuid;

use crate::error::DrawDBError;
use crate::next_id;
use crate::rooms::{get_member_role, RoomsServiceError};

pub const MAX_CATCH_UP: i64 = 500;

#[derive(Debug, Clone)]
pub struct CollabHead {
    pub room_id: String,
    pub diagram_id: String,
    pub server_rev: i64,
    pub snapshot_hash: Option<String>,
    pub checkpoint_revision: Option<i64>,
    pub last_checkpoint_at: Option<String>,
}

#[derive(Debug, Clone)]
pub struct CollabOpEntry {
    pub server_rev: i64,
    pub operation_id: String,
    pub op_type: String,
    pub payload: serde_json::Value,
    pub user_id: String,
    pub created_at: String,
}

#[derive(Debug, Clone)]
pub struct AppendOpResult {
    pub server_rev: i64,
    pub operation_id: String,
    pub op_type: String,
    pub payload: serde_json::Value,
}

pub async fn ensure_collab_head(db: &DatabaseConnection, room_id: &str) -> Result<(), CollabServiceError> {
    db.execute(Statement::from_sql_and_values(
        DatabaseBackend::Sqlite,
        "INSERT OR IGNORE INTO room_collab_head(room_id, server_rev, updated_at) VALUES(?, 0, strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))",
        vec![room_id.into()],
    ))
    .await
    .map_err(|e| CollabServiceError::Db(DrawDBError::DatabaseError(e)))?;
    Ok(())
}

async fn load_room_diagram(
    db: &DatabaseConnection,
    room_id: &str,
) -> Result<Option<String>, CollabServiceError> {
    let row = db
        .query_one(Statement::from_sql_and_values(
            DatabaseBackend::Sqlite,
            "SELECT diagram_id FROM room WHERE id = ? AND archived_at IS NULL LIMIT 1",
            vec![room_id.into()],
        ))
        .await
        .map_err(|e| CollabServiceError::Db(DrawDBError::DatabaseError(e)))?;
    Ok(row.and_then(|r| r.try_get("", "diagram_id").ok()))
}

pub async fn get_collab_head(
    db: &DatabaseConnection,
    room_id: &str,
    user_id: &str,
) -> Result<CollabHead, CollabServiceError> {
    if load_room_diagram(db, room_id).await?.is_none() {
        return Err(CollabServiceError::RoomNotFound);
    }
    if get_member_role(db, room_id, user_id).await?.is_none() {
        return Err(CollabServiceError::NotAMember);
    }
    ensure_collab_head(db, room_id).await?;
    let row = db
        .query_one(Statement::from_sql_and_values(
            DatabaseBackend::Sqlite,
            "SELECT h.room_id, h.server_rev, h.snapshot_hash, h.checkpoint_revision, h.last_checkpoint_at, r.diagram_id FROM room_collab_head h INNER JOIN room r ON r.id = h.room_id WHERE h.room_id = ? LIMIT 1",
            vec![room_id.into()],
        ))
        .await
        .map_err(|e| CollabServiceError::Db(DrawDBError::DatabaseError(e)))?;
    let Some(row) = row else {
        return Err(CollabServiceError::RoomNotFound);
    };
    Ok(CollabHead {
        room_id: row.try_get("", "room_id").unwrap_or_default(),
        diagram_id: row.try_get("", "diagram_id").unwrap_or_default(),
        server_rev: row.try_get("", "server_rev").unwrap_or(0),
        snapshot_hash: row.try_get("", "snapshot_hash").ok(),
        checkpoint_revision: row.try_get("", "checkpoint_revision").ok(),
        last_checkpoint_at: row.try_get("", "last_checkpoint_at").ok(),
    })
}

pub async fn list_collab_ops(
    db: &DatabaseConnection,
    room_id: &str,
    user_id: &str,
    after_rev: i64,
    limit: u64,
) -> Result<(i64, i64, Vec<CollabOpEntry>), CollabServiceError> {
    if after_rev < 0 {
        return Err(CollabServiceError::Validation("afterRev 不能为负".into()));
    }
    let head = get_collab_head(db, room_id, user_id).await?;
    if head.server_rev - after_rev > MAX_CATCH_UP {
        return Err(CollabServiceError::SyncGapTooLarge {
            current_server_rev: head.server_rev,
            max_catch_up: MAX_CATCH_UP,
        });
    }
    let limit = limit.clamp(1, 500);
    let rows = db
        .query_all(Statement::from_sql_and_values(
            DatabaseBackend::Sqlite,
            "SELECT ol.server_rev, ol.operation_id, o.op_type, o.payload, ol.user_id, ol.created_at FROM operation_log ol INNER JOIN operation o ON o.id = ol.operation_id WHERE ol.room_id = ? AND ol.server_rev > ? ORDER BY ol.server_rev ASC LIMIT ?",
            vec![room_id.into(), after_rev.into(), (limit as i64).into()],
        ))
        .await
        .map_err(|e| CollabServiceError::Db(DrawDBError::DatabaseError(e)))?;

    let items: Vec<CollabOpEntry> = rows
        .into_iter()
        .filter_map(|r| {
            let payload_str: String = r.try_get("", "payload").ok()?;
            let payload: serde_json::Value = serde_json::from_str(&payload_str).ok()?;
            Some(CollabOpEntry {
                server_rev: r.try_get("", "server_rev").unwrap_or(0),
                operation_id: r.try_get("", "operation_id").unwrap_or_default(),
                op_type: r.try_get("", "op_type").unwrap_or_default(),
                payload,
                user_id: r.try_get("", "user_id").unwrap_or_default(),
                created_at: r.try_get("", "created_at").unwrap_or_default(),
            })
        })
        .collect();
    let to_rev = items.last().map(|i| i.server_rev).unwrap_or(after_rev);
    Ok((after_rev, to_rev, items))
}

fn payload_hash(payload: &str) -> String {
    hex::encode(Sha256::digest(payload.as_bytes()))
}

pub async fn append_op(
    db: &DatabaseConnection,
    room_id: &str,
    user_id: &str,
    op_type: &str,
    payload: serde_json::Value,
) -> Result<AppendOpResult, CollabServiceError> {
    let role = get_member_role(db, room_id, user_id).await?;
    let Some(role) = role else {
        return Err(CollabServiceError::NotAMember);
    };
    if role == "viewer" {
        return Err(CollabServiceError::ReadOnly);
    }
    let Some(diagram_id) = load_room_diagram(db, room_id).await? else {
        return Err(CollabServiceError::RoomNotFound);
    };
    ensure_collab_head(db, room_id).await?;

    let payload_str = payload.to_string();
    let op_id = Uuid::new_v4().to_string();
    let log_id = next_id();
    let hash = payload_hash(&payload_str);

    let tx = db.begin().await.map_err(|e| CollabServiceError::Db(DrawDBError::DatabaseError(e)))?;

    let head_row = tx
        .query_one(Statement::from_sql_and_values(
            DatabaseBackend::Sqlite,
            "SELECT server_rev FROM room_collab_head WHERE room_id = ? LIMIT 1",
            vec![room_id.into()],
        ))
        .await
        .map_err(|e| CollabServiceError::Db(DrawDBError::DatabaseError(e)))?;
    let cur_rev: i64 = head_row
        .and_then(|r| r.try_get("", "server_rev").ok())
        .unwrap_or(0);
    let new_rev = cur_rev + 1;

    tx.execute(Statement::from_sql_and_values(
        DatabaseBackend::Sqlite,
        "INSERT INTO operation(id, op_type, payload, payload_hash, created_at) VALUES(?, ?, ?, ?, strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))",
        vec![
            op_id.clone().into(),
            op_type.into(),
            payload_str.into(),
            hash.into(),
        ],
    ))
    .await
    .map_err(|e| CollabServiceError::Db(DrawDBError::DatabaseError(e)))?;

    tx.execute(Statement::from_sql_and_values(
        DatabaseBackend::Sqlite,
        "INSERT INTO operation_log(id, room_id, server_rev, operation_id, user_id, created_at) VALUES(?, ?, ?, ?, ?, strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))",
        vec![
            log_id.into(),
            room_id.into(),
            new_rev.into(),
            op_id.clone().into(),
            user_id.into(),
        ],
    ))
    .await
    .map_err(|e| CollabServiceError::Db(DrawDBError::DatabaseError(e)))?;

    tx.execute(Statement::from_sql_and_values(
        DatabaseBackend::Sqlite,
        "UPDATE room_collab_head SET server_rev = ?, updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now') WHERE room_id = ?",
        vec![new_rev.into(), room_id.into()],
    ))
    .await
    .map_err(|e| CollabServiceError::Db(DrawDBError::DatabaseError(e)))?;

    // fix-collab-autosave-race（方案 B 单写者）：op 落库即物化——
    // doc_json 与 op log 同事务提交，恒一致；任何全量加载永远读到 op log 状态。
    let doc_row = tx
        .query_one(Statement::from_sql_and_values(
            DatabaseBackend::Sqlite,
            "SELECT doc_json FROM room_collab_head WHERE room_id = ? LIMIT 1",
            vec![room_id.into()],
        ))
        .await
        .map_err(|e| CollabServiceError::Db(DrawDBError::DatabaseError(e)))?;
    let existing_doc: Option<String> = doc_row
        .and_then(|r| r.try_get::<Option<String>>("", "doc_json").ok().flatten());
    let initialized = existing_doc.is_none();
    let mut doc = match &existing_doc {
        Some(s) => serde_json::from_str::<Value>(s).unwrap_or_else(|_| serde_json::json!({})),
        None => {
            // 首次物化：从关系存储初始化（保真度与现行 GET 一致）。
            let full = crate::diagram_persistence::load_diagram(&tx, &diagram_id)
                .await
                .map_err(CollabServiceError::Db)?
                .ok_or(CollabServiceError::RoomNotFound)?;
            serde_json::to_value(&full).unwrap_or_else(|_| serde_json::json!({}))
        }
    };
    // 注意：apply 必须先求值（`initialized || apply(...)` 会短路跳过首个 op 的 apply）。
    let applied = apply_op_to_doc(&mut doc, &payload);
    if initialized || applied {
        tx.execute(Statement::from_sql_and_values(
            DatabaseBackend::Sqlite,
            "UPDATE room_collab_head SET doc_json = ? WHERE room_id = ?",
            vec![doc.to_string().into(), room_id.into()],
        ))
        .await
        .map_err(|e| CollabServiceError::Db(DrawDBError::DatabaseError(e)))?;
    }

    tx.commit()
        .await
        .map_err(|e| CollabServiceError::Db(DrawDBError::DatabaseError(e)))?;

    Ok(AppendOpResult {
        server_rev: new_rev,
        operation_id: op_id,
        op_type: op_type.to_string(),
        payload,
    })
}

// ---------------------------------------------------------------------------
// fix-collab-autosave-race（方案 B 服务端物化单写者）
// ---------------------------------------------------------------------------

/// diagram 绑定的活跃 room（archived 除外）。
pub async fn room_id_for_diagram(
    db: &DatabaseConnection,
    diagram_id: &str,
) -> Result<Option<String>, DrawDBError> {
    let row = db
        .query_one(Statement::from_sql_and_values(
            DatabaseBackend::Sqlite,
            "SELECT id FROM room WHERE diagram_id = ? AND archived_at IS NULL LIMIT 1",
            vec![diagram_id.into()],
        ))
        .await?;
    Ok(row.and_then(|r| r.try_get("", "id").ok()))
}

/// room 绑定 diagram 的物化文档（doc_json 非 NULL）+ 当前 head server_rev。
pub async fn get_room_document(
    db: &DatabaseConnection,
    diagram_id: &str,
) -> Result<Option<(Value, i64)>, DrawDBError> {
    let row = db
        .query_one(Statement::from_sql_and_values(
            DatabaseBackend::Sqlite,
            "SELECT h.doc_json, h.server_rev FROM room_collab_head h INNER JOIN room r ON r.id = h.room_id WHERE r.diagram_id = ? AND r.archived_at IS NULL AND h.doc_json IS NOT NULL LIMIT 1",
            vec![diagram_id.into()],
        ))
        .await?;
    Ok(row.and_then(|r| {
        let doc_str: Option<String> = r.try_get("", "doc_json").ok();
        let rev: i64 = r.try_get("", "server_rev").unwrap_or(0);
        doc_str
            .and_then(|s| serde_json::from_str(&s).ok())
            .map(|doc| (doc, rev))
    }))
}

fn upsert_by_id(arr: &mut Vec<Value>, id: &str, item: Value) -> bool {
    // upsert 语义：存在即覆盖、不存在追加；均视为已 apply（恒返回 true）。
    if let Some(slot) = arr
        .iter_mut()
        .find(|e| e.get("id").and_then(|v| v.as_str()) == Some(id))
    {
        *slot = item;
    } else {
        arr.push(item);
    }
    true
}

fn delete_by_id(arr: &mut Vec<Value>, id: &str) -> bool {
    let before = arr.len();
    arr.retain(|e| e.get("id").and_then(|v| v.as_str()) != Some(id));
    arr.len() != before
}

/// 纯 JSON 物化（UT-C-07）：将单条 op 幂等 apply 到房间 canonical 文档
/// （前端 Diagram JSON 形状）。返回是否已 apply（false = 未知 op / 缺字段，doc 不变）。
///
/// 规则（与前端 diff_snapshots / apply_remote 对齐）：
/// - table.create：changes 为含 fields 的全量表 JSON → 按 id upsert（缺 fields 补 []）
/// - table.update：合并 changes 中除 fields 外所有键，不触碰 fields；目标不存在按 create 兜底
/// - table.delete：按 id 删除（fields 随表嵌套删除）；幂等
/// - field.create/update：依 parentId 定位父表 fields 数组 upsert
/// - field.delete：依 parentId 定位父表 fields 数组删除；幂等
/// - fields.reorder：targetId=表 id，changes.fieldIds=字段 id 全量目标序；
///   列表外字段保持原相对序追加末尾，列表中未知 id 忽略；幂等
/// - reference/area/note create/update：扁平集合按 id upsert；delete 幂等
pub fn apply_op_to_doc(doc: &mut Value, op: &Value) -> bool {
    let Some(op_type) = op.get("type").and_then(|v| v.as_str()) else {
        return false;
    };
    let Some(target_id) = op.get("targetId").and_then(|v| v.as_str()) else {
        return false;
    };
    let mut it = op_type.split('.');
    let (Some(kind), Some(action)) = (it.next(), it.next()) else {
        return false;
    };
    let parent_id = op
        .get("parentId")
        .and_then(|v| v.as_str())
        .map(str::to_string);
    let changes = op.get("changes").cloned();
    let with_id = |mut item: Value| {
        if let Some(obj) = item.as_object_mut() {
            obj.entry("id".to_string())
                .or_insert_with(|| Value::String(target_id.to_string()));
        }
        item
    };

    match kind {
        "table" => {
            let Some(tables) = doc.get_mut("tables").and_then(|v| v.as_array_mut()) else {
                return false;
            };
            match action {
                "create" => {
                    let Some(mut t) = changes else { return false };
                    if let Some(obj) = t.as_object_mut() {
                        obj.entry("fields".to_string())
                            .or_insert_with(|| Value::Array(vec![]));
                    }
                    upsert_by_id(tables, target_id, with_id(t))
                }
                "update" => {
                    let Some(ch) = changes else { return false };
                    let Some(ch_obj) = ch.as_object() else { return false };
                    let slot = tables.iter_mut().find(|e| {
                        e.get("id").and_then(|v| v.as_str()) == Some(target_id)
                    });
                    match slot {
                        Some(slot) => {
                            let Some(slot_obj) = slot.as_object_mut() else { return false };
                            let mut changed = false;
                            for (k, v) in ch_obj {
                                if k == "fields" {
                                    continue;
                                }
                                if slot_obj.get(k) != Some(v) {
                                    slot_obj.insert(k.clone(), v.clone());
                                    changed = true;
                                }
                            }
                            changed
                        }
                        // 目标不存在：按 create 兜底（upsert，fields 补 []）
                        None => {
                            let mut t = ch.clone();
                            if let Some(obj) = t.as_object_mut() {
                                obj.entry("fields".to_string())
                                    .or_insert_with(|| Value::Array(vec![]));
                            }
                            upsert_by_id(tables, target_id, with_id(t))
                        }
                    }
                }
                "delete" => delete_by_id(tables, target_id),
                _ => false,
            }
        }
        "field" => {
            let Some(parent_id) = parent_id else { return false };
            let Some(tables) = doc.get_mut("tables").and_then(|v| v.as_array_mut()) else {
                return false;
            };
            let Some(table) = tables
                .iter_mut()
                .find(|e| e.get("id").and_then(|v| v.as_str()) == Some(parent_id.as_str()))
            else {
                return false;
            };
            let Some(fields) = table.get_mut("fields").and_then(|v| v.as_array_mut()) else {
                return false;
            };
            match action {
                "create" | "update" => {
                    let Some(f) = changes else { return false };
                    upsert_by_id(fields, target_id, with_id(f))
                }
                "delete" => delete_by_id(fields, target_id),
                _ => false,
            }
        }
        "fields" => {
            // fix-collab-autosave-race：字段换序（ST-SP-LIST-02 暴露的 op 模型缺口）。
            if action != "reorder" {
                return false;
            }
            let Some(ids) = changes
                .as_ref()
                .and_then(|c| c.get("fieldIds"))
                .and_then(|v| v.as_array())
            else {
                return false;
            };
            let order: Vec<&str> = ids.iter().filter_map(|v| v.as_str()).collect();
            let Some(tables) = doc.get_mut("tables").and_then(|v| v.as_array_mut()) else {
                return false;
            };
            let Some(table) = tables
                .iter_mut()
                .find(|e| e.get("id").and_then(|v| v.as_str()) == Some(target_id))
            else {
                return false;
            };
            let Some(fields) = table.get_mut("fields").and_then(|v| v.as_array_mut()) else {
                return false;
            };
            let pos = |f: &Value| {
                f.get("id")
                    .and_then(|v| v.as_str())
                    .and_then(|fid| order.iter().position(|&i| i == fid))
                    .unwrap_or(order.len())
            };
            let before: Vec<usize> = fields.iter().map(|f| pos(f)).collect();
            fields.sort_by_key(|f| pos(f));
            // 稳定排序：列表外字段（key=order.len()）保持原相对序追加末尾
            fields.iter().map(|f| pos(f)).ne(before.iter().copied())
        }
        "reference" | "area" | "note" => {
            let key = match kind {
                "reference" => "references",
                "area" => "areas",
                _ => "notes",
            };
            let Some(arr) = doc.get_mut(key).and_then(|v| v.as_array_mut()) else {
                return false;
            };
            match action {
                "create" | "update" => {
                    let Some(item) = changes else { return false };
                    upsert_by_id(arr, target_id, with_id(item))
                }
                "delete" => delete_by_id(arr, target_id),
                _ => false,
            }
        }
        // feat-data-dictionary（S07）：dict.* op 写入 doc["dictionaries"]；
        // 旧房间 doc_json 无此键 → create/update 时补建空数组（delete 无键视为无操作）
        "dict" => {
            if doc.get("dictionaries").is_none() {
                if action == "delete" {
                    return false;
                }
                doc.as_object_mut()
                    .map(|o| o.insert("dictionaries".to_string(), Value::Array(vec![])));
            }
            let Some(arr) = doc.get_mut("dictionaries").and_then(|v| v.as_array_mut()) else {
                return false;
            };
            match action {
                "create" | "update" => {
                    let Some(item) = changes else { return false };
                    upsert_by_id(arr, target_id, with_id(item))
                }
                "delete" => delete_by_id(arr, target_id),
                _ => false,
            }
        }
        _ => false,
    }
}

pub async fn list_member_presence(
    db: &DatabaseConnection,
    room_id: &str,
) -> Result<Vec<(String, Option<String>, String)>, CollabServiceError> {
    let rows = db
        .query_all(Statement::from_sql_and_values(
            DatabaseBackend::Sqlite,
            "SELECT rm.user_id, rm.role, u.display_name FROM room_member rm INNER JOIN user u ON u.id = rm.user_id WHERE rm.room_id = ? ORDER BY rm.joined_at ASC",
            vec![room_id.into()],
        ))
        .await
        .map_err(|e| CollabServiceError::Db(DrawDBError::DatabaseError(e)))?;
    Ok(rows
        .into_iter()
        .map(|r| {
            (
                r.try_get("", "user_id").unwrap_or_default(),
                r.try_get("", "display_name").ok(),
                r.try_get("", "role").unwrap_or_default(),
            )
        })
        .collect())
}

pub async fn authorize_ws(
    db: &DatabaseConnection,
    room_id: &str,
    user_id: &str,
) -> Result<String, CollabServiceError> {
    if load_room_diagram(db, room_id).await?.is_none() {
        return Err(CollabServiceError::RoomNotFound);
    }
    let role = get_member_role(db, room_id, user_id).await?;
    role.ok_or(CollabServiceError::NotAMember)
}

#[derive(Debug)]
pub enum CollabServiceError {
    RoomNotFound,
    NotAMember,
    ReadOnly,
    SyncGapTooLarge {
        current_server_rev: i64,
        max_catch_up: i64,
    },
    Validation(String),
    InvalidOp(String),
    Internal(String),
    Db(DrawDBError),
}

impl From<DrawDBError> for CollabServiceError {
    fn from(e: DrawDBError) -> Self {
        CollabServiceError::Db(e)
    }
}

impl From<RoomsServiceError> for CollabServiceError {
    fn from(e: RoomsServiceError) -> Self {
        match e {
            RoomsServiceError::RoomNotFound => CollabServiceError::RoomNotFound,
            RoomsServiceError::NotAMember => CollabServiceError::NotAMember,
            RoomsServiceError::Db(err) => CollabServiceError::Db(err),
            RoomsServiceError::Internal(msg) => CollabServiceError::Internal(msg),
            other => CollabServiceError::Internal(format!("{other:?}")),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn mark_pass(id: &'static str) {
        crate::verify_reporter::report_pass(id, 0);
    }

    fn base_doc() -> Value {
        json!({"id":"d1","name":"doc","database":null,"pan":null,"zoom":null,"revision":0,"tables":[],"references":[],"areas":[],"notes":[]})
    }

    /// UT-C-07a — table.create 全量 upsert（含 fields）→ update 合并标量不触碰 fields → delete 幂等
    #[test]
    fn ut_c07_table_lifecycle() {
        mark_pass("UT-C-07");
        let mut doc = base_doc();
        let create = json!({"type":"table.create","targetId":"t1","changes":{"id":"t1","name":"users","x":1.0,"y":2.0,"color":"#fff","comment":"","fields":[{"id":"f1","name":"id"}]}});
        assert!(apply_op_to_doc(&mut doc, &create));
        assert_eq!(doc["tables"].as_array().unwrap().len(), 1);
        assert_eq!(doc["tables"][0]["fields"].as_array().unwrap().len(), 1);

        // update：合并除 fields 外所有键；changes 里的 fields 必须被忽略
        let update = json!({"type":"table.update","targetId":"t1","changes":{"name":"users2","x":9.0,"fields":[]}});
        assert!(apply_op_to_doc(&mut doc, &update));
        assert_eq!(doc["tables"][0]["name"], "users2");
        assert_eq!(doc["tables"][0]["x"], 9.0);
        assert_eq!(doc["tables"][0]["fields"].as_array().unwrap().len(), 1, "fields 不得被 update 触碰");

        // update 无实际变化 → false（doc 未变）
        let noop = json!({"type":"table.update","targetId":"t1","changes":{"name":"users2"}});
        assert!(!apply_op_to_doc(&mut doc, &noop));

        // update 目标不存在 → create 兜底（fields 补 []）
        let ghost = json!({"type":"table.update","targetId":"t9","changes":{"name":"ghost"}});
        assert!(apply_op_to_doc(&mut doc, &ghost));
        assert_eq!(doc["tables"].as_array().unwrap().len(), 2);
        assert_eq!(doc["tables"][1]["fields"].as_array().unwrap().len(), 0);

        // delete 幂等
        let del = json!({"type":"table.delete","targetId":"t1"});
        assert!(apply_op_to_doc(&mut doc, &del));
        assert!(!apply_op_to_doc(&mut doc, &del));
        assert_eq!(doc["tables"].as_array().unwrap().len(), 1);
    }

    /// feat-data-dictionary（S07）— dict.* op：旧 doc 无 dictionaries 键时 create 补建数组；
    /// update upsert / delete 幂等（协议同 area/note 扁平集合口径）
    #[test]
    fn dict_op_lifecycle() {
        let mut doc = base_doc(); // 无 dictionaries 键（旧房间 doc_json）
        let create = json!({"type":"dict.create","targetId":"d1","changes":{"id":"d1","name":"是否","code":"yes_no","items":[{"id":"i1","value":"1","label":"是","sort":0}]}});
        assert!(apply_op_to_doc(&mut doc, &create));
        assert_eq!(doc["dictionaries"].as_array().unwrap().len(), 1);
        assert_eq!(doc["dictionaries"][0]["code"], "yes_no");

        let update = json!({"type":"dict.update","targetId":"d1","changes":{"id":"d1","name":"是否2","code":"yes_no","items":[]}});
        assert!(apply_op_to_doc(&mut doc, &update));
        assert_eq!(doc["dictionaries"][0]["name"], "是否2");

        // delete 幂等；无键 doc 的 delete 视为无操作
        let del = json!({"type":"dict.delete","targetId":"d1"});
        assert!(apply_op_to_doc(&mut doc, &del));
        assert!(!apply_op_to_doc(&mut doc, &del));
        let mut doc2 = base_doc();
        assert!(!apply_op_to_doc(&mut doc2, &del));
        assert!(doc2.get("dictionaries").is_none());
    }

    /// UT-C-07b — field.* 依 parentId 定位父表；create/update/delete + 幂等
    #[test]
    fn ut_c07_field_lifecycle() {
        mark_pass("UT-C-07");
        let mut doc = base_doc();
        apply_op_to_doc(&mut doc, &json!({"type":"table.create","targetId":"t1","changes":{"id":"t1","name":"users","fields":[]}}));
        assert!(apply_op_to_doc(&mut doc, &json!({"type":"field.create","targetId":"f1","parentId":"t1","changes":{"name":"email","type_":"VARCHAR"}})));
        let fields = doc["tables"][0]["fields"].as_array().unwrap();
        assert_eq!(fields.len(), 1);
        assert_eq!(fields[0]["id"], "f1", "缺 id 时由 targetId 补齐");
        assert!(apply_op_to_doc(&mut doc, &json!({"type":"field.update","targetId":"f1","parentId":"t1","changes":{"id":"f1","name":"email2"}})));
        assert_eq!(doc["tables"][0]["fields"][0]["name"], "email2");
        // parentId 缺失 / 父表不存在 → false
        assert!(!apply_op_to_doc(&mut doc, &json!({"type":"field.create","targetId":"f2","changes":{"name":"x"}})));
        assert!(!apply_op_to_doc(&mut doc, &json!({"type":"field.create","targetId":"f2","parentId":"tX","changes":{"name":"x"}})));
        let del = json!({"type":"field.delete","targetId":"f1","parentId":"t1"});
        assert!(apply_op_to_doc(&mut doc, &del));
        assert!(!apply_op_to_doc(&mut doc, &del));
    }

    /// UT-C-10 — fields.reorder：按 fieldIds 全量目标序重排；幂等；列表外字段相对序追加；
    /// 目标表不存在 → false 不改 doc（ST-SP-LIST-02 暴露的字段换序缺口）
    #[test]
    fn ut_c10_fields_reorder_materializes() {
        mark_pass("UT-C-10");
        let mut doc = base_doc();
        apply_op_to_doc(&mut doc, &json!({"type":"table.create","targetId":"t1","changes":{"id":"t1","name":"users","fields":[{"id":"f1"},{"id":"f2"},{"id":"f3"}]}}));

        let reorder = json!({"type":"fields.reorder","targetId":"t1","changes":{"fieldIds":["f3","f1","f2"]}});
        assert!(apply_op_to_doc(&mut doc, &reorder));
        let ids: Vec<&str> = doc["tables"][0]["fields"]
            .as_array()
            .unwrap()
            .iter()
            .map(|f| f["id"].as_str().unwrap())
            .collect();
        assert_eq!(ids, vec!["f3", "f1", "f2"]);
        // 幂等：同序重放 → false（doc 未变）
        assert!(!apply_op_to_doc(&mut doc, &reorder));

        // 列表外字段保持原相对序追加末尾
        apply_op_to_doc(&mut doc, &json!({"type":"field.create","targetId":"f4","parentId":"t1","changes":{"id":"f4"}}));
        let partial = json!({"type":"fields.reorder","targetId":"t1","changes":{"fieldIds":["f2","f3"]}});
        assert!(apply_op_to_doc(&mut doc, &partial));
        let ids: Vec<&str> = doc["tables"][0]["fields"]
            .as_array()
            .unwrap()
            .iter()
            .map(|f| f["id"].as_str().unwrap())
            .collect();
        assert_eq!(ids, vec!["f2", "f3", "f1", "f4"], "f1/f4 不在列表，保持相对序追加");

        // 未知 id 忽略；目标表不存在 → false
        let before = doc.clone();
        assert!(!apply_op_to_doc(&mut doc, &json!({"type":"fields.reorder","targetId":"tX","changes":{"fieldIds":["f1"]}})));
        assert_eq!(doc, before);
    }

    /// UT-C-07c — reference/area/note 扁平集合 upsert/delete；未知 op 不变更 doc
    #[test]
    fn ut_c07_flat_collections_and_unknown() {
        mark_pass("UT-C-07");
        let mut doc = base_doc();
        assert!(apply_op_to_doc(&mut doc, &json!({"type":"reference.create","targetId":"r1","changes":{"startTableId":"t1","endTableId":"t2"}})));
        assert!(apply_op_to_doc(&mut doc, &json!({"type":"area.create","targetId":"a1","changes":{"name":"区域","x":0.0}})));
        assert!(apply_op_to_doc(&mut doc, &json!({"type":"note.create","targetId":"n1","changes":{"content":"hi"}})));
        assert_eq!(doc["references"].as_array().unwrap().len(), 1);
        assert_eq!(doc["areas"].as_array().unwrap().len(), 1);
        assert_eq!(doc["notes"].as_array().unwrap().len(), 1);
        assert_eq!(doc["references"][0]["id"], "r1");
        // update=upsert 覆盖
        assert!(apply_op_to_doc(&mut doc, &json!({"type":"note.update","targetId":"n1","changes":{"content":"hi2"}})));
        assert_eq!(doc["notes"][0]["content"], "hi2");
        assert_eq!(doc["notes"].as_array().unwrap().len(), 1);
        // delete 幂等
        assert!(apply_op_to_doc(&mut doc, &json!({"type":"area.delete","targetId":"a1"})));
        assert!(!apply_op_to_doc(&mut doc, &json!({"type":"area.delete","targetId":"a1"})));
        // 未知 type / 缺 targetId → false 且 doc 不变
        let before = doc.clone();
        assert!(!apply_op_to_doc(&mut doc, &json!({"type":"diagram.rename","targetId":"d1"})));
        assert!(!apply_op_to_doc(&mut doc, &json!({"type":"table.create"})));
        assert_eq!(doc, before);
    }
}
