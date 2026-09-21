# 合并指令

## 变更提案
- 提案名称：fix-mcp-collab-write
- 提案目录：logos/changes/fix-mcp-collab-write/

## 提案内容

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


## 需要合并的 Delta 文件

### 1. deltas/prd/1-product-requirements/core-S06-mcp-service-requirements.md

- Delta 文件：`logos/changes/fix-mcp-collab-write/deltas/prd/1-product-requirements/core-S06-mcp-service-requirements.md`
- 目标目录：`logos/resources/prd/1-product-requirements/`
- 操作：读取 delta 中的 ADDED / MODIFIED / REMOVED 标记，合并到目标目录中对应的主文档

### 2. deltas/prd/2-product-design/1-feature-specs/core-S06-mcp-service-design.md

- Delta 文件：`logos/changes/fix-mcp-collab-write/deltas/prd/2-product-design/1-feature-specs/core-S06-mcp-service-design.md`
- 目标目录：`logos/resources/prd/2-product-design/1-feature-specs/`
- 操作：读取 delta 中的 ADDED / MODIFIED / REMOVED 标记，合并到目标目录中对应的主文档

### 3. deltas/prd/3-technical-plan/2-scenario-implementation/core-S06-ai-client-mcp.md

- Delta 文件：`logos/changes/fix-mcp-collab-write/deltas/prd/3-technical-plan/2-scenario-implementation/core-S06-ai-client-mcp.md`
- 目标目录：`logos/resources/prd/3-technical-plan/2-scenario-implementation/`
- 操作：读取 delta 中的 ADDED / MODIFIED / REMOVED 标记，合并到目标目录中对应的主文档

### 4. deltas/test/core-S06-test-cases.md

- Delta 文件：`logos/changes/fix-mcp-collab-write/deltas/test/core-S06-test-cases.md`
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
   git add -A && git commit -m "docs(fix-mcp-collab-write): merge spec deltas"
   然后提示用户：按更新后的规格实现代码，代码完成后运行 `openlogos verify` 验收，验收通过后明确授权执行 `openlogos archive fix-mcp-collab-write`。
