# 变更提案：fix-mcp-collab-write

> module: core | created: 2026-09-21

## 变更原因

协作房间里的图表被后端收编为只能走 WebSocket op 通道：`PUT /api/v1/diagrams/{id}` 返回 409 `USE_OP_CHANNEL`。MCP 写工具（`update_diagram`、`update_table`、`update_field`、`update_reference`、`layout_diagram`）仍只发全量 PUT，并把一切 409 映射成 `REVISION_CONFLICT`，因此房间图无法写入，模型也看不到真正原因。

同时，每次 `tools/call`（含 `status=ok`）都向 stderr 打一行 JSON。Cursor 把 MCP 子进程的 stderr 标成 `[error]`，第二条日志参数缺失时显示 `undefined`。成功调用被误报为错误，失败调用又没有 `code`/`message`。

## 变更类型

设计级（S06 需求验收、功能设计、时序与测试用例；HTTP API / DB 不变）

## 变更范围

- 影响的需求文档：`prd/1-product-requirements/core-S06-mcp-service-requirements.md`（FR-S06-03、NFR-S06-03、验收场景）
- 影响的功能规格：`prd/2-product-design/1-feature-specs/core-S06-mcp-service-design.md`（写调用、错误表、可观测性）
- 影响的业务场景：S06（`core-S06-ai-client-mcp.md` 房间写入时序）
- 影响的部署方案：无
- 影响的 API：无（沿用既有 `PUT` 409 `USE_OP_CHANNEL` 与 `/ws/rooms/{roomId}`）
- 影响的 DB 表：无
- 影响的编排测试：无（`scenario/core-S06-mcp-service.json` 不改 HTTP 编排）
- 影响的 smoke 测试：无

## 部署影响

- 是否需要部署：否
- 部署原因：只改本地 MCP adapter 与 S06 规格；后端房间写收编已经在运行，无新服务、无迁移
- 影响环境：无
- 是否涉及数据迁移：否
- 是否需要回滚预案：否
- 是否需要 smoke：否

## UI/UX 变更声明

```yaml
ui_impact: false
design_system_mode: generated
design_system_fallback_reason: ""
pages: []
```

## 变更概述

1. 非房间图仍走 `PUT /api/v1/diagrams/{id}`。房间图在收到 `USE_OP_CHANNEL` 后，查找绑定该 diagram 的活跃房间，按当前物化文档与目标文档做 diff，经 WebSocket 提交 op，并返回与现契约相同的瘦响应 `{id, revision}`。
2. `expected_revision` 与服务端 revision 不一致时仍返回 `REVISION_CONFLICT`，不自动覆盖。viewer 返回 `READ_ONLY`。缺少 token 返回 `UNAUTHENTICATED`，且不回显 token。
3. 成功的 tool_call 不写 stderr。失败时 stderr 只有一行 JSON，包含 `code` 与 `message`，不含 diagram 正文和 token。
