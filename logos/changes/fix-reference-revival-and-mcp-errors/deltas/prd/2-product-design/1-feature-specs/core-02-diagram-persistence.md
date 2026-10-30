## MODIFIED — 3. 持久化事务策略

- **创建 / 更新 / 删除**：单事务内写入 `diagram` + 所有关联（`field` / `reference` / `area` / `note` / `table_link` 等）。
- **级联**：删除 diagram → 级联删除所有 fields / references / areas / notes / table_link / diagram_link。
- **软删除关系重建**：全量保存插入 `reference` 前，允许在同一事务中物理清理同 ID 且 `is_deleted=1` 的遗留行，再写入新关系，避免已删除关系占用全局主键而阻断正常编辑。不得删除或覆盖 `is_deleted=0` 的活跃行；活跃 ID 冲突仍按数据库错误返回。该行为只处理当前写入的关系 ID，不批量迁移或清理其他历史数据。
- **隔离级别**：SQLite 默认（SERIALIZABLE）；backend 显式 `BEGIN IMMEDIATE TRANSACTION` 防止并发写入。
- **错误处理**：事务回滚后返回 400 + 详细错误。
