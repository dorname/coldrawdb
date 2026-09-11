# Delta: test/core-PC-import-export-test-cases.md

> 提案：feat-db-connect-import-and-ddl-realdb-verify

## MODIFIED — Phase C 导入/导出抽屉测试用例（用例总表）

主表追加以下行：

```markdown
| UT-PC-11 | 临时真实 SQLite 库（2 表 + PK/FK） | `POST /bridge/import/connect` `{engine:"sqlite", source:<路径>}` | 200；`tables=2`；`ddl` 含两表 CREATE TABLE、列级 PK/NOT NULL 与表级 FOREIGN KEY |
| UT-PC-12 | introspection IR fixtures | `render_introspect_ddl(engine, ir)` + 端点错误映射 | SQLite/PG 方言渲染正确（联合主键表级、FK 表级、标识符引号）；engine 非法 → 400；连接失败 → 502；0 表 → 200 `tables=0` 且 `ddl=""` |
| UT-PC-13 | 嵌入式 PG 实例（建 2 表含联合主键 + FK） | `POST /bridge/import/connect` `{engine:"postgres", source:<连接串>}` | 200；`tables=2`；`ddl` 含 PG 方言类型、表级 PRIMARY KEY(a,b)、FOREIGN KEY |
| UT-PC-14 | store 含 2 表 + 1 关系（含参数化类型/DEFAULT） | `export_diagram_sql(store, postgres/sqlite)` 输出在真实库执行 | 嵌入式 PG / SQLite 执行全部语句成功；建表数、外键数与模型一致 |
| UT-PC-15 | — | `include_str!` 锚点 | `editor_panels.rs` 含 `import-db-engine` / `import-db-source` / `import-db-connect` 且 ImportDrawer 含 `database` Tab 分支与 connect 接线 |
| ST-PC-04 | 编辑器已加载（已有 2 表），mock `import/connect` 返回 1 表 DDL | 导入抽屉 → 数据库 Tab → 填连接信息 → 连接并解析 → 导入到画布 | 解析摘要「已连接 · 检测到 1 张表」；提交走本地合并（PUT 表数 2→3、Toast「已导入」、不跳转、不调 `bridge/import/local`）；连接信息不出现在持久化存储 |
```

## ADDED — UT-PC-11 — SQLite introspection 端点测试

- **位置**：`backend/src/bridge_introspect.rs`（或 `phase3_bridge.rs` tests mod）
- **步骤与预期**：
  1. 在唯一临时目录用 sqlx-sqlite 建真实库：`users(id INTEGER PRIMARY KEY, name TEXT NOT NULL)` + `posts(id INTEGER PRIMARY KEY, user_id INTEGER REFERENCES users(id), title TEXT)`
  2. 调端点（或直连 `introspect_sqlite(path)`）→ `tables=2`；`ddl` 含 `CREATE TABLE users` / `CREATE TABLE posts`、`NOT NULL`、`FOREIGN KEY (user_id) REFERENCES users(id)`
  3. 测试结束删除临时库（遵循 UT-PU-20 不污染工作区约束）

## ADDED — UT-PC-12 — introspection DDL 渲染与错误映射纯函数测试

- **位置**：`backend/src/bridge_introspect.rs`
- **步骤与预期**：
  1. fixture IR（含联合主键表、FK 表、保留字列名）→ SQLite/PG 两路渲染：联合主键 → 表级 `PRIMARY KEY(a, b)`；FK → 表级 `FOREIGN KEY (c) REFERENCES t(c2)`；保留字/大小写混合标识符加双引号
  2. `engine="mysql"` 或空 `source` → 400；不存在路径/不可达连接串 → 502；空库 → 200 `tables=0` + `ddl=""`

## ADDED — UT-PC-13 — PG introspection 嵌入式真实库测试

- **位置**：`backend/src/bridge_introspect.rs`（dev-dependency `postgresql_embedded`）
- **步骤与预期**：
  1. 启动嵌入式 PG；sqlx-postgres 建 `orders(a INT, b INT, PRIMARY KEY(a,b))` + `items(id SERIAL PRIMARY KEY, order_a INT, order_b INT, FOREIGN KEY (order_a, order_b) REFERENCES orders(a,b))`
  2. 调 introspect → `tables=2`；`ddl` 不含 `pg_catalog`/`information_schema` 系统表；联合主键与复合 FK 渲染表级约束
  3. 首次运行允许二进制下载耗时（超时阈值放宽至 120s）；测试结束停止实例

## ADDED — UT-PC-14 — 导出 DDL 真实库执行验证

- **位置**：`frontend-rs/tests/ddl_realdb_verify.rs`（dev-dependencies：`sqlx` sqlite feature + `postgresql_embedded`）
- **步骤与预期**：
  1. 构造 store：2 表（含 `VARCHAR(32)` 参数化类型、`DEFAULT`、自增主键）+ 1 条关系
  2. `export_diagram_sql(store, "postgres")` → 嵌入式 PG 逐句执行 → 成功；`information_schema.tables` 计数=2，FK 约束计数=1
  3. `export_diagram_sql(store, "sqlite")` → sqlx-sqlite 临时库执行 → 成功；`sqlite_master` 计数=2
  4. 任一执行失败视为导出器缺陷，本变更内修复生成逻辑（不接受 skip）

## ADDED — UT-PC-15 — ImportDrawer 数据库来源锚点测试

- **位置**：`frontend-rs/src/editor_panels.rs`（tests mod）
- **断言**（`include_str!` 锚点口径）：
  1. 存在 `data-testid="import-db-engine"` / `import-db-source` / `import-db-connect"`
  2. `ImportFormat` 含 `Database` 变体且 format tabs 渲染第 4 个 Tab
  3. 「连接并解析」接线到 `import_from_connection` 客户端方法（`POST /api/v1/bridge/import/connect`）

## ADDED — ST-PC-04 — 连接数据库导入 e2e

- **位置**：`frontend-rs/scripts/test-spec-parity-d.mjs`（installApi 新增 `POST /api/v1/bridge/import/connect` mock：返回 1 表 DDL）
- **步骤与预期**：
  1. 进编辑器建 2 表 → 更多菜单 → 导入 → 切「数据库」Tab
  2. 引擎选 SQLite、连接信息填 `/tmp/demo.db` → 点「连接并解析」→ 摘要显示「已连接 · 检测到 1 张表」
  3. 点「导入到画布」→ PUT 落账表数 2→3；Toast「已导入」；URL 不变；无 `POST /bridge/import/local`
  4. localStorage 中不出现连接信息字符串

## MODIFIED — 变更记录

追加行：

```markdown
| UT-PC-11～13 | ADDED（feat-db-connect-import-and-ddl-realdb-verify） | 连接数据库导入：后端 introspection 端点（SQLite 真实库 / 渲染纯函数 / 嵌入式 PG） |
| UT-PC-14 | ADDED（feat-db-connect-import-and-ddl-realdb-verify） | 导出 DDL 真实库执行验证（嵌入式 PG + SQLite） |
| UT-PC-15 | ADDED（feat-db-connect-import-and-ddl-realdb-verify） | ImportDrawer 数据库来源 UI 锚点 |
| ST-PC-04 | ADDED（feat-db-connect-import-and-ddl-realdb-verify） | 连接数据库导入端到端（mock 端点 + 本地合并管道复用） |
```
