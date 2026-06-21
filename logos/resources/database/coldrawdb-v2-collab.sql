-- V2 collab schema for coldrawdb (S05).
-- Source API: logos/resources/api/collab.yaml
-- Source scenario: core-S05-ot-collab.md
-- Engine: SQLite 3.40+ (WAL mode).
-- Depends on: coldrawdb-v2-auth.sql (user.id), coldrawdb-v2-rooms.sql (room.id)

PRAGMA foreign_keys = ON;

-- ============================================================
-- 17. operation — OT op 载荷（collab.yaml → op frame payload）
-- ============================================================
CREATE TABLE IF NOT EXISTS operation (
    -- @comment op UUID
    id                  TEXT    PRIMARY KEY,
    -- @comment op 类型，如 table.create / field.update
    op_type             TEXT    NOT NULL,
    -- @comment JSON 载荷，与 collab.yaml CollabOp 对齐
    payload             TEXT    NOT NULL,
    -- @comment SHA-256(payload) 十六进制，用于去重与校验
    payload_hash        TEXT    NOT NULL,
    -- @comment 创建时间
    created_at          TEXT    NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);
-- @table-comment operation S05 OT 操作载荷

CREATE INDEX IF NOT EXISTS idx_operation_type ON operation(op_type);
CREATE INDEX IF NOT EXISTS idx_operation_payload_hash ON operation(payload_hash);

-- ============================================================
-- 18. operation_log — room 内有序 server_rev 链（S05.2 append）
-- ============================================================
CREATE TABLE IF NOT EXISTS operation_log (
    -- @comment 日志条目 UUID
    id                  TEXT    PRIMARY KEY,
    -- @comment 所属 room
    room_id             TEXT    NOT NULL,
    -- @comment room 内单调递增 revision（从 1 开始）
    server_rev          INTEGER NOT NULL,
    -- @comment 关联 operation.id
    operation_id        TEXT    NOT NULL,
    -- @comment 提交者 user.id
    user_id             TEXT    NOT NULL,
    -- @comment 写入时间
    created_at          TEXT    NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    FOREIGN KEY (room_id) REFERENCES room(id) ON DELETE CASCADE,
    FOREIGN KEY (operation_id) REFERENCES operation(id) ON DELETE RESTRICT,
    FOREIGN KEY (user_id) REFERENCES user(id) ON DELETE CASCADE,
    UNIQUE(room_id, server_rev)
);
-- @table-comment operation_log S05 room 内 OT 有序日志

CREATE INDEX IF NOT EXISTS idx_operation_log_room_rev ON operation_log(room_id, server_rev);
CREATE INDEX IF NOT EXISTS idx_operation_log_user ON operation_log(user_id);

-- ============================================================
-- 19. room_collab_head — room 协作头指针（S05.1 connected 帧）
-- ============================================================
CREATE TABLE IF NOT EXISTS room_collab_head (
    -- @comment room.id，每 room 一行
    room_id             TEXT    PRIMARY KEY,
    -- @comment 当前最大 server_rev，0 表示尚无 op
    server_rev          INTEGER NOT NULL DEFAULT 0,
    -- @comment diagram 快照哈希（checkpoint 后更新，可选）
    snapshot_hash       TEXT,
    -- @comment 最近一次 REST checkpoint 的 diagram.revision
    checkpoint_revision INTEGER,
    -- @comment 最后 checkpoint 时间
    last_checkpoint_at  TEXT,
    -- @comment 头指针更新时间
    updated_at          TEXT    NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    FOREIGN KEY (room_id) REFERENCES room(id) ON DELETE CASCADE
);
-- @table-comment room_collab_head S05 协作 revision 头
