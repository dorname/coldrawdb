# 实现任务

## [delta] 规格变更

- [x] 产出 delta 文件到 `deltas/prd/1-product-requirements/core-S06-mcp-service-requirements.md` — 房间写入与 stderr 验收
- [x] 产出 delta 文件到 `deltas/prd/2-product-design/1-feature-specs/core-S06-mcp-service-design.md` — 写路径、错误码、可观测性
- [x] 产出 delta 文件到 `deltas/prd/3-technical-plan/2-scenario-implementation/core-S06-ai-client-mcp.md` — 房间 op 时序
- [x] 产出 delta 文件到 `deltas/test/core-S06-test-cases.md` — UT-MCP-25～27

## [code] 代码实现

- [x] `mcp-server`：成功 tool_call 不写 stderr；失败日志含 `code` 与 `message`
- [x] `mcp-server`：409 `USE_OP_CHANNEL` 不再映射为 `REVISION_CONFLICT`；房间图经 WS op 写入
- [x] 补齐 UT-MCP-25～27，并保持非房间 PUT 测试通过
