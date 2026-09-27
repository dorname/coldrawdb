# Delta — core-PC-import-export-test-cases.md（fix-open-issues-26-33 / #29）

> 模块：core | 提案：fix-open-issues-26-33
> 目标：import/connect 产 references 的正向验收 + 双路径对称用例；改写 UT-PC-25/26 增加 references 断言。

## MODIFIED — UT-PC-25 — SQLite introspection 注释引擎限制 + 响应 schema

- **位置**：`backend/src/bridge_introspect.rs`（tests）
- **Given**：临时真实 SQLite 库：2 张业务表，其中 `orders.user_id` 为指向 `users.id` 的 FK（含 `ON DELETE CASCADE`）
- **When**：`POST /bridge/import/connect`（engine=sqlite）
- **Then**：
  1. 200；响应 schema 完整（`table_count == tables.len()`；每表含 `name`/`comment`/`fields`；每字段含 `name`/`type`/`primary`/`not_null`/`default`/`comment`）
  2. 全部 comment 为空串且不报错（SQLite 引擎限制，见 `core-01d` §4.7）
  3. **（本变更新增）** `references` 键存在且含 1 条：`{ table:"orders", column:"user_id", ref_table:"users", ref_column:"id", on_delete:"CASCADE", on_update:"" }`；无 FK 库时 `references` 为空数组而非缺键

## MODIFIED — UT-PC-26 — import/connect 结构化响应前端消费

- **位置**：`frontend-rs/src/editor_panels.rs`（tests mod）
- **步骤**：mock 响应 `{ engine:"postgres", table_count:1, tables:[{name:"users", comment:"用户表", fields:[{name:"id", type:"INTEGER", primary:true, not_null:true, default:"", comment:""},{name:"status", type:"TEXT", comment:"状态"}]}], references:[{table:"orders", column:"user_id", ref_table:"users", ref_column:"id", on_delete:"", on_update:""}] }`（tables 补齐 orders 表）→ 前端消费函数 `parse_bridge_import_tables(&data.tables, &data.references)`
- **断言**：产出 2 表；`Table.comment=="用户表"`；`status.comment=="状态"`；**（本变更新增）** 产出 1 条 `Reference`，端点解析到 orders.user_id → users.id 的确定性 ID；表/字段 ID 仍为解析期确定性形式（重键在 `merge_import_into_store` 一次完成，见 UT-PC-20）；`references` 传空数组时行为与旧版一致（仅表结构，不报错）

## ADDED — UT-PC-31 — 后端 fks → references 序列化（含复合 FK 展开）

- **位置**：`backend/src/bridge_introspect.rs`（`connect_response_data` tests）
- **Given**：手工构造 IR：表 `a`（列 `x`,`y`）、表 `b`（列 `x`,`y`）、表 `c`；`a` 含复合 FK `(x,y) → b(x,y)`；`a` 含指向不存在表 `ghost` 的 FK；一条 `columns` 与 `ref_columns` 长度不一致的残缺 FK
- **When**：`connect_response_data("postgres", &tables)`
- **Then**：
  1. 复合 FK 展开为 2 条 references（`a.x→b.x`、`a.y→b.y`），on_delete/on_update 透传
  2. 指向 `ghost` 的 FK 被跳过（不产悬空引用）；残缺 FK 按较短列数截断、残缺列对跳过
  3. 输出顺序确定（表序 + 收集顺序），快照断言稳定
  4. 无 FK 的 IR → `references: []`（键存在）

## ADDED — UT-PC-32 — 前端 bridge references 名址解析与合并

- **位置**：`frontend-rs/src/editor_panels.rs`（tests mod）
- **Given**：mock `tables`（users / orders，orders.user_id 为 FK 列）+ 名址 `references`（含 1 条合法 + 1 条指向不存在列 `orders.ghost` 的条目）
- **When**：`parse_bridge_import_tables` 后经 `merge_import_into_store(store, tables, references)`
- **Then**：
  1. 合法条目解析为 `Reference`：`start_*` = orders.user_id 重键后 ID，`end_*` = users.id 重键后 ID，无 `import-j-` 残留（同 UT-PC-20 口径）
  2. 指向不存在列的条目跳过，不报错
  3. 合并后 references 追加到既有关系末尾，既有关系不受影响

## ADDED — UT-PC-33 — SQL 导入 schema 限定 COMMENT ON 解析加固

- **位置**：`frontend-rs/src/editor_panels.rs`（`parse_sql_import_tables` tests）
- **Given**：DDL 含 `CREATE TABLE public.users (...)` 与 `CREATE TABLE users (...)` 两种写法 fixture；注释语句覆盖：`COMMENT ON TABLE public.users IS '用户表'`、`COMMENT ON COLUMN "app"."orders"."id" IS '主键'`、`COMMENT ON COLUMN orders.status IS '状态'`、指向不存在表/列的 `COMMENT ON`
- **When**：`parse_sql_import_tables(content)`
- **Then**：
  1. schema 限定 / 引号混写的注释目标按最后一段标识符（去引号）匹配到已解析表/列并回填 comment
  2. 指向不存在表/列的注释语句跳过（不报错）
  3. 既有非限定 `COMMENT ON` 与 MySQL 行内 `COMMENT` 行为不回归（UT-PC-07 / ST-PC-07 口径保持）

## ADDED — ST-PC-09 — 双路径对称 e2e（同一 PG schema）

- **位置**：`frontend-rs` e2e / bridge 集成（嵌入式 PG fixture）
- **Given**：嵌入式 PG schema 含 ≥3 表（含 ≥2 条 FK、表/列 `COMMENT ON` 注释）；导出其 `pg_dump` 风格 DDL 文本
- **When**：路径 A：SQL Tab 粘贴 DDL → 导入到画布；路径 B：数据库 Tab 连接该库 → 导入到画布（分别在新图进行）
- **Then**：两条路径画布的表数相等、关系数相等、非空表/列 comment 数一致（允许实体 ID 与坐标差异）
- **reporter**：`ST-PC-09` 写入 `logos/resources/verify/test-results.jsonl`

## MODIFIED — 变更记录（追加行）

| UT-PC-25 / UT-PC-26 | MODIFIED（fix-open-issues-26-33） | import/connect 响应与前端消费增加 `references`（fks 名址展开 → 画布关系线），废弃「import/connect 不产 references」的历史口径 |
| UT-PC-31 / UT-PC-32 / UT-PC-33 | ADDED（fix-open-issues-26-33） | 后端 fks→references 序列化 / 前端名址解析合并 / SQL schema 限定 COMMENT ON 加固 |
| ST-PC-09 | ADDED（fix-open-issues-26-33） | 同一 PG schema 双路径（SQL dump vs import/connect）对称 e2e |
