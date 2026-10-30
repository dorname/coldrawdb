## ADDED — UT-S04-20 queryAll 返回真实图表元数据

- **位置**：`backend/src/diagrams/mod.rs::tests`，独立临时 SQLite。
- **前置**：迁移后的数据库含活跃图与软删除图；活跃图 `revision=8`、`updated_at='2026-10-29T12:34:56Z'`，`lastModified` 使用不同旧值以区分字段来源。
- **操作**：调用 `list_active_diagrams` 并序列化结果。
- **断言**：活跃图摘要 `revision=8`、`updated_at` 等于数据库真实值，原有 `lastModified` 保持兼容；软删除图仍不出现；不使用默认 0/null 覆盖真实元数据。
- **reporter**：登记 `UT-S04-20` 到 `logos/resources/verify/test-results.jsonl`。

### 用例登记追加

| ID | 前置 | 操作 | 预期 |
|---|---|---|---|
| UT-S04-20 | 活跃图含 revision=8 与真实 updated_at，另有软删除图 | 调用 queryAll 列表查询 | 返回真实 revision/updated_at 和兼容 lastModified；软删除图被过滤 |
