-- V2 rooms schema for coldrawdb (S04).
-- Source scenario: core-S04-room-lifecycle.md
-- Engine: SQLite 3.40+ (WAL mode).
-- Depends on: coldrawdb-v2-auth.sql (user.id FK)

PRAGMA foreign_keys = ON;

-- ============================================================
-- 14. room — 协作房间（S04.1 create）
-- ============================================================
CREATE TABLE IF NOT EXISTS room (
    -- @comment 房间 UUID
    id                  TEXT    PRIMARY KEY,
    -- @comment 房间显示名
    name                TEXT    NOT NULL,
    -- @comment 绑定的 diagram，V2 约束一 diagram 一 active room
    diagram_id          TEXT    NOT NULL,
    -- @comment 创建者 user.id
    owner_id            TEXT    NOT NULL,
    -- @comment 归档时间，NULL 表示 active
    archived_at         TEXT,
    -- @comment 创建时间
    created_at          TEXT    NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    -- @comment 更新时间
    updated_at          TEXT    NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    FOREIGN KEY (diagram_id) REFERENCES diagram(id) ON DELETE CASCADE,
    FOREIGN KEY (owner_id) REFERENCES user(id) ON DELETE RESTRICT
);
-- @table-comment room S04 协作房间

CREATE INDEX IF NOT EXISTS idx_room_diagram_active ON room(diagram_id) WHERE archived_at IS NULL;
CREATE INDEX IF NOT EXISTS idx_room_owner ON room(owner_id);

-- ============================================================
-- 15. room_member — 成员与角色（S04.3 accept）
-- ============================================================
CREATE TABLE IF NOT EXISTS room_member (
    -- @comment 成员记录 UUID
    id                  TEXT    PRIMARY KEY,
    -- @comment 房间 ID
    room_id             TEXT    NOT NULL,
    -- @comment 用户 ID
    user_id             TEXT    NOT NULL,
    -- @comment 角色 owner / editor / viewer
    role                TEXT    NOT NULL CHECK(role IN ('owner', 'editor', 'viewer')),
    -- @comment 加入时间
    joined_at           TEXT    NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    FOREIGN KEY (room_id) REFERENCES room(id) ON DELETE CASCADE,
    FOREIGN KEY (user_id) REFERENCES user(id) ON DELETE CASCADE,
    UNIQUE(room_id, user_id)
);
-- @table-comment room_member S04 房间成员

CREATE INDEX IF NOT EXISTS idx_room_member_user ON room_member(user_id);

-- ============================================================
-- 16. room_invite — 邀请 token（S04.2 invite）
-- ============================================================
CREATE TABLE IF NOT EXISTS room_invite (
    -- @comment 邀请 UUID
    id                  TEXT    PRIMARY KEY,
    -- @comment URL token 哈希或明文 token（实现时仅存 hash）
    token_hash          TEXT    NOT NULL UNIQUE,
    -- @comment 目标房间
    room_id             TEXT    NOT NULL,
    -- @comment 邀请分配角色 editor / viewer
    role                TEXT    NOT NULL CHECK(role IN ('editor', 'viewer')),
    -- @comment 邀请人 user.id
    invited_by          TEXT    NOT NULL,
    -- @comment 过期时间
    expires_at          TEXT    NOT NULL,
    -- @comment 使用时间，NULL 表示未使用（可配置单次邀请）
    used_at             TEXT,
    -- @comment 创建时间
    created_at          TEXT    NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    FOREIGN KEY (room_id) REFERENCES room(id) ON DELETE CASCADE,
    FOREIGN KEY (invited_by) REFERENCES user(id) ON DELETE CASCADE
);
-- @table-comment room_invite S04 房间邀请

CREATE INDEX IF NOT EXISTS idx_room_invite_token ON room_invite(token_hash);
CREATE INDEX IF NOT EXISTS idx_room_invite_room ON room_invite(room_id);
