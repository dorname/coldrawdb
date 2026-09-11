-- Phase 1: 数据库落地（v1 基线）

-- 1) schema migration tracking
CREATE TABLE IF NOT EXISTS schema_migrations (
    version TEXT PRIMARY KEY,
    applied_at TEXT NOT NULL DEFAULT (datetime('now'))
);

-- 2) diagram: 合并 last_modified -> updated_at；补 revision / is_deleted
ALTER TABLE diagram ADD COLUMN updated_at TEXT;
ALTER TABLE diagram ADD COLUMN revision INTEGER NOT NULL DEFAULT 0;
ALTER TABLE diagram ADD COLUMN is_deleted BOOLEAN NOT NULL DEFAULT 0;

-- 3) core entities: 补 is_deleted
ALTER TABLE "table" ADD COLUMN is_deleted BOOLEAN NOT NULL DEFAULT 0;
ALTER TABLE field ADD COLUMN is_deleted BOOLEAN NOT NULL DEFAULT 0;
ALTER TABLE indice ADD COLUMN is_deleted BOOLEAN NOT NULL DEFAULT 0;
ALTER TABLE reference ADD COLUMN is_deleted BOOLEAN NOT NULL DEFAULT 0;
ALTER TABLE note ADD COLUMN is_deleted BOOLEAN NOT NULL DEFAULT 0;
ALTER TABLE area ADD COLUMN is_deleted BOOLEAN NOT NULL DEFAULT 0;

-- 4) link consistency: reference -> reference_id
ALTER TABLE diagram_link RENAME COLUMN reference TO reference_id;

-- 5) link ordering
ALTER TABLE table_link ADD COLUMN order_no INTEGER NOT NULL DEFAULT 0;
ALTER TABLE indice_link ADD COLUMN order_no INTEGER NOT NULL DEFAULT 0;

-- 6) indexes
CREATE INDEX IF NOT EXISTS idx_diagram_link_diagram_id ON diagram_link(diagram_id);
CREATE INDEX IF NOT EXISTS idx_table_link_table_id ON table_link(table_id);
CREATE INDEX IF NOT EXISTS idx_indice_link_indice_id ON indice_link(indice_id);

-- 7) backfill updated_at
UPDATE diagram
SET updated_at = COALESCE(updated_at, CAST(last_modified AS TEXT), datetime('now'));
