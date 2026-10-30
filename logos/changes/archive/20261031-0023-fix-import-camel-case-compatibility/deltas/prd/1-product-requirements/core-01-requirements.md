## MODIFIED — FR-12

| FR-12 | JSON 导入/导出 | 通过 `POST /api/v1/diagrams/import`；同一 payload 重复导入创建实体 ID 隔离的独立新图并保留关系端点；关系端点同时接受 camelCase（`startTableId` 等）与 snake_case（`start_table_id` 等），字段非空同时接受 `notNull` / `not_null` 并正确持久化；嵌套类型错误返回可诊断 400，持久化失败返回非空 details，不得静默变成空图成功 | CAP-BRIDGE-04 / CAP-PERSIST-04 |
