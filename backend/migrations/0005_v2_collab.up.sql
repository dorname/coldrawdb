-- V2 collab schema (S05) — see logos/resources/database/coldrawdb-v2-collab.sql

PRAGMA foreign_keys = ON;

CREATE TABLE IF NOT EXISTS operation (
    id                  TEXT    PRIMARY KEY,
    op_type             TEXT    NOT NULL,
    payload             TEXT    NOT NULL,
    payload_hash        TEXT    NOT NULL,
    created_at          TEXT    NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);

CREATE INDEX IF NOT EXISTS idx_operation_type ON operation(op_type);
CREATE INDEX IF NOT EXISTS idx_operation_payload_hash ON operation(payload_hash);

CREATE TABLE IF NOT EXISTS operation_log (
    id                  TEXT    PRIMARY KEY,
    room_id             TEXT    NOT NULL,
    server_rev          INTEGER NOT NULL,
    operation_id        TEXT    NOT NULL,
    user_id             TEXT    NOT NULL,
    created_at          TEXT    NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    FOREIGN KEY (room_id) REFERENCES room(id) ON DELETE CASCADE,
    FOREIGN KEY (operation_id) REFERENCES operation(id) ON DELETE RESTRICT,
    FOREIGN KEY (user_id) REFERENCES user(id) ON DELETE CASCADE,
    UNIQUE(room_id, server_rev)
);

CREATE INDEX IF NOT EXISTS idx_operation_log_room_rev ON operation_log(room_id, server_rev);
CREATE INDEX IF NOT EXISTS idx_operation_log_user ON operation_log(user_id);

CREATE TABLE IF NOT EXISTS room_collab_head (
    room_id             TEXT    PRIMARY KEY,
    server_rev          INTEGER NOT NULL DEFAULT 0,
    snapshot_hash       TEXT,
    checkpoint_revision INTEGER,
    last_checkpoint_at  TEXT,
    updated_at          TEXT    NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    FOREIGN KEY (room_id) REFERENCES room(id) ON DELETE CASCADE
);
