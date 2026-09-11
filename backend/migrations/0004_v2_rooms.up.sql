-- V2 rooms schema (S04) — see logos/resources/database/coldrawdb-v2-rooms.sql

PRAGMA foreign_keys = ON;

CREATE TABLE IF NOT EXISTS room (
    id                  TEXT    PRIMARY KEY,
    name                TEXT    NOT NULL,
    diagram_id          TEXT    NOT NULL,
    owner_id            TEXT    NOT NULL,
    archived_at         TEXT,
    created_at          TEXT    NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    updated_at          TEXT    NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    FOREIGN KEY (diagram_id) REFERENCES diagram(id) ON DELETE CASCADE,
    FOREIGN KEY (owner_id) REFERENCES user(id) ON DELETE RESTRICT
);

CREATE INDEX IF NOT EXISTS idx_room_diagram_active ON room(diagram_id) WHERE archived_at IS NULL;
CREATE INDEX IF NOT EXISTS idx_room_owner ON room(owner_id);

CREATE TABLE IF NOT EXISTS room_member (
    id                  TEXT    PRIMARY KEY,
    room_id             TEXT    NOT NULL,
    user_id             TEXT    NOT NULL,
    role                TEXT    NOT NULL CHECK(role IN ('owner', 'editor', 'viewer')),
    joined_at           TEXT    NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    FOREIGN KEY (room_id) REFERENCES room(id) ON DELETE CASCADE,
    FOREIGN KEY (user_id) REFERENCES user(id) ON DELETE CASCADE,
    UNIQUE(room_id, user_id)
);

CREATE INDEX IF NOT EXISTS idx_room_member_user ON room_member(user_id);

CREATE TABLE IF NOT EXISTS room_invite (
    id                  TEXT    PRIMARY KEY,
    token_hash          TEXT    NOT NULL UNIQUE,
    room_id             TEXT    NOT NULL,
    role                TEXT    NOT NULL CHECK(role IN ('editor', 'viewer')),
    invited_by          TEXT    NOT NULL,
    expires_at          TEXT    NOT NULL,
    used_at             TEXT,
    created_at          TEXT    NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    FOREIGN KEY (room_id) REFERENCES room(id) ON DELETE CASCADE,
    FOREIGN KEY (invited_by) REFERENCES user(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_room_invite_token ON room_invite(token_hash);
CREATE INDEX IF NOT EXISTS idx_room_invite_room ON room_invite(room_id);
