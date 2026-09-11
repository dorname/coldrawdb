DROP INDEX IF EXISTS idx_operation_log_user;
DROP INDEX IF EXISTS idx_operation_log_room_rev;
DROP TABLE IF EXISTS operation_log;

DROP INDEX IF EXISTS idx_operation_payload_hash;
DROP INDEX IF EXISTS idx_operation_type;
DROP TABLE IF EXISTS operation;

DROP TABLE IF EXISTS room_collab_head;
