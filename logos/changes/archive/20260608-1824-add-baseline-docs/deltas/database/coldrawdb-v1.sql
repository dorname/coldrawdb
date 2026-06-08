-- ADDED — V1 DDL（11 张表）
-- 模块：core | 提案：add-baseline-docs
-- 路径：`logos/resources/database/coldrawdb-v1.sql`
-- 对齐参考源：`backend/init.sql` + `database_design.json` + `backend/src/entity/*`

-- V1 schema for coldrawdb. 11 tables total. Aligned with `backend/init.sql` and
-- `database_design.json` for column naming and types.
-- -- Engine: SQLite 3.40+ (WAL mode)
-- Naming: snake_case for columns; BIGINT auto-increment primary keys; UUID stored as TEXT
-- -- 表清单（11 张）：
--   1. task
--   2. diagram
--   3. diagram_link
--   4. table
--   5. field
--   6. table_link
--   7. indice
--   8. indice_link
--   9. reference
--   10. area
--   11. note

PRAGMA foreign_keys = ON;
PRAGMA journal_mode = WAL;

-- ============================================================
-- 1. task — 导入任务日志（todos 领域）
-- ============================================================
CREATE TABLE IF NOT EXISTS task (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    type            TEXT    NOT NULL CHECK(type IN ('import_sql', 'import_dbml', 'import_json')),
    status          TEXT    NOT NULL DEFAULT 'pending' CHECK(status IN ('pending', 'success', 'failed')),
    message         TEXT    NOT NULL DEFAULT '',
    diagram_id      TEXT    NOT NULL DEFAULT '',
    payload         TEXT    NOT NULL DEFAULT '',
    created_at      TEXT    NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    completed_at    TEXT
);
CREATE INDEX IF NOT EXISTS idx_task_type_status ON task(type, status);
CREATE INDEX IF NOT EXISTS idx_task_created_at ON task(created_at DESC);

-- ============================================================
-- 2. diagram — 主表（diagrams 领域）
-- ============================================================
CREATE TABLE IF NOT EXISTS diagram (
    id              TEXT    PRIMARY KEY,
    title           TEXT    NOT NULL,
    revision        INTEGER NOT NULL DEFAULT 0,
    created_at      TEXT    NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    updated_at      TEXT    NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);
CREATE INDEX IF NOT EXISTS idx_diagram_updated_at ON diagram(updated_at DESC);

-- ============================================================
-- 3. diagram_link — diagram 关联
-- ============================================================
CREATE TABLE IF NOT EXISTS diagram_link (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    source_id       TEXT    NOT NULL,
    target_id       TEXT    NOT NULL,
    created_at      TEXT    NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    FOREIGN KEY (source_id) REFERENCES diagram(id) ON DELETE CASCADE,
    FOREIGN KEY (target_id) REFERENCES diagram(id) ON DELETE CASCADE
);
CREATE INDEX IF NOT EXISTS idx_diagram_link_source ON diagram_link(source_id);
CREATE INDEX IF NOT EXISTS idx_diagram_link_target ON diagram_link(target_id);

-- ============================================================
-- 4. table — 表元数据（tables 领域）
-- ============================================================
CREATE TABLE IF NOT EXISTS `table` (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    diagram_id      TEXT    NOT NULL,
    name            TEXT    NOT NULL,
    x               REAL    NOT NULL DEFAULT 0,
    y               REAL    NOT NULL DEFAULT 0,
    comment         TEXT    NOT NULL DEFAULT '',
    created_at      TEXT    NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    FOREIGN KEY (diagram_id) REFERENCES diagram(id) ON DELETE CASCADE,
    UNIQUE(diagram_id, name)
);
CREATE INDEX IF NOT EXISTS idx_table_diagram_id ON `table`(diagram_id);

-- ============================================================
-- 5. field — 字段（fields 领域）
-- ============================================================
CREATE TABLE IF NOT EXISTS field (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    table_id        INTEGER NOT NULL,
    name            TEXT    NOT NULL,
    type            TEXT    NOT NULL,
    size            INTEGER,
    default_value   TEXT    NOT NULL DEFAULT '',
    check_expr      TEXT    NOT NULL DEFAULT '',
    is_primary      INTEGER NOT NULL DEFAULT 0 CHECK(is_primary IN (0, 1)),
    is_unique       INTEGER NOT NULL DEFAULT 0 CHECK(is_unique IN (0, 1)),
    not_null        INTEGER NOT NULL DEFAULT 0 CHECK(not_null IN (0, 1)),
    increment       INTEGER NOT NULL DEFAULT 0 CHECK(increment IN (0, 1)),
    comment         TEXT    NOT NULL DEFAULT '',
    sort_order      INTEGER NOT NULL DEFAULT 0,
    FOREIGN KEY (table_id) REFERENCES `table`(id) ON DELETE CASCADE,
    UNIQUE(table_id, name)
);
CREATE INDEX IF NOT EXISTS idx_field_table_id ON field(table_id);
CREATE INDEX IF NOT EXISTS idx_field_table_sort ON field(table_id, sort_order);

-- ============================================================
-- 6. table_link — 表关联（多对多）
-- ============================================================
CREATE TABLE IF NOT EXISTS table_link (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    source_id       INTEGER NOT NULL,
    target_id       INTEGER NOT NULL,
    FOREIGN KEY (source_id) REFERENCES `table`(id) ON DELETE CASCADE,
    FOREIGN KEY (target_id) REFERENCES `table`(id) ON DELETE CASCADE
);
CREATE INDEX IF NOT EXISTS idx_table_link_source ON table_link(source_id);
CREATE INDEX IF NOT EXISTS idx_table_link_target ON table_link(target_id);

-- ============================================================
-- 7. indice — 索引（indices 领域；V1 后端实体化但 frontend 不写）
-- ============================================================
CREATE TABLE IF NOT EXISTS indice (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    table_id        INTEGER NOT NULL,
    name            TEXT    NOT NULL,
    is_unique       INTEGER NOT NULL DEFAULT 0 CHECK(is_unique IN (0, 1)),
    index_type      TEXT    NOT NULL DEFAULT 'BTREE' CHECK(index_type IN ('BTREE', 'HASH', 'FULLTEXT', 'SPATIAL', '')),
    FOREIGN KEY (table_id) REFERENCES `table`(id) ON DELETE CASCADE,
    UNIQUE(table_id, name)
);
CREATE INDEX IF NOT EXISTS idx_indice_table_id ON indice(table_id);

-- ============================================================
-- 8. indice_link — 索引字段关联
-- ============================================================
CREATE TABLE IF NOT EXISTS indice_link (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    indice_id       INTEGER NOT NULL,
    field_id        INTEGER NOT NULL,
    sort_order      INTEGER NOT NULL DEFAULT 0,
    FOREIGN KEY (indice_id) REFERENCES indice(id) ON DELETE CASCADE,
    FOREIGN KEY (field_id) REFERENCES field(id) ON DELETE CASCADE
);
CREATE INDEX IF NOT EXISTS idx_indice_link_indice ON indice_link(indice_id);
CREATE INDEX IF NOT EXISTS idx_indice_link_field ON indice_link(field_id);

-- ============================================================
-- 9. reference — 关系（references 领域）
-- ============================================================
CREATE TABLE IF NOT EXISTS reference (
    id                  INTEGER PRIMARY KEY AUTOINCREMENT,
    diagram_id          TEXT    NOT NULL,
    name                TEXT    NOT NULL DEFAULT '',
    start_table_id      INTEGER NOT NULL,
    start_field_id      INTEGER NOT NULL,
    end_table_id        INTEGER NOT NULL,
    end_field_id        INTEGER NOT NULL,
    cardinality         TEXT    NOT NULL CHECK(cardinality IN ('one_to_one', 'one_to_many', 'many_to_one', 'many_to_many')),
    on_update           TEXT    NOT NULL DEFAULT 'NO ACTION' CHECK(on_update IN ('CASCADE', 'RESTRICT', 'SET NULL', 'NO ACTION', 'SET DEFAULT')),
    on_delete           TEXT    NOT NULL DEFAULT 'NO ACTION' CHECK(on_delete IN ('CASCADE', 'RESTRICT', 'SET NULL', 'NO ACTION', 'SET DEFAULT')),
    FOREIGN KEY (diagram_id) REFERENCES diagram(id) ON DELETE CASCADE,
    FOREIGN KEY (start_table_id) REFERENCES `table`(id) ON DELETE CASCADE,
    FOREIGN KEY (end_table_id) REFERENCES `table`(id) ON DELETE CASCADE,
    FOREIGN KEY (start_field_id) REFERENCES field(id) ON DELETE CASCADE,
    FOREIGN KEY (end_field_id) REFERENCES field(id) ON DELETE CASCADE
);
CREATE INDEX IF NOT EXISTS idx_reference_diagram ON reference(diagram_id);
CREATE INDEX IF NOT EXISTS idx_reference_start_table ON reference(start_table_id);
CREATE INDEX IF NOT EXISTS idx_reference_end_table ON reference(end_table_id);

-- ============================================================
-- 10. area — 区域（areas 领域）
-- ============================================================
CREATE TABLE IF NOT EXISTS area (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    diagram_id      TEXT    NOT NULL,
    name            TEXT    NOT NULL,
    x               REAL    NOT NULL DEFAULT 0,
    y               REAL    NOT NULL DEFAULT 0,
    width           REAL    NOT NULL DEFAULT 200,
    height          REAL    NOT NULL DEFAULT 200,
    color           TEXT    NOT NULL DEFAULT '#e0f2fe',
    FOREIGN KEY (diagram_id) REFERENCES diagram(id) ON DELETE CASCADE
);
CREATE INDEX IF NOT EXISTS idx_area_diagram_id ON area(diagram_id);

-- ============================================================
-- 11. note — 便签（notes 领域）
-- ============================================================
CREATE TABLE IF NOT EXISTS note (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    diagram_id      TEXT    NOT NULL,
    x               REAL    NOT NULL DEFAULT 0,
    y               REAL    NOT NULL DEFAULT 0,
    content         TEXT    NOT NULL DEFAULT '',
    color           TEXT    NOT NULL DEFAULT '#fef3c7',
    FOREIGN KEY (diagram_id) REFERENCES diagram(id) ON DELETE CASCADE
);
CREATE INDEX IF NOT EXISTS idx_note_diagram_id ON note(diagram_id);

-- ============================================================
-- 初始化配置（bridge config singleton）
-- ============================================================
-- 实际存储在 `entity` 表 / 或独立 config 表；本 DDL 仅涵盖 11 张业务表。
-- bridge config 在 V1 实际用 backend 的 config.toml；SQL 端不再单建表。
