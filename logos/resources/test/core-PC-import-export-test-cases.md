# core-PC-import-export-test-cases.md

> 模块：core | 提案：redesign-phase-c-import-export, align-v1-api-completion
> 路径：`logos/resources/test/core-PC-import-export-test-cases.md`
> 最后更新：2026-09-08

## Phase C 导入/导出抽屉测试用例

| TC ID | Given | When | Then |
|-------|-------|------|------|
| UT-PC-01 | SQL 含 2 条 CREATE | `parse_sql_statements` | `Ok(vec![..])` len==2；摘要显示「2 条语句」 |
| UT-PC-02 | store 含表（含 UNIQUE/自增/DEFAULT/COMMENT 字段）+ 关系 | `export_diagram_sql(store, engine)` | 输出含 `CREATE TABLE` 与表名；列含 `UNIQUE`/`DEFAULT`/自增（generic/mysql → `AUTO_INCREMENT`，pg 由 SERIAL 承担不追加）/COMMENT（mysql 行内，其余 `COMMENT ON COLUMN`）；含 `FOREIGN KEY (c) REFERENCES t(c2)` |
| UT-PC-03 | store 含表+关系 | `export_diagram_dbml(store)` | 输出含 `Table` 与 `ref:` |
| UT-PC-04 | Inspector 展开 | `open_import_drawer()` | `inspector_open==false` 且 `io_drawer==Import` |
| UT-PC-05 | DBML 含 2 个 Table 块 | `count_dbml_tables(text)` | 返回 2 |
| UT-PC-06 | 零表画布 | 点击 `guide-import-sql` | `import-drawer` 可见 |
| UT-PC-07 | DDL 文本（多表，含参数化类型/列级约束/DEFAULT/COMMENT/表级联合主键） | `parse_sql_import_tables(content)` | 表名/列名/类型整串（`VARCHAR(32)` 保留）正确；列级 PK/NOT NULL/UNIQUE/AUTO_INCREMENT（SERIAL→自增）/DEFAULT/COMMENT 落字段；**PG `COMMENT ON TABLE t IS '...'` 回填表 comment、`COMMENT ON COLUMN t.c IS '...'` 回填列 comment；目标不存在则跳过不报错**；表级 `PRIMARY KEY(a,b)` 双列置位；引号/方括号规范化；非 CREATE TABLE 语句跳过；无合法语句报错 |
| UT-PC-08 | DDL 含表级 `FOREIGN KEY (c) REFERENCES t(c2)` 与列级 `REFERENCES t(c2)` | `parse_sql_import_tables(content)` | 生成 references 连线（端点解析到正确表/字段 id；跨已存在表可解析）；无 REFERENCES 时 references 为空。合并后（fix-dbimport-save-and-pg-schema）端点与对应表/字段**重键后** ID 一致 |
| ST-PC-01 | 编辑器已加载（已有 2 表） | 更多菜单 → 导入 → 粘贴含 2 张表的 DDL → 提交 | 解析摘要可见；**合并进当前画布**（表数 2→4，新增表避让现有布局）；落账 PUT 含 4 表；Toast「已导入」；无页面跳转（URL 不变）；不调用 `/bridge/import/local` |
| UT-PC-09 | store 已有表（含坐标）+ 导入解析结果（tables + references） | `merge_import_into_store(store, tables, references)` | 返回合并后的表/关系；新表 x/y 避让现有表包围盒（网格落位不重叠）；既有表坐标不变；references 全量追加。导入实体 ID 重键为全局唯一（fix-dbimport-save-and-pg-schema），端点一致且无 `import-t-` 残留 |
| ST-PC-02 | 已进入 ListView（有表） | 工具条点「导入」→ 粘贴含 1 张表的 DDL → 提交；再点「导出」 | `list-btn-import` 打开 ImportDrawer；提交走本地合并（PUT 表数 +1、Toast「已导入」、不跳转、不调 bridge）；关闭抽屉回到 ListView 且表树含新表；`list-btn-export` 打开 ExportDrawer 预览含当前模型 |
| ST-PC-03 | 编辑器已加载，视口 720×480 | 打开更多菜单 | `app-bar-overflow-menu` bounding box 完整落在视口内（right ≤ 720、bottom ≤ 480）；菜单可滚动到达末项「命令面板」 |
| UT-PC-10 | — | `include_str!` 锚点 | `editor_panels.rs` 含 `list-btn-import` / `list-btn-export` 且 IoDrawer 在 ListView 态可渲染；`styles.css` 溢出菜单含 `max-height` + `overflow-y` |
| UT-PC-11 | 临时真实 SQLite 库（2 表 + PK/FK） | `POST /bridge/import/connect` `{engine:"sqlite", source:<路径>}` | 200；`table_count=2`；`tables` 数组含两表（name / fields 含列级 PK/NOT_NULL 与 FK 端点字段）；**SQLite 无注释：所有 comment 为空串**（引擎限制，不报错） |
| UT-PC-12 | introspection IR fixtures | IR → 响应序列化 + 端点错误映射 | SQLite/PG 两路序列化正确（联合主键各列 primary=true、default 缺省空串、comment 透传）；engine 非法 → 400；连接失败 → 502；0 表 → 200 `table_count=0` 且 `tables=[]` |
| UT-PC-13 | 嵌入式 PG 实例（建 2 表含联合主键 + FK） | `POST /bridge/import/connect` `{engine:"postgres", source:<连接串>}` | 200；`table_count=2`；`tables` 含 PG 规范化类型（`character varying(n)` → `VARCHAR(n)`）、联合主键两列 primary=true、FK 表/列端点正确；不含 `pg_catalog`/`information_schema` 系统表 |
| UT-PC-14 | store 含 2 表 + 1 关系（含参数化类型/DEFAULT） | `export_diagram_sql(store, postgres/sqlite)` 输出在真实库执行 | 嵌入式 PG / SQLite 执行全部语句成功；建表数、外键数与模型一致 |
| UT-PC-15 | — | `include_str!` 锚点 | `editor_panels.rs` 含 `import-db-engine` / `import-db-source` / `import-db-connect` 且 ImportDrawer 含 `database` Tab 分支与 connect 接线 |
| ST-PC-04 | 编辑器已加载（已有 2 表），mock `import/connect` 返回 1 表结构化 tables | 导入抽屉 → 数据库 Tab → 填连接信息 → 连接并解析 → 导入到画布 | 解析摘要「已连接 · 检测到 1 张表」；提交走本地合并（PUT 表数 2→3、Toast「已导入」、不跳转、不调 `bridge/import/local`）；连接信息不出现在持久化存储 |
| UT-PC-16 | 临时 SQLite 文件路径 + 2 表含 FK 的 SQLite 方言 DDL | `POST /bridge/export/execute` `{engine:"sqlite", source, ddl}` | 200；`statements` 与实际执行条数一致；`tables=2`；直查 `sqlite_master` 含 2 表；重复执行同一 DDL → 502 含「表已存在」类引擎错误 |
| UT-PC-17 | 嵌入式 PG 实例 + PG 方言 DDL（联合主键 + FK） | `POST /bridge/export/execute` `{engine:"postgres", ...}`；另测非法 engine/空 source/空 ddl/语法错误 DDL | 200 且 `tables` 与模型一致，`information_schema` 可查；engine 非法或 source/ddl 空 → 400；语法错误 → 502 含 PG 错误片段 |
| UT-PC-18 | — | `include_str!` 锚点 | `editor_panels.rs` 含 `export-db-engine` / `export-db-source` / `export-db-execute` / `export-db-result` 且接线 `export/execute`；空连接信息内联提示不发请求；连接信息不入 localStorage |
| UT-PC-19 | — | `include_str!` 锚点 | `styles.css` `.cdb-form-select` 含 `appearance` / chevron `background-image` / `padding-right` / `color-scheme` / `option` 配色 |
| ST-PC-05 | 编辑器已加载（2 表），mock `export/execute` 返回 `{statements:2, tables:2}` | 导出抽屉 → SQL Tab → 填连接信息 → 在数据库中执行 | 请求体 `ddl` 与预览一致；`export-db-result` 显示「已执行 2 条语句 · 建表 2 张」；Toast「已导出到数据库」；连接信息不入持久化存储 |
| UT-PC-20 | 两组 `parse_sql_import_tables` 结果（确定性 ID） | 先后经 `merge_import_into_store` 合并 | 两次产出的表/字段/关系 ID 互不相同（无 `import-t-` 残留）；reference 四端点指向重键后新 ID 无悬空；语义内容不受影响 |
| UT-PC-21 | 嵌入式 PG：public 与 app 双 schema 各建同名 users（列不同）+ app.orders FK | `introspect` 缺省 / `schema=app` / 不存在 schema | 缺省仅 public.users（public 列集合）；`schema=app` 仅 app 两表且 FK 指向 app.users；不存在 schema → 0 表不报错 |
| UT-PC-22 | — | `include_str!` 锚点 | `editor_panels.rs` 含 `import-db-schema` 且仅 postgres 分支渲染；`import_from_connection` 请求体含 `schema`；sqlite 分支不发送 |
| ST-PC-06 | 编辑器已加载（2 表），mock `import/connect` + `PUT /diagrams/{id}` | 数据库 Tab 连续两次连接并解析 → 两次导入到画布 → 保存 | 表数为两次之和；两次保存请求体实体 ID 无交集；两次保存 200 且 saveText「已保存」；postgres 时 `import-db-schema` 可见且请求体带 `schema`，sqlite 无 `schema` 字段 |
| UT-PC-23 | — | `include_str!` 锚点 | `styles.css` 中命中 select 的复合选择器（`.cdb-create-room-modal .cdb-input` / `.cdb-auth-card .cdb-input`）不含 `background:` 简写（仅 `background-color`）；暗色 chevron 规则含 `background-repeat: no-repeat` |
| UT-PC-24 | 嵌入式 PG：建表带 `COMMENT ON TABLE` + `COMMENT ON COLUMN`（含中文与单引号转义） | `POST /bridge/import/connect` introspect | 200；`tables[].comment` = 表注释（obj_description）；对应 `fields[].comment` = 列注释（col_description）；无注释表/列返回空串 |
| UT-PC-25 | 临时真实 SQLite 库（2 表） | `POST /bridge/import/connect` | 200；响应 schema 完整（table_count == `tables.len()`；每表含 name/comment/fields；每字段含 name/type/primary/not_null/default/comment）；所有 comment 为空串 |
| UT-PC-26 | mock 结构化 `import/connect` 响应（1 表 2 字段含 comment） | 前端消费路径（`data.tables` → JSON 导入解析） | `parse_json_import_tables`（或轻适配层）输出 `Table.comment` / `Field.comment` 与响应一致；确定性 ID 形式不变（合并入口重键，见 UT-PC-20） |
| UT-PC-27 | store 含表（表 comment 非空、列 comment 非空） | `export_diagram_sql(store, engine)` | postgres/generic 输出含 `COMMENT ON TABLE t IS '...';`（位于该表 CREATE TABLE 之后）；mysql 输出含表选项 `COMMENT='...'`；空 comment 不输出；单引号转义为 `''` |
| UT-PC-28 | JSON 导入文本（tables 含表/字段 comment，字段缺省 comment 键） | `parse_json_import_tables(content)` | **加强断言**：表/字段 comment 原样透传；缺省键落空串；与 UT-PC-26 数据库导入路径产出一致 |
| UT-PC-29 | — | `include_str!` 锚点 | `editor_panels.rs` 含 `list-table-comment-` testid 且仅在 `t.comment` 非空分支渲染（无注释不占位）；`styles.css` 注释行含单行截断规则 |
| UT-PC-30 | — | `include_str!` 锚点 | `editor_panels.rs` 含 `inspector-table-comment`（受控输入 + blur 落账通路，仿 `on_rename_table`）与 `inspector-field-comment-`；`list-field-comment-` 既有锚点保留 |
| ST-PC-07 | store 含表（表/列 comment 非空） | 导出 PG SQL → 粘贴回 SQL 导入 Tab → 解析合并 | round-trip 后表/列 comment 与原始一致（COMMENT ON TABLE / COMMENT ON COLUMN 解析回填） |
| ST-PC-08 | 编辑器已加载（1 表无注释），mock `PUT /diagrams/{id}` | Inspector 填表注释 → blur → 保存；表树注释行可见；重新加载 | `inspector-table-comment` blur 后 PUT 请求体 `table.comment` 一致；表树出现 `list-table-comment-{id}`；重载后注释仍在 |

### UT-PC-07 — DDL 列级导入解析纯函数测试

- **位置**：`frontend-rs/src/editor_panels.rs`（`parse_sql_import_tables`）
- **步骤与预期**：
  1. 解析 `CREATE TABLE users (id UUID PRIMARY KEY, name VARCHAR(32) NOT NULL UNIQUE, age INT DEFAULT 18, bio TEXT COMMENT '简介');` → 表 users 4 字段：`id` pk+类型 UUID；`name` 类型 `VARCHAR(32)`（参数化整串保留）+ not_null + unique；`age` default="18"；`bio` comment="简介"
  2. `AUTO_INCREMENT` 列 → increment=true；PG `SERIAL`/`BIGSERIAL` 类型列 → increment=true
  3. 表级 `PRIMARY KEY(a, b)` → a、b 两列 primary=true（联合主键）
  4. 反引号/双引号/方括号标识符（`` `user` `` / `"user"` / `[user]`）→ 列名去引号
  5. `INSERT INTO ...` / `CREATE INDEX ...` 等非 CREATE TABLE 语句跳过；全文无合法 CREATE TABLE → Err
  6. 未知类型（如 `MONEY`）整串原样保留，不做方言映射

### UT-PC-08 — DDL 导入外键生成 references 纯函数测试

- **位置**：`frontend-rs/src/editor_panels.rs`（`parse_sql_import_tables` 或拆出的 REFERENCES 收集函数）
- **步骤与预期**：
  1. 表级约束 `FOREIGN KEY (user_id) REFERENCES users(id)` → references 含 1 条，start=当前表.user_id，end=users.id（端点解析到表/字段实体 id）
  2. 列级 `user_id UUID REFERENCES users(id)` → 同样生成 1 条
  3. 无 REFERENCES 的 DDL → references 为空
  4. 指向不存在表/列的 FK → 跳过该条（不报错、不产悬空引用）

### UT-PC-09 — 导入合并纯函数测试

- **位置**：`frontend-rs/src/editor_panels.rs`（`merge_import_into_store`）
- **步骤与预期**：
  1. 现有 1 表 (x=100,y=100)，导入 2 表 → 新表落在现有内容右侧/下方网格，不与现有表包围盒重叠；现有表坐标不变
  2. 导入 references 追加到合并结果末尾，端点 id 原样保留（解析时已解析到导入表实体 id）
  3. 空 store（0 表）→ 新表从默认起点网格落位
  4. 幂等性由调用方保证（纯函数不修改入参，返回新 Vec）

### UT-PC-10 — ListView IO 入口与菜单边界锚点测试

- **位置**：`frontend-rs/src/editor_panels.rs`
- **断言**（`include_str!` 锚点口径）：
  1. 存在 `data-testid="list-btn-import"` 与 `data-testid="list-btn-export"`
  2. ListView 工具条导入按钮接线到 IoDrawer 打开（on_open_import / on_open_export 或等价路径）
  3. `styles.css` 的 `.cdb-app-bar__overflow-menu` 规则含 `max-height` 与 `overflow-y`

### UT-PC-11 — SQLite introspection 端点测试

- **位置**：`backend/src/bridge_introspect.rs`（或 `phase3_bridge.rs` tests mod）
- **步骤与预期**：
  1. 在唯一临时目录用 sqlx-sqlite 建真实库：`users(id INTEGER PRIMARY KEY, name TEXT NOT NULL)` + `posts(id INTEGER PRIMARY KEY, user_id INTEGER REFERENCES users(id), title TEXT)`
  2. 调端点（或直连 `introspect_sqlite(path)`）→ `table_count=2`；`tables` 数组含 users / posts 两表；字段含列级 PK / NOT NULL；posts 表 FK 端点（user_id → users.id）；comment 全为空串（SQLite 引擎限制）
  3. 测试结束删除临时库（遵循 UT-PU-20 不污染工作区约束）

### UT-PC-12 — introspection 响应序列化与错误映射纯函数测试

- **位置**：`backend/src/bridge_introspect.rs`
- **步骤与预期**：
  1. fixture IR（含联合主键表、FK 表、保留字列名）→ SQLite/PG 两路序列化：联合主键各列 `primary=true`；FK 端点字段正确；`default` 缺省空串；`comment` 原样透传；保留字/大小写混合标识符原样输出（结构化 JSON 不加引号）
  2. `engine="mysql"` 或空 `source` → 400；不存在路径/不可达连接串 → 502；空库 → 200 `table_count=0` + `tables=[]`

### UT-PC-13 — PG introspection 嵌入式真实库测试

- **位置**：`backend/src/bridge_introspect.rs`（dev-dependency `postgresql_embedded`）
- **步骤与预期**：
  1. 启动嵌入式 PG；sqlx-postgres 建 `orders(a INT, b INT, PRIMARY KEY(a,b))` + `items(id SERIAL PRIMARY KEY, order_a INT, order_b INT, FOREIGN KEY (order_a, order_b) REFERENCES orders(a,b))`
  2. 调 introspect → `table_count=2`；`tables` 不含 `pg_catalog`/`information_schema` 系统表；orders 两列 `primary=true`（联合主键）；items 复合 FK 端点（order_a,order_b → orders.a,b）正确
  3. 首次运行允许二进制下载耗时（超时阈值放宽至 120s）；测试结束停止实例

### UT-PC-14 — 导出 DDL 真实库执行验证

- **位置**：`frontend-rs/tests/ddl_realdb_verify.rs`（dev-dependencies：`sqlx` sqlite feature + `postgresql_embedded`）
- **步骤与预期**：
  1. 构造 store：2 表（含 `VARCHAR(32)` 参数化类型、`DEFAULT`、自增主键）+ 1 条关系
  2. `export_diagram_sql(store, "postgres")` → 嵌入式 PG 逐句执行 → 成功；`information_schema.tables` 计数=2，FK 约束计数=1
  3. `export_diagram_sql(store, "sqlite")` → sqlx-sqlite 临时库执行 → 成功；`sqlite_master` 计数=2
  4. 任一执行失败视为导出器缺陷，本变更内修复生成逻辑（不接受 skip）

### UT-PC-15 — ImportDrawer 数据库来源锚点测试

- **位置**：`frontend-rs/src/editor_panels.rs`（tests mod）
- **断言**（`include_str!` 锚点口径）：
  1. 存在 `data-testid="import-db-engine"` / `import-db-source` / `import-db-connect"`
  2. `ImportFormat` 含 `Database` 变体且 format tabs 渲染第 4 个 Tab
  3. 「连接并解析」接线到 `import_from_connection` 客户端方法（`POST /api/v1/bridge/import/connect`）

### ST-PC-04 — 连接数据库导入 e2e

- **位置**：`frontend-rs/scripts/test-spec-parity-d.mjs`（installApi 新增 `POST /api/v1/bridge/import/connect` mock：返回 1 表结构化 tables JSON）
- **步骤与预期**：
  1. 进编辑器建 2 表 → 更多菜单 → 导入 → 切「数据库」Tab
  2. 引擎选 SQLite、连接信息填 `/tmp/demo.db` → 点「连接并解析」→ 摘要显示「已连接 · 检测到 1 张表」
  3. 点「导入到画布」→ PUT 落账表数 2→3；Toast「已导入」；URL 不变；无 `POST /bridge/import/local`
  4. localStorage 中不出现连接信息字符串

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

### ST-PC-05 — 导出到数据库 e2e

- **前置**：编辑器已加载（2 表），mock `POST /api/v1/bridge/export/execute` 返回 `{ statements: 2, tables: 2 }`
- **步骤**：导出抽屉 → SQL Tab → 填连接信息 → 「在数据库中执行」
- **预期**：请求体 `ddl` 与预览区 SQL 一致；`export-db-result` 显示「已执行 2 条语句 · 建表 2 张」；Toast「已导出到数据库」；连接信息不出现在持久化存储

### UT-PC-20 — 导入合并 ID 重键全局唯一

- **位置**：`editor_panels::tests::test_merge_import_rekeys_entity_ids_ut_pc_20`
- **步骤**：构造两组 `parse_sql_import_tables` 结果（确定性 ID `import-t-0` 系列）；先后经 `merge_import_into_store` 合并
- **断言**：
  1. 两次合并产出的表/字段/关系 ID 互不相同（无 `import-t-` 字面残留；匹配 `^(auto|ref)-[0-9a-f]{16}$` 形式）
  2. reference 四端点（start/end table/field）全部指向重键后的新 ID，无悬空引用
  3. 表名、字段名、类型、PK/NOT NULL 等语义内容不受重键影响

### UT-PC-21 — PG introspection 按 schema 精确过滤

- **位置**：`bridge_introspect::tests::ut_pc_21_introspect_pg_schema_scope`
- **夹具**：嵌入式 PG 建两个 schema（`public` 与 `app`），各建同名表 `users`（列不同）+ `app.orders` 带 FK → `app.users`
- **断言**：
  1. 缺省（不带 schema）→ 仅 `public.users`，列集合为 public 版本
  2. `schema=app` → 仅 `app` 的 `users`+`orders`，FK 指向 `app.users`；`public.users` 不出现
  3. 不存在的 schema → 0 张表（不报错）

### UT-PC-22 — 前端锚点：ImportDrawer schema 输入

- **位置**：`editor_panels::tests::test_import_db_schema_anchor_ut_pc_22`
- **断言**（源码锚点口径）：
  1. `import-db-schema` testid 存在且仅在 `postgres` 引擎分支渲染
  2. `import_from_connection` 客户端方法签名与请求体包含 `schema`
  3. sqlite 引擎分支不发送 `schema`

### ST-PC-06 — 数据库导入二次导入 + 保存成功 + schema 透传

- **位置**：`frontend-rs/scripts/test-spec-parity-d.mjs`
- **步骤**：进入编辑器 → 导入抽屉数据库 Tab → 连接并解析（mock `/bridge/import/connect`）→ 导入合并 → 再次连接并解析 → 再次导入 → 触发保存（mock `PUT /diagrams/{id}`）
- **断言**：
  1. 两次导入合并后画布表数为两次之和（无 ID 冲突去重/覆盖）
  2. 两次保存请求体中表/字段/关系 ID 无交集（全局唯一）
  3. 两次保存均返回 200，saveText 为「已保存」
  4. 引擎选 postgres 时 `import-db-schema` 可见，请求体 `schema` 为输入值；选 sqlite 时请求体无 `schema` 字段


### UT-PC-23 — 样式锚点：select 背景简写修复

- **位置**：`editor_panels::tests::test_select_background_shorthand_anchor_ut_pc_23`
- **断言**：
  1. `.cdb-create-room-modal .cdb-input` 与 `.cdb-auth-card .cdb-input` 规则块不含 `background:` 简写（防止重置 chevron image/repeat）
  2. `[data-mode="dark"] .cdb-form-select, [data-mode="dark"] .cdb-select` 规则块含 `background-repeat: no-repeat` 与 `background-position`（双保险）

### UT-PC-24 — PG introspection 注释直采

- **位置**：`backend/src/bridge_introspect.rs`（`ut_pc_24_introspect_pg_comments`，dev-dependency `postgresql_embedded`，复用 UT-PC-13/21 实例夹具）
- **夹具**：嵌入式 PG 建 `users(id SERIAL PRIMARY KEY, status TEXT)` + `COMMENT ON TABLE users IS '用户表';` + `COMMENT ON COLUMN users.status IS '状态';` + `COMMENT ON COLUMN users.status IS 'it''s fine';`（转义）
- **断言**：
  1. `tables[].name=="users"` → `comment=="用户表"`（obj_description）
  2. `status` 字段 `comment=="状态"`；转义夹具 `comment=="it's fine"`（col_description）
  3. 无注释表（如同库再建裸表）→ comment 为空串

### UT-PC-25 — SQLite introspection 注释引擎限制 + 响应 schema

- **位置**：`backend/src/bridge_introspect.rs`（`ut_pc_25_introspect_sqlite_response_schema`）
- **断言**：
  1. 响应 `table_count == tables.len()`；0 表时 `tables=[]`（替代旧 `ddl=""` 断言）
  2. 每表含 `name`/`comment`/`fields`；每字段含 `name`/`type`/`primary`/`not_null`/`default`/`comment`
  3. 全部 comment 为空串且不报错（SQLite 引擎限制，见 `core-01d` §4.7）

### UT-PC-26 — import/connect 结构化响应前端消费

- **位置**：`frontend-rs/src/editor_panels.rs`（tests mod）
- **步骤**：mock 响应 `{ engine:"postgres", table_count:1, tables:[{name:"users", comment:"用户表", fields:[{name:"id", type:"INTEGER", primary:true, not_null:true, default:"", comment:""},{name:"status", type:"TEXT", comment:"状态"}]}] }` → 前端消费函数（`parse_json_import_tables` 直喂 `data.tables` 序列化文本，或轻适配 `parse_bridge_import_tables`）
- **断言**：产出 1 表 2 字段；`Table.comment=="用户表"`；`status.comment=="状态"`；表/字段 ID 仍为解析期确定性形式（重键在 `merge_import_into_store` 一次完成，见 UT-PC-20）

### UT-PC-27 — 导出 COMMENT ON TABLE

- **位置**：`frontend-rs/src/editor_panels.rs`（`export_diagram_sql` tests）
- **断言**：
  1. postgres：`out.contains("COMMENT ON TABLE users IS '用户表';")` 且位于 `CREATE TABLE users` 之后
  2. mysql：`out.contains("COMMENT='用户表'")`（表选项），且不出现 `COMMENT ON TABLE`
  3. 空 comment 表：无对应语句
  4. 单引号：comment `it's` → 输出 `IS 'it''s';`

### UT-PC-28 — JSON 导入注释透传（加强断言）

- **位置**：`frontend-rs/src/editor_panels.rs`（`parse_json_import_tables` tests）
- **说明**：`JsonField.comment` 反序列化（`#[serde(default)]`）既有实现已透传，本用例将其固化为显式断言
- **断言**：表/字段 comment 非空值原样透传；缺省键落空串；与 UT-PC-26 数据库导入路径产出逐字段一致

### UT-PC-29 — 表树注释展示锚点

- **位置**：`frontend-rs/src/editor_panels.rs` + `styles.css`
- **断言**（`include_str!` 锚点口径）：
  1. `list-table-comment-` testid 存在于 `t.comment` 非空条件分支
  2. 无注释表不渲染注释行（不占位）
  3. `styles.css` 注释行类含单行截断（`text-overflow` / `overflow`）

### UT-PC-30 — Inspector 表/字段注释编辑锚点

- **位置**：`frontend-rs/src/editor_panels.rs`
- **断言**（`include_str!` 锚点口径）：
  1. `inspector-table-comment` 受控输入存在（`prop:value` 绑定当前表 comment），blur 落账通路仿 `on_rename_table`（store 写 → undo → schedule_save）
  2. `inspector-field-comment-` testid 存在于 Inspector 字段卡
  3. `list-field-comment-` 既有锚点保留（ListView 字段网格第 2 列）

### ST-PC-07 — 导出→导入 round-trip 注释不丢

- **位置**：`frontend-rs/tests/`（native）或 spec-parity 脚本
- **步骤**：构造 store（1 表，表 comment + 2 列各带 comment）→ `export_diagram_sql(store, "postgres")` → 输出喂入 `parse_sql_import_tables`
- **断言**：解析结果表/列 comment 与原始 store 逐字段一致（`COMMENT ON TABLE` / `COMMENT ON COLUMN` 均回填）

### ST-PC-08 — Inspector 表注释编辑端到端

- **前置**：编辑器已加载（1 表无注释），mock `PUT /diagrams/{id}`
- **步骤**：选中表 → Inspector `inspector-table-comment` 输入「用户表」→ blur → 等待自动保存
- **断言**：
  1. PUT 请求体该表 `comment=="用户表"`
  2. 表树渲染 `list-table-comment-{table_id}` 文本「用户表」
  3. mock 重载后注释展示仍在（落库链路通）

### 变更记录

| TC ID | 变更 | 说明 |
|-------|------|------|
| ST-PC-01 | MODIFIED（fix-appbar-roomname-back-and-import-merge） | 导入语义由「bridge 新建游离 diagram + 跳转」改为「本地合并进当前画布」；不再断言 bridge 返回 diagramId |
| UT-PC-09 | ADDED（fix-appbar-roomname-back-and-import-merge） | `merge_import_into_store` 避让布局合并纯函数 |
| ST-PC-02 | ADDED（fix-overflow-menu-room-delete-listview-io） | ListView 导入/导出入口端到端 |
| ST-PC-03 | ADDED（fix-overflow-menu-room-delete-listview-io） | 更多菜单小视口边界保护 |
| UT-PC-10 | ADDED（fix-overflow-menu-room-delete-listview-io） | ListView IO 按钮 + 菜单边界 CSS 锚点 |
| UT-PC-11～13 | ADDED（feat-db-connect-import-and-ddl-realdb-verify） | 连接数据库导入：后端 introspection 端点（SQLite 真实库 / 渲染纯函数 / 嵌入式 PG） |
| UT-PC-14 | ADDED（feat-db-connect-import-and-ddl-realdb-verify） | 导出 DDL 真实库执行验证（嵌入式 PG + SQLite） |
| UT-PC-15 | ADDED（feat-db-connect-import-and-ddl-realdb-verify） | ImportDrawer 数据库来源 UI 锚点 |
| ST-PC-04 | ADDED（feat-db-connect-import-and-ddl-realdb-verify） | 连接数据库导入端到端（mock 端点 + 本地合并管道复用） |
| UT-PC-16~17 | ADDED（feat-room-recycle-bin-dropdown-and-db-export） | 连接数据库导出执行：SQLite 真实库 / 嵌入式 PG + 错误映射 |
| UT-PC-18 | ADDED（feat-room-recycle-bin-dropdown-and-db-export） | ExportDrawer 导出到数据库 UI 锚点 |
| UT-PC-19 | ADDED（feat-room-recycle-bin-dropdown-and-db-export） | 表单下拉（select）样式条款锚点 |
| ST-PC-05 | ADDED（feat-room-recycle-bin-dropdown-and-db-export） | 导出到数据库端到端（mock 端点） |
| UT-PC-08 | MODIFIED（fix-dbimport-save-and-pg-schema） | FK 端点断言由字面 `import-t-0/1` 改为「端点与对应表/字段重键后 ID 一致」（解析器内部 ID 不变，仅合并后断言口径变化） |
| UT-PC-09 | MODIFIED（fix-dbimport-save-and-pg-schema） | `merge_import_into_store` 断言由字面 `import-t-1` 改为重键一致性 + 全局唯一性 |
| UT-PC-20~22 / ST-PC-06 | ADDED（fix-dbimport-save-and-pg-schema） | 导入合并 ID 重键 / PG schema introspection / schema 输入锚点 / 二次导入+保存 e2e |
| UT-PC-23 | ADDED（fix-select-bg-diagram-delete-and-log-format） | select 背景简写平铺回归锚点 |
| UT-PC-07 | MODIFIED（fix-canvas-zoom-invite-comment-resize） | `parse_sql_import_tables` 新增 PG `COMMENT ON TABLE` / `COMMENT ON COLUMN` 解析回填（目标不存在跳过） |
| UT-PC-11 / UT-PC-12 / UT-PC-13 / ST-PC-04 | MODIFIED（fix-canvas-zoom-invite-comment-resize） | `import/connect` 响应由 `{tables: int, ddl: string}` 改为 `{table_count, tables[]}` 结构化 JSON；既有 `ddl` 断言迁移为 `tables` 数组断言 |
| UT-PC-24 / UT-PC-25 | ADDED（fix-canvas-zoom-invite-comment-resize） | PG introspection 表/列注释（obj_description/col_description）/ SQLite 注释引擎限制 + 响应 schema |
| UT-PC-26 | ADDED（fix-canvas-zoom-invite-comment-resize） | import/connect 结构化响应前端消费（复用 JSON 导入路径） |
| UT-PC-27 | ADDED（fix-canvas-zoom-invite-comment-resize） | 导出 `COMMENT ON TABLE`（pg/generic 后置、mysql 表选项） |
| UT-PC-28 | ADDED（fix-canvas-zoom-invite-comment-resize） | JSON 导入注释透传加强断言 |
| UT-PC-29 / UT-PC-30 / ST-PC-08 | ADDED（fix-canvas-zoom-invite-comment-resize） | 表树 / Inspector 表注释展示与编辑锚点及端到端（testid：`list-table-comment-*` / `inspector-table-comment` / `inspector-field-comment-*`） |
| ST-PC-07 | ADDED（fix-canvas-zoom-invite-comment-resize） | 导出→导入 round-trip 注释不丢 |

### UT-ALIGN-B01 — DiagramClient bridge config GET/PUT

- **位置**：`frontend-rs/src/editor_data_access.rs`（`get_bridge_config` / `update_bridge_config`）
- **Given**：mock HTTP 返回 envelope `{ "code": 0, "data": { "db_read_preferred": false, "db_write_enabled": true, "dual_write_local": false, "updated_at": "2026-06-21" }, "request_id": "r1" }`
- **When**：调用 `get_bridge_config()`
- **Then**：
  - 返回 `BridgeConfig` 字段与 `data` 一致
  - `db_read_preferred == false`，`db_write_enabled == true`
- **When**：调用 `update_bridge_config(&BridgeConfigUpdate { dual_write_local: Some(true), ..Default::default() })`，mock PUT 返回 200 + `{ "code": 0, "data": { "updated": true }, "request_id": "r2" }`
- **Then**：返回 `Ok(())`

### UT-ALIGN-B02 — DiagramClient 导入日志列表与重试

- **位置**：`frontend-rs/src/editor_data_access.rs`（`list_import_logs` / `retry_import_log`）
- **Given**：mock GET `/bridge/import/local/logs` 返回 1 条 `{ id: "log-1", status: "failed", retry_count: 0, error_message: "parse error" }`
- **When**：调用 `list_import_logs(None)`
- **Then**：`Vec<ImportLogEntry>` len==1 且 `id == "log-1"`
- **When**：mock POST `/bridge/import/local/retry/log-1` 返回 `{ id: "log-1", status: "success", retry_count: 1, diagram_id: "d-new" }`
- **When**：调用 `retry_import_log("log-1")`
- **Then**：`RetryImportResponse.diagram_id == Some("d-new")` 且 `retry_count == 1`

### UT-ALIGN-B03 — AppBar 溢出菜单与 ImportDrawer 日志区

- **位置**：`frontend-rs/src/editor_panels.rs`（`AppBarOverflow` / `ImportDrawer` / `BridgeSettingsModal`）
- **Given**：编辑器已加载，overflow 菜单未展开
- **When**：点击 `data-testid="btn-bridge-settings"`（或等效设置项）
- **Then**：`data-testid="modal-bridge-settings"` 可见；含 `db_read_preferred` / `db_write_enabled` / `dual_write_local` 三个开关
- **When**：点击 `data-testid="btn-delete-diagram"` 且 confirm 返回 true
- **Then**：调用 `DiagramClient::delete(current_id)`；成功后跳转 `/editor`
- **When**：打开 Import 抽屉
- **Then**：`data-testid="import-logs-panel"` 可见；失败条目显示 `data-testid="import-log-retry-{id}"` 按钮
- **When**：点击 refresh（`import-logs-refresh`）
- **Then**：再次调用 `list_import_logs`

### 回归更新

| TC ID | 变更 |
|-------|------|
| UT-AB-04 | Phase C：`btn-import` **enabled**（替换 Phase A disabled 断言） |

### 不在范围

| 条目 | 变更 |
|------|------|
| SQL/DBML 全屏代码视图（Phase D） | 保持不变 |
| Mermaid / PNG 导出 | 保持不变 |

## 统一原型对齐范围与状态

IO：入口经 AppBar **更多菜单** → IO 抽屉；格式 SQL/DBML/JSON（及既有 bridge 能力）。

状态：后端已实现；生产前端部分接入。本提案 `implement-unified-prototype-spec-parity`（D 批）将 ST-PC-MENU/FMT/INSPECTOR 落实为自动化，结果写入 `logos/resources/verify/test-results.jsonl`。不得将「规格已写」标为「生产已完成」。

## ADDED / MODIFIED — 入口与格式

| ID | 前置 | 操作 | 预期 | 状态 |
|---|---|---|---|---|
| ST-PC-MENU-01 | room-editor | 打开更多菜单 → 导入/导出 | 打开 IO 抽屉；非历史独立 Import 模态为主路径 | 本提案 D 批实现 |
| ST-PC-FMT-01 | 导出抽屉 | 切换 SQL/DBML/JSON | 预览随模型更新；可复制/下载（生产以规格为准） | 本提案 D 批实现 |
| ST-PC-INSPECTOR | Inspector 展开 | 打开 IO | Inspector 折叠或让位；关闭 IO 后恢复 | 本提案 D 批实现 |
| UT-PC-01～05 / ST-PC-01 | 既有 | — | 保留；入口叙述改为更多菜单→抽屉 | 既有；D 批回归 |

## 边界

- Mermaid / PNG/PDF 等未实现格式不得标完成。
- 演示导入数据 ≠ 生产 bridge 成功。
