-- Phase 1 down migration (best-effort for SQLite)
-- NOTE: SQLite 对 DROP COLUMN 支持有限，此处提供最小回退（仅清理索引与迁移记录）。

DROP INDEX IF EXISTS idx_diagram_link_diagram_id;
DROP INDEX IF EXISTS idx_table_link_table_id;
DROP INDEX IF EXISTS idx_indice_link_indice_id;

DELETE FROM schema_migrations WHERE version = '0001_phase1_schema';
