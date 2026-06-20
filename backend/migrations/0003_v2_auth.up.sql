-- V2 auth schema (S03) — see logos/resources/database/coldrawdb-v2-auth.sql

PRAGMA foreign_keys = ON;

CREATE TABLE IF NOT EXISTS user (
    id                  TEXT    PRIMARY KEY,
    email               TEXT    NOT NULL UNIQUE,
    password_hash       TEXT    NOT NULL,
    display_name        TEXT,
    email_verified_at   TEXT,
    created_at          TEXT    NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    updated_at          TEXT    NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);

CREATE UNIQUE INDEX IF NOT EXISTS idx_user_email ON user(email);

CREATE TABLE IF NOT EXISTS auth_token (
    id                  TEXT    PRIMARY KEY,
    user_id             TEXT    NOT NULL,
    token_hash          TEXT    NOT NULL,
    expires_at          TEXT    NOT NULL,
    revoked_at          TEXT,
    created_at          TEXT    NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    FOREIGN KEY (user_id) REFERENCES user(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_auth_token_hash ON auth_token(token_hash);
CREATE INDEX IF NOT EXISTS idx_auth_token_user_active ON auth_token(user_id, revoked_at);
