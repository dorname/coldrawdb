# 变更提案：连接数据库导入 + DDL 真实库验证

> module: core | created: 2026-09-07

## 变更原因

用户反馈（2026-09-07，四项完善优化之第 3/4 项，经决策拆分为独立变更）：

1. **连接数据库导入**：当前导入仅支持粘贴 DDL 文本，用户希望直接连接真实数据库（SQLite 文件 / PostgreSQL 连接串） introspect schema 后导入画布。
2. **DDL 真实库验证**：导出生成的 DDL 目前只做文本级断言，未验证能否在真实数据库执行；需要验证 PostgreSQL 与 SQLite 两个引擎。

关键决策（用户已确认）：
- 连接导入支持 **SQLite + PostgreSQL** 两种引擎
- PG 真实库验证使用**嵌入式 PG**（postgresql_embedded crate，运行时从 Maven Central 拉取二进制，已实测本机网络可达）
- 与变更 A（菜单边界/房间删除/ListView IO 入口）拆分为两个提案，本变更为 B

## 变更类型

设计级变更（新功能：产品交互 + 新 API 端点 + 测试验证体系扩展）

## 变更范围

- 影响的需求文档：无（既有 S01 编辑保存场景的导入能力增强，不新增场景编号）
- 影响的功能规格：
  - `core-03-bridge-io.md` — bridge 新增连接数据库导入能力（端点行为、错误语义、安全边界）
  - `core-01d-import-export.md` — ImportDrawer 新增「从数据库连接导入」来源；DDL 导出真实库验证口径
  - `core-00-information-architecture.md` — IA 同步（导入来源新增「数据库连接」）
- 影响的业务场景：S01（导入增强，不改动时序图主链路）
- 影响的部署方案：无（无新部署物；postgresql_embedded 仅 dev-dependency 测试用）
- 影响的 API：`api/bridge.yaml` 新增 `POST /api/v1/bridge/import/connect`
- 影响的 DB 表：无（不改动应用自身 schema；introspection 目标是用户外部库）
- 影响的编排测试：无（非 API 编排项目链路）
- 影响的 smoke 测试：无
- 影响的测试用例文档：
  - `core-PC-import-export-test-cases.md` — 新增连接导入 UT/ST + DDL 真实库验证 UT
  - `core-PV-verify-pre-run-test-cases.md` — 标题式用例兼容索引登记（如有）

## 部署影响

- 是否需要部署：**否**
- 部署原因：开发阶段功能增强，本地重新构建即生效；无新部署物、无配置变更、无数据迁移（遵循近期后端端点变更惯例，如 p0-fix 删除房间端点）
- 影响环境：无
- 是否涉及数据迁移：否
- 是否需要回滚预案：否
- 是否需要 smoke：否

## 变更概述

**特性 1：连接数据库导入。** 后端 `phase3_bridge` 新增 `POST /api/v1/bridge/import/connect` 端点：请求携带 `{ engine: "sqlite" | "postgres", source }`（SQLite 为服务端本机文件路径，PG 为连接串），后端用 sqlx 连接目标库 introspect schema（SQLite 读 `sqlite_master`/`PRAGMA table_info`，PG 读 `information_schema`），生成对应方言的 DDL 文本返回。前端 ImportDrawer 新增「数据库连接」来源（引擎下拉 + 连接参数输入 + 连接解析按钮），拿到 DDL 后**复用既有本地合并管道**（`parse_sql_import_tables` → 解析摘要 → `merge_import_into_store` 避让合并 → PUT 落账），不新建游离 diagram、不跳转页面。错误语义：连接失败 / 无表 / 不支持引擎分类文案。

**特性 2：DDL 真实库验证。** frontend-rs 新增集成测试（native target，dev-dependencies：`sqlx` sqlite feature + `postgresql_embedded`）：调用导出纯函数 `export_diagram_sql` 生成 DDL，分别在真实 SQLite 与嵌入式 PG 实例上执行，断言执行成功且建表数量/外键与模型一致。覆盖本次承诺验证的 PostgreSQL 与 SQLite 两路方言输出。

安全边界：连接导入的 `source` 由后端进程本机解析（文件路径/连接串不出服务端）；端点挂既有 auth 中间件；不在前端持久化连接串。
