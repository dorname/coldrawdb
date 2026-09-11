# 合并指令

## 变更提案
- 提案名称：fix-dbimport-save-and-pg-schema
- 提案目录：logos/changes/fix-dbimport-save-and-pg-schema/

## 提案内容

# 变更提案：数据库导入后保存失败修复 + PostgreSQL 导入 schema 定位

> module: core | created: 2026-09-08

## 变更原因

用户目标（2026-09-08，两项）：

1. **问题修复：从数据库导入表数据之后保存失败。** 根因已定位：导入合并管道（`parse_sql_import_tables` / `parse_dbml_import_tables` / `parse_json_import_tables`）生成的实体 ID 是**确定性的**——表 `import-t-{序号}`、字段 `import-t-{i}-f{n}`、关系 `import-r-{n}`（editor_panels.rs:756/958/1032/1078/644）。而后端 `table` / `field` / `reference` 的 `id` 是**全局单列主键**（init.sql:47/65/113，`PRIMARY KEY("id")`）。第一次导入保存后这些 ID 即在全局表中占位；此后**任意 diagram 再做一次数据库导入**（含同一图二次导入、新图首次导入），保存时 `INSERT INTO "table"(id…)` 主键冲突 → 500 → 前端落入「保存失败（离线）」。这与 `fix-global-entity-id-uniqueness` 修过的「`auto-N` 计数器 → 新账户建表保存 500」是同一类根因（editor_core.rs:261 注释），但导入管道当时未接入 `new_entity_id` 全局唯一 ID 生成器。
2. **问题优化：PostgreSQL 导入需要支持定位到 schema。** 现状 `introspect_postgres`（bridge_introspect.rs:256）所有查询仅排除 `pg_catalog` / `information_schema`，把**全部用户 schema 的表混在一起**：无法只导 `public` 或指定 schema；不同 schema 的同名表还会互相串列、串 PK/FK。需支持请求级 schema 参数，默认 `public`。

## 变更类型

接口级变更（import/connect 请求体新增可选 `schema` 字段）+ 代码级修复（导入合并 ID 重键）；无 DB 结构变更、无部署变更

## 变更范围

- 影响的需求文档：无（不新增场景编号）
- 影响的功能规格：
  - `core-01d-import-export.md` — ImportDrawer 数据库来源新增 schema 输入（仅 PG 显示）；本地合并管道「导入实体 ID 重键为全局唯一」条款
  - `core-03-bridge-io.md` — §13 连接数据库导入：请求体新增可选 `schema`（仅 postgres 生效，默认 `public`）
- 影响的业务场景：Phase C（导入导出）
- 影响的部署方案：无
- 影响的 API：
  - `api/bridge.yaml` — `POST /api/v1/bridge/import/connect` 请求体新增可选 `schema: string`
- 影响的 DB 表：无结构变更
- 影响的编排测试：无（遵循既有惯例）
- 影响的 smoke 测试：无
- 影响的测试用例文档：
  - `core-PC-import-export-test-cases.md` — 新增导入合并 ID 重键 UT、PG schema introspection UT（嵌入式 PG）、前端 schema 输入锚点 UT、二次导入+保存 e2e；UT-PC-08/09 断言口径由「字面 import-t-N」改为「重键后一致性」需 MODIFIED 登记

## 部署影响

- 是否需要部署：否
- 部署原因：开发阶段缺陷修复与能力增强，本地重新构建即生效；无新部署物、无配置变更、无数据迁移
- 影响环境：无
- 是否涉及数据迁移：否（存量已保存的 `import-t-N` 数据不受影响；修复只影响新导入的合并行为）
- 是否需要回滚预案：否
- 是否需要 smoke：否

## 变更概述

**修复 1：导入实体 ID 重键。** 在本地合并管道入口 `merge_import_into_store`（SQL/DBML/JSON 与数据库连接导入的共同漏斗）对导入的新表/字段/关系做 ID 重键：以 `new_entity_id`（fix-global-entity-id-uniqueness 引入的全局唯一生成器）替换确定性 `import-t-N` 系列 ID，并按 旧→新 映射同步改写 reference 的四个端点（start/end table/field）。重键后任意次导入保存均不会再撞全局主键。UT-PC-08/09 既有断言中针对 `import-t-1` 字面量的部分随之改为断言重键一致性与唯一性。

**优化 2：PG 导入 schema 定位。** `POST /api/v1/bridge/import/connect` 请求体新增可选 `schema` 字段：仅 `engine=postgres` 时生效，缺省 `public`；`introspect_postgres` 的表/列/PK/FK 四个查询全部改为 `table_schema = $2` / `ns.nspname = $2` 精确过滤，杜绝跨 schema 串表。SQLite 忽略该字段。前端 ImportDrawer 数据库面板在引擎选 `postgres` 时显示「Schema」输入框（占位 `public`），随请求发送；引擎为 `sqlite` 时不显示、不发送。


## 需要合并的 Delta 文件

### 1. deltas/api/bridge.yaml

- Delta 文件：`logos/changes/fix-dbimport-save-and-pg-schema/deltas/api/bridge.yaml`
- 目标目录：`logos/resources/api/`
- 操作：读取 delta 中的 ADDED / MODIFIED / REMOVED 标记，合并到目标目录中对应的主文档

### 2. deltas/prd/2-product-design/1-feature-specs/core-01d-import-export.md

- Delta 文件：`logos/changes/fix-dbimport-save-and-pg-schema/deltas/prd/2-product-design/1-feature-specs/core-01d-import-export.md`
- 目标目录：`logos/resources/prd/2-product-design/1-feature-specs/`
- 操作：读取 delta 中的 ADDED / MODIFIED / REMOVED 标记，合并到目标目录中对应的主文档

### 3. deltas/prd/2-product-design/1-feature-specs/core-03-bridge-io.md

- Delta 文件：`logos/changes/fix-dbimport-save-and-pg-schema/deltas/prd/2-product-design/1-feature-specs/core-03-bridge-io.md`
- 目标目录：`logos/resources/prd/2-product-design/1-feature-specs/`
- 操作：读取 delta 中的 ADDED / MODIFIED / REMOVED 标记，合并到目标目录中对应的主文档

### 4. deltas/test/core-PC-import-export-test-cases.md

- Delta 文件：`logos/changes/fix-dbimport-save-and-pg-schema/deltas/test/core-PC-import-export-test-cases.md`
- 目标目录：`logos/resources/test/`
- 操作：读取 delta 中的 ADDED / MODIFIED / REMOVED 标记，合并到目标目录中对应的主文档

## 执行要求

1. 逐个 Delta 文件处理，每处理完一个报告修改摘要
2. 对于 ADDED 标记：在主文档的指定位置插入新内容
3. 对于 MODIFIED 标记：替换主文档中同名章节的内容
4. 对于 REMOVED 标记：从主文档中删除对应章节
5. 保持主文档的原有格式和风格
6. 如果主文档有"最后更新"时间戳，同步更新
7. 所有变更完成后，列出修改清单
8. 所有变更合并完成后，自动执行 git commit（告知用户，无需确认）：
   git add -A && git commit -m "docs(fix-dbimport-save-and-pg-schema): merge spec deltas"
   然后提示用户：按更新后的规格实现代码，代码完成后运行 `openlogos verify` 验收，验收通过后明确授权执行 `openlogos archive fix-dbimport-save-and-pg-schema`。
