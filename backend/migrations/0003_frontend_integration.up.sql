-- Phase 4: Frontend Integration
-- diagram 表新增列：gist_id、loaded_from_gist_id 及各嵌套实体 JSON 列
ALTER TABLE diagram ADD COLUMN gist_id TEXT;
ALTER TABLE diagram ADD COLUMN loaded_from_gist_id TEXT;
ALTER TABLE diagram ADD COLUMN tables_json TEXT;
ALTER TABLE diagram ADD COLUMN references_json TEXT;
ALTER TABLE diagram ADD COLUMN notes_json TEXT;
ALTER TABLE diagram ADD COLUMN areas_json TEXT;
ALTER TABLE diagram ADD COLUMN tasks_json TEXT;
ALTER TABLE diagram ADD COLUMN enums_json TEXT;
ALTER TABLE diagram ADD COLUMN types_json TEXT;

-- 新建 template 表
CREATE TABLE IF NOT EXISTS template (
    id TEXT PRIMARY KEY,
    title TEXT,
    description TEXT,
    database TEXT,
    custom INTEGER NOT NULL DEFAULT 0,
    tables_json TEXT,
    relationships_json TEXT,
    notes_json TEXT,
    subject_areas_json TEXT,
    todos_json TEXT,
    types_json TEXT,
    enums_json TEXT,
    pan TEXT,
    zoom TEXT,
    created_at TEXT DEFAULT (datetime('now')),
    updated_at TEXT DEFAULT (datetime('now'))
);
