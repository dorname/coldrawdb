## MODIFIED — 4.1 读调用（追加摘要元数据规则）

- `list_diagrams` 默认最多返回 100 条摘要；adapter 负责稳定排序和本地 `limit` 截断。
- `GET /diagrams/queryAll` 必须读取并返回每张图的真实 `revision` 与 `updated_at`，同时保留 `lastModified` 兼容字段和 `is_deleted=0` 过滤。旧数据 `updated_at` 缺失时可回退到 `lastModified`；不得用 `0/null` 覆盖已有值，也不得用当前时间伪造修改时间。
- MCP `list_diagrams` 的每个摘要项必须包含 `revision`、`updated_at` 键并透传上述真实值；现有名称过滤、排序和 limit 行为不变。本变更不增加 table/reference 计数。
