## ADDED — Issue #57 软删除关系重建用例

### UT-S01-16 — 同 ID 软删除 reference 可在全量保存中重建

- **位置**：`backend/src/diagram_persistence.rs` 单元测试，独立临时 SQLite。
- **前置**：图内已有两张表及字段；数据库另有一条同 ID、`is_deleted=1` 且无有效 `diagram_link` 的遗留 reference。
- **操作**：以最新 revision 全量保存含同 ID、更新后端点/属性的 reference，再 GET 加载。
- **断言**：保存成功且 revision +1；同 ID 仅一行，`is_deleted=0`，加载后的关系端点和属性等于新快照；事务不修改其他活跃关系。
- **reporter**：登记 `UT-S01-16` 到 `logos/resources/verify/test-results.jsonl`。

### 附录 A：用例 ID 清单追加

| ID | 标题 | 对齐实现 |
|---|---|---|
| UT-S01-16 | 同 ID 软删除 reference 可在全量保存中重建 | `backend/src/diagram_persistence.rs` |
