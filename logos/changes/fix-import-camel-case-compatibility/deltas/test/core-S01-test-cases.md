## ADDED — Issue #54 导入命名兼容用例

### UT-S01-17 — camelCase 端点与 notNull 规范化

- **位置**：`backend/src/diagram_persistence.rs`。
- **输入**：table/field 使用固定 ID，reference 仅提供 `startTableId/endTableId/startFieldId/endFieldId`，字段提供 `notNull:true`。
- **操作**：执行 `normalize_import_payload` 和导入 DTO 构造。
- **断言**：关系未被丢弃；四个 snake_case 端点已补写并等于重映射后的 table/field ID；DTO `not_null=true`；snake_case 优先规则不被别名覆盖。

### ST-S01-07 — 两种命名风格导入后关系与约束一致

- **位置**：`backend/src/diagrams_v1.rs::tests`，独立临时 SQLite。
- **步骤**：分别用 camelCase 与 snake_case 导入同一组五表、五字段、四关系的 CMS payload，字段显式使用对应风格的非空约束；GET 两张图。
- **断言**：两次均 200 且实际持久化 5 表、5 字段、4 关系；关系端点均指向各自新图；目标字段 `not_null=true`；camelCase 响应无 `missing endpoint id` warnings；snake_case 不再 500。
- **reporter**：登记 `UT-S01-17`、`ST-S01-07` 到 `logos/resources/verify/test-results.jsonl`。

### 附录 A：用例 ID 清单追加

| ID | 标题 | 对齐实现 |
|---|---|---|
| UT-S01-17 | camelCase 端点与 notNull 规范化 | `backend/src/diagram_persistence.rs` |
| ST-S01-07 | 两种命名风格导入后关系与约束一致 | `backend/src/diagrams_v1.rs::tests` |
