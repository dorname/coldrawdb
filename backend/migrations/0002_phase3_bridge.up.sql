-- Phase 3: 迁移桥接能力（灰度配置 + 本地草稿导入记录）

CREATE TABLE IF NOT EXISTS bridge_config (
    id INTEGER PRIMARY KEY CHECK (id = 1),
    db_read_preferred BOOLEAN NOT NULL DEFAULT 1,
    db_write_enabled BOOLEAN NOT NULL DEFAULT 1,
    dual_write_local BOOLEAN NOT NULL DEFAULT 0,
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

INSERT OR IGNORE INTO bridge_config(id, db_read_preferred, db_write_enabled, dual_write_local)
VALUES (1, 1, 1, 0);

CREATE TABLE IF NOT EXISTS local_draft_import_log (
    id TEXT PRIMARY KEY,
    source TEXT,
    payload TEXT NOT NULL,
    imported_diagram_id TEXT,
    status TEXT NOT NULL,
    retry_count INTEGER NOT NULL DEFAULT 0,
    error_message TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_local_draft_import_status ON local_draft_import_log(status);
