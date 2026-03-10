# Phase 1 Schema 变更说明

本文件对应 `backend/migrations/0001_phase1_schema.up.sql` 与 `0001_phase1_schema.down.sql`。

## 1. 变更摘要

- 新增迁移记录表：`schema_migrations`
- `diagram`：新增 `updated_at`、`revision`、`is_deleted`
- 核心实体新增软删除：
  - `table.is_deleted`
  - `field.is_deleted`
  - `indice.is_deleted`
  - `reference.is_deleted`
  - `note.is_deleted`
  - `area.is_deleted`
- `diagram_link.reference` 重命名为 `diagram_link.reference_id`
- 链接表新增顺序字段：
  - `table_link.order_no`
  - `indice_link.order_no`
- 新增索引：
  - `idx_diagram_link_diagram_id`
  - `idx_table_link_table_id`
  - `idx_indice_link_indice_id`
- 数据回填：`diagram.updated_at` 从 `last_modified` 或当前时间回填

## 2. 向后兼容说明

- 迁移采用幂等版本跟踪（`schema_migrations`），重复执行不会重复应用。
- `down` 脚本为 SQLite best-effort：清理索引和迁移记录，不做 destructive column rollback。

## 3. 风险提示

- 由于 SQLite 列回退限制，若需完全回滚列变更，建议通过库文件备份恢复。
