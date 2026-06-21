-- V2 auth schema for coldrawdb (S03).
-- Source API: logos/resources/api/auth.yaml
-- Source scenario: core-S03-user-auth.md
-- Engine: SQLite 3.40+ (WAL mode), appended to coldrawdb-v1.sql on V2 migration.
-- Naming: snake_case columns; UUID as TEXT; timestamps ISO 8601 TEXT.

PRAGMA foreign_keys = ON;

-- ============================================================
-- 12. user — 注册用户（auth.yaml → register, login, me）
-- ============================================================
CREATE TABLE IF NOT EXISTS user (
    -- @comment 用户 UUID v4 字符串
    id                  TEXT    PRIMARY KEY,
    -- @comment 登录邮箱，全局唯一
    email               TEXT    NOT NULL UNIQUE,
    -- @comment Argon2id 密码哈希，明文永不存储
    password_hash       TEXT    NOT NULL,
    -- @comment 显示名称，可选
    display_name        TEXT,
    -- @comment 邮箱验证完成时间，NULL 表示未验证
    email_verified_at   TEXT,
    -- @comment 账户创建时间 ISO 8601
    created_at          TEXT    NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    -- @comment 最后更新时间，应用层维护
    updated_at          TEXT    NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);
-- @table-comment user S03 注册用户表

CREATE UNIQUE INDEX IF NOT EXISTS idx_user_email ON user(email);

-- ============================================================
-- 13. auth_token — refresh token 持久化（auth.yaml → login, refresh, logout）
-- ============================================================
CREATE TABLE IF NOT EXISTS auth_token (
    -- @comment refresh 记录 UUID
    id                  TEXT    PRIMARY KEY,
    -- @comment 所属用户
    user_id             TEXT    NOT NULL,
    -- @comment SHA-256(refresh_token) 哈希，原始 token 仅存 Cookie
    token_hash          TEXT    NOT NULL,
    -- @comment refresh 过期时间 ISO 8601
    expires_at          TEXT    NOT NULL,
    -- @comment 撤销时间，logout 或 rotation 时写入
    revoked_at          TEXT,
    -- @comment 创建时间
    created_at          TEXT    NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    FOREIGN KEY (user_id) REFERENCES user(id) ON DELETE CASCADE
);
-- @table-comment auth_token S03 refresh token 存储

-- 登录/refresh 按 hash 查找（S03 Step 10）
CREATE INDEX IF NOT EXISTS idx_auth_token_hash ON auth_token(token_hash);
-- 用户维度列出有效 token / logout all devices
CREATE INDEX IF NOT EXISTS idx_auth_token_user_active ON auth_token(user_id, revoked_at);
