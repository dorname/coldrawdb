# 变更提案：修复图表列表摘要元数据

> module: core | created: 2026-10-31 | issue: #53

## 变更原因

Issue #53 指出 MCP `list_diagrams` 及其上游 `GET /diagrams/queryAll` 的摘要始终返回 `revision=0`、`updated_at=null`，而 `get_diagram` 中同一图已有真实 revision 和修改时间，导致同名图无法按新旧区分。

根因是遗留列表查询仍使用旧 SeaORM `diagram::Model` / `DiagramVo` 映射，该模型只含 `last_modified`，没有读取迁移后新增的 `revision`、`updated_at` 列；MCP 规范化收到缺字段后按既有兼容逻辑回退为 `0/null`。

## 变更类型

接口级变更（补齐既有摘要字段的真实值，不新增端点，不改变 MCP 工具名称或参数）。

## 变更范围

- 影响的需求文档：`core-S06-mcp-service-requirements.md` FR-S06-02。
- 影响的功能规格：`core-S06-mcp-service-design.md` §4.1。
- 影响的业务场景：S04 创建房间关联图候选、S06 图表发现。
- 影响的 API：`GET /diagrams/queryAll` 响应项、MCP `list_diagrams` 的 `DiagramSummary` 契约。
- 影响的 DB 表：`diagram`（只读取既有 `revision`、`updated_at`，不改 Schema）。
- 影响的测试：`core-S04-test-cases.md`、`core-S06-test-cases.md`。
- 不影响：正式 `/api/v1/diagrams/{id}` 单图结构、列表过滤/排序/limit、软删除过滤、数据库迁移、可选 table/reference 计数。

## 部署影响

- 是否需要部署：否。
- 部署原因：本地后端查询映射与 MCP 摘要回归，无部署或数据迁移。
- 影响环境：无。
- 是否涉及数据迁移：否。
- 是否需要回滚预案：否。
- 是否需要 smoke：否。

## 变更概述

遗留 `queryAll` 列表查询直接读取 `diagram` 的既有元数据列，响应项同时保留原有 `lastModified` 兼容字段，并返回真实 `revision` 与 `updated_at`；`updated_at` 缺失时仅为旧数据回退到 `lastModified`，不得用常量或当前时间伪造。

MCP `list_diagrams` 保持现有过滤、名称排序和 limit 行为，只透传真实元数据。新增后端真实 SQLite 回归与 MCP 规范化回归，确保 revision=8 等值不会在列表链路中退化为 0/null。
