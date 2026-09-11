# Delta: core-PC-import-export-test-cases.md

> 模块：core | 提案：fix-canvas-zoom-invite-comment-resize（问题3：数据库导入注释丢失 + 注释展示）

编号自 UT-PC-23 / ST-PC-06 续编。响应结构变更（`ddl` → `tables` 结构化 JSON）影响既有用例 UT-PC-11 / UT-PC-12 / UT-PC-13 / ST-PC-04，见文末变更记录。

## MODIFIED — 用例行（响应结构变更适配）

| TC ID | Given | When | Then |
|-------|-------|------|------|
| UT-PC-11 | 临时真实 SQLite 库（2 表 + PK/FK） | `POST /bridge/import/connect` `{engine:"sqlite", source:<路径>}` | 200；`table_count=2`；`tables` 数组含两表（name / fields 含列级 PK/NOT_NULL 与 FK 端点字段）；**SQLite 无注释：所有 comment 为空串**（引擎限制，不报错） |
| UT-PC-12 | introspection IR fixtures | IR → 响应序列化 + 端点错误映射 | SQLite/PG 两路序列化正确（联合主键各列 primary=true、default 缺省空串、comment 透传）；engine 非法 → 400；连接失败 → 502；0 表 → 200 `table_count=0` 且 `tables=[]` |
| UT-PC-13 | 嵌入式 PG 实例（建 2 表含联合主键 + FK） | `POST /bridge/import/connect` `{engine:"postgres", source:<连接串>}` | 200；`table_count=2`；`tables` 含 PG 规范化类型（`character varying(n)` → `VARCHAR(n)`）、联合主键两列 primary=true、FK 表/列端点正确；不含 `pg_catalog`/`information_schema` 系统表 |
| ST-PC-04 | 编辑器已加载（已有 2 表），mock `import/connect` 返回 1 表结构化 tables | 导入抽屉 → 数据库 Tab → 填连接信息 → 连接并解析 → 导入到画布 | 解析摘要「已连接 · 检测到 1 张表」；提交走本地合并（PUT 表数 2→3、Toast「已导入」、不跳转、不调 `bridge/import/local`）；连接信息不出现在持久化存储 |
| UT-PC-07 | DDL 文本（多表，含参数化类型/列级约束/DEFAULT/COMMENT/表级联合主键） | `parse_sql_import_tables(content)` | 表名/列名/类型整串（`VARCHAR(32)` 保留）正确；列级 PK/NOT NULL/UNIQUE/AUTO_INCREMENT（SERIAL→自增）/DEFAULT/COMMENT 落字段；**PG `COMMENT ON TABLE t IS '...'` 回填表 comment、`COMMENT ON COLUMN t.c IS '...'` 回填列 comment；目标不存在则跳过不报错**；表级 `PRIMARY KEY(a,b)` 双列置位；引号/方括号规范化；非 CREATE TABLE 语句跳过；无合法语句报错 |

## ADDED — 用例行

| TC ID | Given | When | Then |
|-------|-------|------|------|
| UT-PC-24 | 嵌入式 PG：建表带 `COMMENT ON TABLE` + `COMMENT ON COLUMN`（含中文与单引号转义） | `POST /bridge/import/connect` introspect | 200；`tables[].comment` = 表注释（obj_description）；对应 `fields[].comment` = 列注释（col_description）；无注释表/列返回空串 |
| UT-PC-25 | 临时真实 SQLite 库（2 表） | `POST /bridge/import/connect` | 200；响应 schema 完整（table_count == `tables.len()`；每表含 name/comment/fields；每字段含 name/type/primary/not_null/default/comment）；所有 comment 为空串 |
| UT-PC-26 | mock 结构化 `import/connect` 响应（1 表 2 字段含 comment） | 前端消费路径（`data.tables` → JSON 导入解析） | `parse_json_import_tables`（或轻适配层）输出 `Table.comment` / `Field.comment` 与响应一致；确定性 ID 形式不变（合并入口重键，见 UT-PC-20） |
| UT-PC-27 | store 含表（表 comment 非空、列 comment 非空） | `export_diagram_sql(store, engine)` | postgres/generic 输出含 `COMMENT ON TABLE t IS '...';`（位于该表 CREATE TABLE 之后）；mysql 输出含表选项 `COMMENT='...'`；空 comment 不输出；单引号转义为 `''` |
| UT-PC-28 | JSON 导入文本（tables 含表/字段 comment，字段缺省 comment 键） | `parse_json_import_tables(content)` | **加强断言**：表/字段 comment 原样透传；缺省键落空串；与 UT-PC-26 数据库导入路径产出一致 |
| UT-PC-29 | — | `include_str!` 锚点 | `editor_panels.rs` 含 `list-table-comment-` testid 且仅在 `t.comment` 非空分支渲染（无注释不占位）；`styles.css` 注释行含单行截断规则 |
| UT-PC-30 | — | `include_str!` 锚点 | `editor_panels.rs` 含 `inspector-table-comment`（受控输入 + blur 落账通路，仿 `on_rename_table`）与 `inspector-field-comment-`；`list-field-comment-` 既有锚点保留 |
| ST-PC-07 | store 含表（表/列 comment 非空） | 导出 PG SQL → 粘贴回 SQL 导入 Tab → 解析合并 | round-trip 后表/列 comment 与原始一致（COMMENT ON TABLE / COMMENT ON COLUMN 解析回填） |
| ST-PC-08 | 编辑器已加载（1 表无注释），mock `PUT /diagrams/{id}` | Inspector 填表注释 → blur → 保存；表树注释行可见；重新加载 | `inspector-table-comment` blur 后 PUT 请求体 `table.comment` 一致；表树出现 `list-table-comment-{id}`；重载后注释仍在 |

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
| UT-PC-07 | MODIFIED（fix-canvas-zoom-invite-comment-resize） | `parse_sql_import_tables` 新增 PG `COMMENT ON TABLE` / `COMMENT ON COLUMN` 解析回填（目标不存在跳过） |
| UT-PC-11 / UT-PC-12 / UT-PC-13 / ST-PC-04 | MODIFIED（fix-canvas-zoom-invite-comment-resize） | `import/connect` 响应由 `{tables: int, ddl: string}` 改为 `{table_count, tables[]}` 结构化 JSON；既有 `ddl` 断言迁移为 `tables` 数组断言 |
| UT-PC-24 / UT-PC-25 | ADDED | PG introspection 表/列注释（obj_description/col_description）/ SQLite 注释引擎限制 + 响应 schema |
| UT-PC-26 | ADDED | import/connect 结构化响应前端消费（复用 JSON 导入路径） |
| UT-PC-27 | ADDED | 导出 `COMMENT ON TABLE`（pg/generic 后置、mysql 表选项） |
| UT-PC-28 | ADDED | JSON 导入注释透传加强断言 |
| UT-PC-29 / UT-PC-30 / ST-PC-08 | ADDED | 表树 / Inspector 表注释展示与编辑锚点及端到端（testid：`list-table-comment-*` / `inspector-table-comment` / `inspector-field-comment-*`） |
| ST-PC-07 | ADDED | 导出→导入 round-trip 注释不丢 |
