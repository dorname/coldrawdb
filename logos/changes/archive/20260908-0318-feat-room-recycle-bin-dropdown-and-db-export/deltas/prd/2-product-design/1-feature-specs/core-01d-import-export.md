# Delta: prd/2-product-design/1-feature-specs/core-01d-import-export.md

> 提案：feat-room-recycle-bin-dropdown-and-db-export

## ADDED — 5.4 导出到数据库（连接执行）

ExportDrawer SQL Tab 在「复制 / 下载」行下方新增「导出到数据库」区块（仅 SQL Tab 显示；DBML/JSON 不支持连接执行）：

```
┌─ 导出到数据库 ──────────────────────────┐
│ 引擎: [ sqlite ▼ ]                      │  ← export-db-engine（sqlite / postgres）
│ 连接信息: [_______________________]      │  ← export-db-source（SQLite 文件绝对路径 / PG 连接串）
│ [ 在数据库中执行 ]                       │  ← export-db-execute
│ 执行结果反馈区                           │  ← export-db-result
└──────────────────────────────────────────┘
```

- testid：`export-db-engine` / `export-db-source` / `export-db-execute` / `export-db-result`。
- 行为：
  - 点击「在数据库中执行」→ 取**当前预览区已生成的 SQL**（`export_diagram_sql` 输出）作为 `ddl`，调 `POST /api/v1/bridge/export/execute`（`{ engine, source, ddl }`，Bearer 鉴权）。
  - 空连接信息 → 内联错误「请填写连接信息」，不发请求；预览为空（空 diagram）→ 按钮 disabled。
  - 成功（200）→ `export-db-result` 显示「已执行 N 条语句 · 建表 M 张」+ Toast「已导出到数据库」。
  - 失败映射：400 →「不支持的数据库引擎」/「请填写连接信息」；502 →「导出执行失败：<引擎错误消息>」（直接展示后端 message，便于定位语法/连接问题）。
- 安全口径：连接信息仅当次请求使用，**不写入任何持久化存储**（localStorage / import log 均不涉及）；与 §13 连接导入同一边界。
- 引擎下拉与连接执行不联动切换预览引擎：预览引擎（`export-engine-select`）与目标库引擎（`export-db-engine`）独立；用户须自行保证二者匹配（UI 文案提示「请确保导出引擎与目标库一致」）。

## MODIFIED — 8. 测试 ID 索引

| TC ID | 描述 |
|-------|------|
| UT-PC-01 | `parse_sql_statements` 驱动导入摘要 |
| UT-PC-02 | `export_diagram_sql` 非空 diagram 输出含 `CREATE TABLE` |
| UT-PC-03 | `export_diagram_dbml` 含 `Table` 块 |
| UT-PC-04 | `IoDrawerKind` 互斥：开 Import 折叠 Inspector |
| UT-PC-05 | `count_dbml_tables` 纯函数 |
| ST-PC-01 | e2e：btn-import → 粘贴 SQL → 解析摘要可见 |
| UT-PC-11 | 后端 SQLite introspection → 方言 DDL（真实临时库） |
| UT-PC-12 | 后端 introspection DDL 渲染纯函数 + 错误映射（400/502/tables=0） |
| UT-PC-13 | 后端 PG introspection（嵌入式 PG） |
| UT-PC-14 | 导出 DDL 真实库执行验证（嵌入式 PG + SQLite） |
| UT-PC-15 | 前端锚点：ImportDrawer 数据库来源 testid 与接线 |
| ST-PC-04 | e2e：数据库来源连接导入 → 本地合并落账 |
| UT-PC-16 | 后端导出执行：SQLite 真实库执行 DDL（建表数/语句数） |
| UT-PC-17 | 后端导出执行：嵌入式 PG + 错误映射（400/502） |
| UT-PC-18 | 前端锚点：ExportDrawer 导出到数据库 testid 与接线 |
| UT-PC-19 | 样式锚点：`.cdb-form-select` / `.cdb-select` 视觉条款落地 |
| ST-PC-05 | e2e：导出到数据库 → mock 执行成功反馈 |

详细步骤见 `core-PC-import-export-test-cases.md`。
