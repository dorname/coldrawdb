use rand::Rng;
use sea_orm::{ConnectionTrait, DatabaseBackend, DatabaseConnection, Statement, TransactionTrait};
use sha2::{Digest, Sha256};
use uuid::Uuid;

use crate::error::DrawDBError;
use crate::next_id;

const INVITE_TTL_DAYS: i64 = 7;

#[derive(Debug, Clone)]
pub struct RoomRow {
    pub id: String,
    pub name: String,
    pub diagram_id: String,
    pub owner_id: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone)]
pub struct RoomDetail {
    pub room: RoomRow,
    pub diagram_title: String,
    pub my_role: String,
    pub member_count: i64,
}

#[derive(Debug, Clone)]
pub struct RoomSummary {
    pub id: String,
    pub name: String,
    pub diagram_id: String,
    pub diagram_title: String,
    pub my_role: String,
    pub member_count: i64,
    pub updated_at: String,
}

#[derive(Debug, Clone)]
pub struct InviteCreated {
    pub invite_url: String,
    pub token: String,
    pub role: String,
    pub expires_at: String,
}

#[derive(Debug, Clone)]
pub struct InvitePreview {
    pub room_name: String,
    pub diagram_title: String,
    pub diagram_id: String,
    pub role: String,
    pub invited_by: Option<String>,
    pub expires_at: String,
}

#[derive(Debug, Clone)]
pub struct AcceptInviteResult {
    pub room_id: String,
    pub diagram_id: String,
    pub role: String,
    pub already_member: bool,
}

#[derive(Debug, Clone)]
pub struct RoomMemberRow {
    pub user_id: String,
    pub email: String,
    pub display_name: Option<String>,
    pub role: String,
    pub joined_at: String,
}

fn token_hash(raw: &str) -> String {
    let digest = Sha256::digest(raw.as_bytes());
    hex::encode(digest)
}

fn generate_invite_token() -> String {
    let bytes: [u8; 32] = rand::thread_rng().gen();
    hex::encode(bytes)
}

async fn diagram_exists(db: &DatabaseConnection, diagram_id: &str) -> Result<bool, RoomsServiceError> {
    let row = db
        .query_one(Statement::from_sql_and_values(
            DatabaseBackend::Sqlite,
            "SELECT id FROM diagram WHERE id = ? AND (is_deleted = 0 OR is_deleted IS NULL) LIMIT 1",
            vec![diagram_id.into()],
        ))
        .await
        .map_err(|e| RoomsServiceError::Db(DrawDBError::DatabaseError(e)))?;
    Ok(row.is_some())
}

async fn diagram_title(db: &DatabaseConnection, diagram_id: &str) -> Result<String, RoomsServiceError> {
    let row = db
        .query_one(Statement::from_sql_and_values(
            DatabaseBackend::Sqlite,
            "SELECT name FROM diagram WHERE id = ? LIMIT 1",
            vec![diagram_id.into()],
        ))
        .await
        .map_err(|e| RoomsServiceError::Db(DrawDBError::DatabaseError(e)))?;
    Ok(row
        .and_then(|r| r.try_get("", "name").ok())
        .flatten()
        .unwrap_or_else(|| "Untitled".to_string()))
}

async fn active_room_for_diagram(
    db: &DatabaseConnection,
    diagram_id: &str,
) -> Result<Option<String>, RoomsServiceError> {
    let row = db
        .query_one(Statement::from_sql_and_values(
            DatabaseBackend::Sqlite,
            "SELECT id FROM room WHERE diagram_id = ? AND archived_at IS NULL LIMIT 1",
            vec![diagram_id.into()],
        ))
        .await
        .map_err(|e| RoomsServiceError::Db(DrawDBError::DatabaseError(e)))?;
    Ok(row.and_then(|r| r.try_get("", "id").ok()))
}

pub async fn get_member_role(
    db: &DatabaseConnection,
    room_id: &str,
    user_id: &str,
) -> Result<Option<String>, RoomsServiceError> {
    let row = db
        .query_one(Statement::from_sql_and_values(
            DatabaseBackend::Sqlite,
            "SELECT rm.role FROM room_member rm INNER JOIN room r ON r.id = rm.room_id WHERE rm.room_id = ? AND rm.user_id = ? AND r.archived_at IS NULL LIMIT 1",
            vec![room_id.into(), user_id.into()],
        ))
        .await
        .map_err(|e| RoomsServiceError::Db(DrawDBError::DatabaseError(e)))?;
    Ok(row.and_then(|r| r.try_get("", "role").ok()))
}

async fn member_count(db: &DatabaseConnection, room_id: &str) -> Result<i64, RoomsServiceError> {
    let row = db
        .query_one(Statement::from_sql_and_values(
            DatabaseBackend::Sqlite,
            "SELECT COUNT(1) AS c FROM room_member WHERE room_id = ?",
            vec![room_id.into()],
        ))
        .await
        .map_err(|e| RoomsServiceError::Db(DrawDBError::DatabaseError(e)))?;
    Ok(row
        .and_then(|r| r.try_get::<i64>("", "c").ok())
        .unwrap_or(0))
}

async fn load_active_room(
    db: &DatabaseConnection,
    room_id: &str,
) -> Result<Option<RoomRow>, RoomsServiceError> {
    let row = db
        .query_one(Statement::from_sql_and_values(
            DatabaseBackend::Sqlite,
            "SELECT id, name, diagram_id, owner_id, created_at, updated_at FROM room WHERE id = ? AND archived_at IS NULL LIMIT 1",
            vec![room_id.into()],
        ))
        .await
        .map_err(|e| RoomsServiceError::Db(DrawDBError::DatabaseError(e)))?;
    Ok(row.map(|r| RoomRow {
        id: r.try_get("", "id").unwrap_or_default(),
        name: r.try_get("", "name").unwrap_or_default(),
        diagram_id: r.try_get("", "diagram_id").unwrap_or_default(),
        owner_id: r.try_get("", "owner_id").unwrap_or_default(),
        created_at: r.try_get("", "created_at").unwrap_or_default(),
        updated_at: r.try_get("", "updated_at").unwrap_or_default(),
    }))
}

pub async fn create_room(
    db: &DatabaseConnection,
    owner_id: &str,
    name: &str,
    diagram_id: &str,
) -> Result<RoomRow, RoomsServiceError> {
    if name.trim().is_empty() || name.len() > 64 {
        return Err(RoomsServiceError::Validation {
            fields: vec![("name".into(), "房间名称长度 1-64".into())],
        });
    }
    if diagram_id.trim().is_empty() {
        return Err(RoomsServiceError::Validation {
            fields: vec![("diagramId".into(), "diagramId 不能为空".into())],
        });
    }
    if !diagram_exists(db, diagram_id).await? {
        return Err(RoomsServiceError::DiagramNotFound);
    }
    if let Some(existing) = active_room_for_diagram(db, diagram_id).await? {
        return Err(RoomsServiceError::DiagramTaken { existing_room_id: existing });
    }

    let room_id = Uuid::new_v4().to_string();
    let member_id = next_id();
    let tx = db.begin().await.map_err(|e| RoomsServiceError::Db(DrawDBError::DatabaseError(e)))?;

    tx.execute(Statement::from_sql_and_values(
        DatabaseBackend::Sqlite,
        "INSERT INTO room(id, name, diagram_id, owner_id, archived_at, created_at, updated_at) VALUES(?, ?, ?, ?, NULL, strftime('%Y-%m-%dT%H:%M:%fZ', 'now'), strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))",
        vec![
            room_id.clone().into(),
            name.into(),
            diagram_id.into(),
            owner_id.into(),
        ],
    ))
    .await
    .map_err(|e| RoomsServiceError::Db(DrawDBError::DatabaseError(e)))?;

    tx.execute(Statement::from_sql_and_values(
        DatabaseBackend::Sqlite,
        "INSERT INTO room_member(id, room_id, user_id, role, joined_at) VALUES(?, ?, ?, 'owner', strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))",
        vec![member_id.into(), room_id.clone().into(), owner_id.into()],
    ))
    .await
    .map_err(|e| RoomsServiceError::Db(DrawDBError::DatabaseError(e)))?;

    tx.commit()
        .await
        .map_err(|e| RoomsServiceError::Db(DrawDBError::DatabaseError(e)))?;

    load_active_room(db, &room_id)
        .await?
        .ok_or(RoomsServiceError::Internal("room create failed".into()))
}

pub async fn list_rooms(
    db: &DatabaseConnection,
    user_id: &str,
    limit: u64,
    offset: u64,
) -> Result<(Vec<RoomSummary>, i64), RoomsServiceError> {
    let limit = limit.clamp(1, 100);
    let count_row = db
        .query_one(Statement::from_sql_and_values(
            DatabaseBackend::Sqlite,
            "SELECT COUNT(DISTINCT r.id) AS c FROM room r INNER JOIN room_member rm ON rm.room_id = r.id WHERE rm.user_id = ? AND r.archived_at IS NULL",
            vec![user_id.into()],
        ))
        .await
        .map_err(|e| RoomsServiceError::Db(DrawDBError::DatabaseError(e)))?;
    let total: i64 = count_row
        .and_then(|r| r.try_get("", "c").ok())
        .unwrap_or(0);

    let rows = db
        .query_all(Statement::from_sql_and_values(
            DatabaseBackend::Sqlite,
            "SELECT r.id, r.name, r.diagram_id, r.updated_at, rm.role, d.name AS diagram_title, (SELECT COUNT(1) FROM room_member m WHERE m.room_id = r.id) AS member_count FROM room r INNER JOIN room_member rm ON rm.room_id = r.id LEFT JOIN diagram d ON d.id = r.diagram_id WHERE rm.user_id = ? AND r.archived_at IS NULL ORDER BY r.updated_at DESC LIMIT ? OFFSET ?",
            vec![user_id.into(), (limit as i64).into(), (offset as i64).into()],
        ))
        .await
        .map_err(|e| RoomsServiceError::Db(DrawDBError::DatabaseError(e)))?;

    let items = rows
        .into_iter()
        .map(|r| RoomSummary {
            id: r.try_get("", "id").unwrap_or_default(),
            name: r.try_get("", "name").unwrap_or_default(),
            diagram_id: r.try_get("", "diagram_id").unwrap_or_default(),
            diagram_title: r
                .try_get::<Option<String>>("", "diagram_title")
                .ok()
                .flatten()
                .unwrap_or_else(|| "Untitled".to_string()),
            my_role: r.try_get("", "role").unwrap_or_default(),
            member_count: r.try_get("", "member_count").unwrap_or(1),
            updated_at: r.try_get("", "updated_at").unwrap_or_default(),
        })
        .collect();

    Ok((items, total))
}

pub async fn get_room_detail(
    db: &DatabaseConnection,
    room_id: &str,
    user_id: &str,
) -> Result<RoomDetail, RoomsServiceError> {
    let room = load_active_room(db, room_id)
        .await?
        .ok_or(RoomsServiceError::RoomNotFound)?;
    let role = get_member_role(db, room_id, user_id).await?;
    let Some(my_role) = role else {
        return Err(RoomsServiceError::NotAMember);
    };
    Ok(RoomDetail {
        diagram_title: diagram_title(db, &room.diagram_id).await?,
        my_role,
        member_count: member_count(db, room_id).await?,
        room,
    })
}

pub async fn archive_room(
    db: &DatabaseConnection,
    room_id: &str,
    user_id: &str,
) -> Result<(), RoomsServiceError> {
    let room = load_active_room(db, room_id)
        .await?
        .ok_or(RoomsServiceError::RoomNotFound)?;
    if room.owner_id != user_id {
        return Err(RoomsServiceError::Forbidden("仅房间 owner 可删除房间".into()));
    }
    db.execute(Statement::from_sql_and_values(
        DatabaseBackend::Sqlite,
        "UPDATE room SET archived_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now'), updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now') WHERE id = ?",
        vec![room_id.into()],
    ))
    .await
    .map_err(|e| RoomsServiceError::Db(DrawDBError::DatabaseError(e)))?;
    Ok(())
}

pub async fn create_invite(
    db: &DatabaseConnection,
    room_id: &str,
    inviter_id: &str,
    role: &str,
) -> Result<InviteCreated, RoomsServiceError> {
    if role != "editor" && role != "viewer" {
        return Err(RoomsServiceError::Validation {
            fields: vec![("role".into(), "role 必须为 editor 或 viewer".into())],
        });
    }
    if load_active_room(db, room_id).await?.is_none() {
        return Err(RoomsServiceError::RoomNotFound);
    }
    let member_role = get_member_role(db, room_id, inviter_id).await?;
    let Some(r) = member_role else {
        return Err(RoomsServiceError::Forbidden("你没有邀请权限".into()));
    };
    if r != "owner" && r != "editor" {
        return Err(RoomsServiceError::Forbidden("你没有邀请权限".into()));
    }

    let token = generate_invite_token();
    let hash = token_hash(&token);
    let invite_id = Uuid::new_v4().to_string();
    let expires_at = (chrono::Utc::now() + chrono::Duration::days(INVITE_TTL_DAYS)).to_rfc3339();

    db.execute(Statement::from_sql_and_values(
        DatabaseBackend::Sqlite,
        "INSERT INTO room_invite(id, token_hash, room_id, role, invited_by, expires_at, used_at, created_at) VALUES(?, ?, ?, ?, ?, ?, NULL, strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))",
        vec![
            invite_id.into(),
            hash.into(),
            room_id.into(),
            role.into(),
            inviter_id.into(),
            expires_at.clone().into(),
        ],
    ))
    .await
    .map_err(|e| RoomsServiceError::Db(DrawDBError::DatabaseError(e)))?;

    Ok(InviteCreated {
        invite_url: format!("http://localhost/invite/{token}"),
        token,
        role: role.to_string(),
        expires_at,
    })
}

async fn load_invite_by_token(
    db: &DatabaseConnection,
    raw_token: &str,
) -> Result<(String, String, String, String, String, Option<String>, String), RoomsServiceError> {
    let hash = token_hash(raw_token);
    let row = db
        .query_one(Statement::from_sql_and_values(
            DatabaseBackend::Sqlite,
            "SELECT ri.id, ri.room_id, ri.role, ri.expires_at, ri.used_at, r.name AS room_name, r.diagram_id, u.display_name, u.email FROM room_invite ri INNER JOIN room r ON r.id = ri.room_id LEFT JOIN user u ON u.id = ri.invited_by WHERE ri.token_hash = ? AND r.archived_at IS NULL LIMIT 1",
            vec![hash.into()],
        ))
        .await
        .map_err(|e| RoomsServiceError::Db(DrawDBError::DatabaseError(e)))?;

    let Some(row) = row else {
        return Err(RoomsServiceError::InviteNotFound);
    };

    let expires_at: String = row.try_get("", "expires_at").unwrap_or_default();
    let exp = chrono::DateTime::parse_from_rfc3339(&expires_at)
        .map_err(|e| RoomsServiceError::Internal(e.to_string()))?
        .with_timezone(&chrono::Utc);
    if exp < chrono::Utc::now() {
        return Err(RoomsServiceError::InviteExpired);
    }

    Ok((
        row.try_get("", "id").unwrap_or_default(),
        row.try_get("", "room_id").unwrap_or_default(),
        row.try_get("", "room_name").unwrap_or_default(),
        row.try_get("", "diagram_id").unwrap_or_default(),
        row.try_get("", "role").unwrap_or_default(),
        row.try_get::<Option<String>>("", "display_name")
            .ok()
            .flatten()
            .or_else(|| row.try_get::<Option<String>>("", "email").ok().flatten()),
        expires_at,
    ))
}

pub async fn preview_invite(
    db: &DatabaseConnection,
    raw_token: &str,
) -> Result<InvitePreview, RoomsServiceError> {
    let (_, _, room_name, diagram_id, role, invited_by, expires_at) =
        load_invite_by_token(db, raw_token).await?;
    Ok(InvitePreview {
        room_name,
        diagram_title: diagram_title(db, &diagram_id).await?,
        diagram_id,
        role,
        invited_by,
        expires_at,
    })
}

pub async fn accept_invite(
    db: &DatabaseConnection,
    raw_token: &str,
    user_id: &str,
) -> Result<AcceptInviteResult, RoomsServiceError> {
    let (invite_id, room_id, _, diagram_id, role, _, _) =
        load_invite_by_token(db, raw_token).await?;

    if let Some(existing_role) = get_member_role(db, &room_id, user_id).await? {
        return Ok(AcceptInviteResult {
            room_id: room_id.clone(),
            diagram_id: diagram_id.clone(),
            role: existing_role,
            already_member: true,
        });
    }

    let member_id = next_id();
    let tx = db.begin().await.map_err(|e| RoomsServiceError::Db(DrawDBError::DatabaseError(e)))?;
    tx.execute(Statement::from_sql_and_values(
        DatabaseBackend::Sqlite,
        "INSERT INTO room_member(id, room_id, user_id, role, joined_at) VALUES(?, ?, ?, ?, strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))",
        vec![
            member_id.into(),
            room_id.clone().into(),
            user_id.into(),
            role.clone().into(),
        ],
    ))
    .await
    .map_err(|e| RoomsServiceError::Db(DrawDBError::DatabaseError(e)))?;
    tx.execute(Statement::from_sql_and_values(
        DatabaseBackend::Sqlite,
        "UPDATE room_invite SET used_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now') WHERE id = ?",
        vec![invite_id.into()],
    ))
    .await
    .map_err(|e| RoomsServiceError::Db(DrawDBError::DatabaseError(e)))?;
    tx.commit()
        .await
        .map_err(|e| RoomsServiceError::Db(DrawDBError::DatabaseError(e)))?;

    Ok(AcceptInviteResult {
        room_id,
        diagram_id,
        role,
        already_member: false,
    })
}

pub async fn list_members(
    db: &DatabaseConnection,
    room_id: &str,
    user_id: &str,
) -> Result<Vec<RoomMemberRow>, RoomsServiceError> {
    if load_active_room(db, room_id).await?.is_none() {
        return Err(RoomsServiceError::RoomNotFound);
    }
    if get_member_role(db, room_id, user_id).await?.is_none() {
        return Err(RoomsServiceError::NotAMember);
    }
    let rows = db
        .query_all(Statement::from_sql_and_values(
            DatabaseBackend::Sqlite,
            "SELECT rm.user_id, rm.role, rm.joined_at, u.email, u.display_name FROM room_member rm INNER JOIN user u ON u.id = rm.user_id WHERE rm.room_id = ? ORDER BY rm.joined_at ASC",
            vec![room_id.into()],
        ))
        .await
        .map_err(|e| RoomsServiceError::Db(DrawDBError::DatabaseError(e)))?;

    Ok(rows
        .into_iter()
        .map(|r| RoomMemberRow {
            user_id: r.try_get("", "user_id").unwrap_or_default(),
            email: r.try_get("", "email").unwrap_or_default(),
            display_name: r.try_get("", "display_name").ok(),
            role: r.try_get("", "role").unwrap_or_default(),
            joined_at: r.try_get("", "joined_at").unwrap_or_default(),
        })
        .collect())
}

pub async fn leave_room(
    db: &DatabaseConnection,
    room_id: &str,
    user_id: &str,
) -> Result<(), RoomsServiceError> {
    if load_active_room(db, room_id).await?.is_none() {
        return Err(RoomsServiceError::RoomNotFound);
    }
    let role = get_member_role(db, room_id, user_id).await?;
    let Some(r) = role else {
        return Err(RoomsServiceError::NotAMember);
    };
    if r == "owner" {
        return Err(RoomsServiceError::OwnerCannotLeave);
    }
    db.execute(Statement::from_sql_and_values(
        DatabaseBackend::Sqlite,
        "DELETE FROM room_member WHERE room_id = ? AND user_id = ?",
        vec![room_id.into(), user_id.into()],
    ))
    .await
    .map_err(|e| RoomsServiceError::Db(DrawDBError::DatabaseError(e)))?;
    Ok(())
}

pub async fn update_member_role(
    db: &DatabaseConnection,
    room_id: &str,
    actor_id: &str,
    target_user_id: &str,
    role: &str,
) -> Result<RoomMemberRow, RoomsServiceError> {
    if role != "editor" && role != "viewer" {
        return Err(RoomsServiceError::Validation {
            fields: vec![("role".into(), "role 必须为 editor 或 viewer".into())],
        });
    }
    let room = load_active_room(db, room_id)
        .await?
        .ok_or(RoomsServiceError::RoomNotFound)?;
    if room.owner_id != actor_id {
        return Err(RoomsServiceError::Forbidden("仅房间 owner 可修改成员角色".into()));
    }
    if target_user_id == room.owner_id {
        return Err(RoomsServiceError::MemberNotFound);
    }
    let existing = get_member_role(db, room_id, target_user_id).await?;
    if existing.is_none() {
        return Err(RoomsServiceError::MemberNotFound);
    }
    db.execute(Statement::from_sql_and_values(
        DatabaseBackend::Sqlite,
        "UPDATE room_member SET role = ? WHERE room_id = ? AND user_id = ?",
        vec![role.into(), room_id.into(), target_user_id.into()],
    ))
    .await
    .map_err(|e| RoomsServiceError::Db(DrawDBError::DatabaseError(e)))?;

    let rows = list_members(db, room_id, actor_id).await?;
    rows.into_iter()
        .find(|m| m.user_id == target_user_id)
        .ok_or(RoomsServiceError::MemberNotFound)
}

pub async fn remove_member(
    db: &DatabaseConnection,
    room_id: &str,
    actor_id: &str,
    target_user_id: &str,
) -> Result<(), RoomsServiceError> {
    let room = load_active_room(db, room_id)
        .await?
        .ok_or(RoomsServiceError::RoomNotFound)?;
    if room.owner_id != actor_id {
        return Err(RoomsServiceError::Forbidden("仅房间 owner 可移除成员".into()));
    }
    if target_user_id == room.owner_id {
        return Err(RoomsServiceError::CannotRemoveOwner);
    }
    let existing = get_member_role(db, room_id, target_user_id).await?;
    if existing.is_none() {
        return Err(RoomsServiceError::MemberNotFound);
    }
    db.execute(Statement::from_sql_and_values(
        DatabaseBackend::Sqlite,
        "DELETE FROM room_member WHERE room_id = ? AND user_id = ?",
        vec![room_id.into(), target_user_id.into()],
    ))
    .await
    .map_err(|e| RoomsServiceError::Db(DrawDBError::DatabaseError(e)))?;
    Ok(())
}

#[derive(Debug)]
pub enum RoomsServiceError {
    DiagramNotFound,
    DiagramTaken { existing_room_id: String },
    RoomNotFound,
    NotAMember,
    Forbidden(String),
    InviteNotFound,
    InviteExpired,
    OwnerCannotLeave,
    CannotRemoveOwner,
    MemberNotFound,
    Validation { fields: Vec<(String, String)> },
    Internal(String),
    Db(DrawDBError),
}

impl From<DrawDBError> for RoomsServiceError {
    fn from(e: DrawDBError) -> Self {
        RoomsServiceError::Db(e)
    }
}
