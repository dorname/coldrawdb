# Delta: test/core-PC-import-export-test-cases.md

> 提案：feat-room-recycle-bin-dropdown-and-db-export

## ADDED — 用例（导出到数据库 + 下拉样式）

### UT-PC-16 — 导出执行：SQLite 真实库

- **位置**：`backend/src/phase3_bridge.rs` 或 `bridge_introspect.rs` 测试模块（`ut_pc_16_export_execute_sqlite`）
- **步骤**：临时目录新建 SQLite 文件路径 → `POST /api/v1/bridge/export/execute`（engine=sqlite，ddl 为 2 表含 FK 的 SQLite 方言 DDL）
- **断言**：200；`data.statements` = 实际执行条数；`data.tables` = 2；用 sqlx 直查该库 `sqlite_master` 含 2 张表且 FK 子句存在
- **附加**：对同一路径重复执行同一 DDL（无 `IF NOT EXISTS`）→ 502 且 message 含引擎错误（表已存在）

### UT-PC-17 — 导出执行：嵌入式 PG + 错误映射

- **位置**：同上（`ut_pc_17_export_execute_pg`，dev-dependency `postgresql_embedded`，复用 UT-PC-13 的实例夹具）
- **断言**：
  1. PG 方言 DDL（含表级 `PRIMARY KEY(a,b)` 与 `FOREIGN KEY`）执行 200，`tables` 与模型一致；直查 `information_schema.tables` 存在
  2. engine=`mysql` → 400；`source` 空或 `ddl` 空 → 400
  3. 语法错误 DDL → 502 且 message 含 PG 引擎错误片段

### UT-PC-18 — 前端锚点：ExportDrawer 导出到数据库

- **位置**：`frontend-rs/src/editor_panels.rs`
- **断言**（`include_str!` 锚点口径）：
  1. 存在 `data-testid="export-db-engine"` / `export-db-source` / `export-db-execute` / `export-db-result`
  2. ExportDrawer 含导出执行接线（调 `POST /api/v1/bridge/export/execute` 的 client 方法，请求体含 `engine`/`source`/`ddl`）
  3. 空连接信息时内联提示「请填写连接信息」且不发请求（守卫分支存在）
  4. 连接信息不写入 localStorage（无对应 setItem 调用）

### UT-PC-19 — 样式锚点：表单下拉视觉条款

- **位置**：`frontend-rs/src/styles.css`
- **断言**（`include_str!` 锚点口径）：`.cdb-form-select` 规则块含
  1. `appearance`（去原生外观）
  2. chevron `background-image`（自定义箭头）
  3. `padding-right`（为箭头预留空间）
  4. `color-scheme`（暗色弹层适配）
  5. `option` 配色规则存在

### ST-PC-05 — e2e：导出到数据库

- **前置**：编辑器已加载（2 表），mock `export/execute` 返回 `{ statements: 2, tables: 2 }`
- **步骤**：导出抽屉 → SQL Tab → 填连接信息 → 「在数据库中执行」
- **预期**：请求体 `ddl` 与预览区 SQL 一致；`export-db-result` 显示「已执行 2 条语句 · 建表 2 张」；Toast「已导出到数据库」；连接信息不出现在持久化存储

## MODIFIED — 变更记录

在变更记录追加一行：

| feat-room-recycle-bin-dropdown-and-db-export | 新增 UT-PC-16~19 / ST-PC-05（导出到数据库执行 + 表单下拉样式条款） |
