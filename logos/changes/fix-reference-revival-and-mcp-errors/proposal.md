# 变更提案：修复关系软删除重建与 MCP 上游错误诊断

> module: core | created: 2026-10-30 | issue: #57

## 变更原因

Issue #57 包含两个相互独立但影响同一写入链路的 Bug：

1. 全量保存只清理由 `diagram_link` 关联的实体，遗留的软删除 `reference` 行仍占用全局主键；同 ID 关系重建时触发 `UNIQUE constraint failed: reference.id`，用户无法通过正常编辑恢复。
2. MCP 虽能把连接失败映射为传输错误，但错误仍写入 `structuredContent`，导致 Cursor 等客户端按成功 `outputSchema` 校验并报「缺少 id」，掩盖真实原因；错误中也缺少实际使用的 `COLDRAWDB_BASE_URL`。

## 变更类型

接口级变更（持久化行为修复 + MCP 错误结果契约修正）。

## 变更范围

- 影响的需求文档：无新增需求；沿用 S01 持久化与 S06 错误体验验收目标。
- 影响的功能规格：`core-02-diagram-persistence.md` §3；`core-S06-mcp-service-design.md` §4.3。
- 影响的业务场景：S01（编辑保存）、S06（MCP stdio 服务）。
- 影响的 API：`PUT /api/v1/diagrams/{id}` 的同 ID 软删除关系重建行为；MCP `tools/call` 错误结果与 `UPSTREAM_UNREACHABLE` 错误码。
- 影响的 DB 表：`reference`（仅事务内清理同 ID 且 `is_deleted=1` 的遗留行，不改 Schema）。
- 影响的编排测试：`core-S01-test-cases.md`、`core-S06-test-cases.md`。
- 不影响：其他实体的全局 ID 约束、活跃关系行、成功 `structuredContent`、HTTP API envelope、部署配置。

## 部署影响

- 是否需要部署：否。
- 部署原因：本地源码与自动化回归修复，无环境或数据结构变更。
- 影响环境：无。
- 是否涉及数据迁移：否；保存时按需清理目标软删除行，不执行批量迁移。
- 是否需要回滚预案：否。
- 是否需要 smoke：否。

## 变更概述

后端在事务内插入每条关系前，仅删除同 ID 且已软删除的遗留行，再写入新关系；活跃的同 ID 行仍受全局唯一约束保护，不得被覆盖或误删。正常全量保存、revision 乐观锁和其他实体逻辑保持不变。

MCP HTTP 连接失败统一返回 `UPSTREAM_UNREACHABLE`，message 与 details 提供脱敏后的 BASE_URL origin，超时仍使用 `UPSTREAM_TIMEOUT`。所有工具错误结果保留 `isError: true` 和包含完整 `ToolError` JSON 的文本 `content`，但不再发送只适用于成功响应的 `structuredContent`，避免客户端 outputSchema 二次报错。成功响应协议不变。
