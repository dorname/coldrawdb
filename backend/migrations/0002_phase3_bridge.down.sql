DROP INDEX IF EXISTS idx_local_draft_import_status;
DROP TABLE IF EXISTS local_draft_import_log;
DROP TABLE IF EXISTS bridge_config;

DELETE FROM schema_migrations WHERE version = '0002_phase3_bridge';
