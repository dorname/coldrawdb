# Delta: core-01d-import-export.md

> 提案：feat-db-connect-import-and-ddl-realdb-verify

## MODIFIED — 4.1 布局

替换为（格式 Tab 新增「数据库」来源）：

````markdown
### 4.1 布局

```
┌─ 导入 ───────────────────────────── [×] ─┐
│ [ SQL ] [ DBML ] [ JSON ] [ 数据库 ]      │  ← format tabs（数据库 = 连接导入）
│ 数据库引擎: [ generic ▼ ]  （SQL 时显示）  │
│ ┌─────────────────────────────────────┐ │
│ │ 粘贴 SQL / 拖放 .sql 文件            │ │  ← import-textarea
│ └─────────────────────────────────────┘ │
│ 解析摘要: 3 条语句 · 预计 2 张表          │  ← import-parse-summary
│ [ 取消 ]              [ 导入到画布 ▶ ]   │
└──────────────────────────────────────────┘

数据库 Tab：
┌──────────────────────────────────────────┐
│ 引擎: [ SQLite ▼ ]                        │  ← import-db-engine（sqlite/postgres）
│ 连接: [ /path/to/db.sqlite 或 PG 连接串 ] │  ← import-db-source
│ [ 连接并解析 ]                            │  ← import-db-connect
│ 解析摘要: 已连接 · 检测到 N 张表          │  ← 复用 import-parse-summary
│ [ 取消 ]              [ 导入到画布 ▶ ]   │
└──────────────────────────────────────────┘
```

- 宽度：**400px**（与 Inspector 默认 320px 区分，可挤压画布）
- testid：`import-drawer` / `import-format-tabs` / `import-engine-select` / `import-textarea` / `import-parse-summary` / `import-submit` / `import-cancel`
- 数据库来源新增 testid：`import-db-engine` / `import-db-source` / `import-db-connect`
````

## MODIFIED — 4.2 格式 Tab

替换为（新增 database 行）：

```markdown
### 4.2 格式 Tab

| format | 引擎选择 | 解析预览 |
|--------|----------|----------|
| `sql` | 必填（默认 `generic`） | `parse_sql_statements` 语句数 + 简单表名启发式（可选） |
| `dbml` | 隐藏 | 行数 / `Table` 块计数（纯函数 `count_dbml_tables`） |
| `json` | 隐藏 | `serde_json` 校验 + tables 数组长度 |
| `database` | 必填（`sqlite` / `postgres`，默认 `sqlite`） | 「连接并解析」→ `POST /api/v1/bridge/import/connect` 返回 DDL → 复用 SQL 解析摘要（见 §4.4 第 3.5 条） |
```

## MODIFIED — 4.4 提交行为

在第 3 条「本地合并进当前画布」之后追加 3.5 条（其余条目不变）：

```markdown
3.5. **数据库连接导入（feat-db-connect-import-and-ddl-realdb-verify）**——`database` Tab 提交流程：
   - 「连接并解析」：校验引擎与连接信息非空 → `POST /api/v1/bridge/import/connect`（`{ engine, source }`）→ 200 取 `data.ddl` 经 `parse_sql_import_tables` 解析 → 解析摘要显示「已连接 · 检测到 N 张表」；`tables=0` → inline 错误「未检测到数据表」
   - 端点错误文案：400 → 「不支持的数据库引擎」/「请填写连接信息」；502 → 「连接数据库失败，请检查连接信息」（inline 错误，不关抽屉）
   - 解析成功后「导入到画布」提交：与第 3 条**完全相同的本地合并管道**（`merge_import_into_store` 避让合并 → schedule_save PUT 落账 → Toast「已导入」→ 关闭抽屉），不调 `bridge/import/local`、不跳转页面
   - 连接信息（文件路径/连接串）仅用于当次请求，不写入 localStorage / 导入日志
   - 只读（Viewer）下 ImportDrawer 整体不可达（入口已禁用），数据库 Tab 不单独授权
```

## ADDED — 4.6 DDL 真实库验证口径

> 提案：feat-db-connect-import-and-ddl-realdb-verify

导出 DDL 的正确性验证从「文本断言」升级为「真实库执行」：

- 验证范围：本次承诺 **PostgreSQL + SQLite** 两路（MySQL/Oracle 等其余引擎仍文本断言，不在本变更）
- 验证方式：frontend-rs 集成测试（native target）调 `export_diagram_sql(store, engine)` 生成 DDL → 在**真实数据库实例**执行全部语句 → 断言执行成功且 `information_schema.tables` / `sqlite_master` 中建表数量、外键数量与模型一致
- PG 实例：`postgresql_embedded`（dev-dependency，运行时从 Maven Central 拉取二进制，首次运行有下载耗时）
- SQLite 实例：sqlx-sqlite 临时文件（测试结束清理）
- 若生成 DDL 在真实库执行失败，视为导出器缺陷，须在本变更内修复生成逻辑（不接受跳过断言）

## MODIFIED — 8. 测试 ID 索引

替换为（追加 6 行）：

```markdown
## 8. 测试 ID 索引

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

详细步骤见 `core-PC-import-export-test-cases.md`。
```
