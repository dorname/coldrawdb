# Delta: core-PC-import-export-test-cases.md

> 提案：fix-dbimport-save-and-pg-schema

## ADDED — UT 用例

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
- **断言**（源码锚点）：
  1. `import-db-schema` testid 存在且仅在 `postgres` 引擎分支渲染
  2. `import_from_connection` 客户端方法签名与请求体包含 `schema`
  3. sqlite 引擎分支不发送 `schema`

## ADDED — ST 用例

### ST-PC-06 — 数据库导入二次导入 + 保存成功 + schema 透传

- **位置**：`frontend-rs/scripts/test-spec-parity-d.mjs`
- **步骤**：进入编辑器 → 导入抽屉数据库 Tab → 连接并解析（mock `/bridge/import/connect`）→ 导入合并 → 再次连接并解析 → 再次导入 → 触发保存（mock `PUT /diagrams/{id}`）
- **断言**：
  1. 两次导入合并后画布表数为两次之和（无 ID 冲突去重/覆盖）
  2. 两次保存请求体中表/字段/关系 ID 无交集（全局唯一）
  3. 两次保存均返回 200，saveText 为「已保存」
  4. 引擎选 postgres 时 `import-db-schema` 可见，请求体 `schema` 为输入值；选 sqlite 时请求体无 `schema` 字段

## MODIFIED — 既有用例口径

| 用例 | 变更 | 说明 |
|---|---|---|
| UT-PC-08 | MODIFIED（fix-dbimport-save-and-pg-schema） | FK 端点断言由字面 `import-t-0/1` 改为「端点与对应表/字段重键后 ID 一致」（解析器内部 ID 不变，仅合并后断言口径变化） |
| UT-PC-09 | MODIFIED（fix-dbimport-save-and-pg-schema） | `merge_import_into_store` 断言由字面 `import-t-1` 改为重键一致性 + 全局唯一性 |

## ADDED — 变更记录行

| UT-PC-20~22 / ST-PC-06 | ADDED（fix-dbimport-save-and-pg-schema） | 导入合并 ID 重键 / PG schema introspection / schema 输入锚点 / 二次导入+保存 e2e |
