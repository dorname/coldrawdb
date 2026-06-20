mod hub;
mod ws;

pub use hub::CollabHub;
pub use ws::collab_ws_handler;

use sea_orm::{ConnectionTrait, DatabaseBackend, DatabaseConnection, Statement, TransactionTrait};
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
    if load_room_diagram(db, room_id).await?.is_none() {
        return Err(CollabServiceError::RoomNotFound);
    }
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
