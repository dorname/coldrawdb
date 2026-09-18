# 实现任务

## [delta] 规格变更

- [x] 产出 delta 文件到 `deltas/api/mcp-tools.yaml` — 新增 `update_table` / `update_field` / `update_reference` / `layout_diagram` 四个 tool 定义 + 相应 schemas
- [x] 产出 delta 文件到 `deltas/prd/3-technical-plan/2-scenario-implementation/core-S06-ai-client-mcp.md` — 新增画布编辑时序（辅时序）与工具推导表更新
- [x] 产出 delta 文件到 `deltas/test/core-S06-test-cases.md` — 新增 UT-MCP-16～22、ST-MCP-10～12

## [code] 代码实现

- [x] 新增 `mcp-server/src/layout.rs` — 力导向布局算法（Fruchterman-Reingold 变体，确定性输出，固定随机种子）
- [x] 更新 `mcp-server/src/service.rs` — 实现 `update_table` / `update_field` / `update_reference` / `layout_diagram` 四个 tool handler
- [x] 更新 `mcp-server/src/api.rs` — 如需要，新增辅助函数（全量 diagram 读取 + 修改 + 写回）
- [x] 编写 UT 测试：UT-MCP-16～22（参数校验、布局算法正确性、工具契约稳定性）
- [x] 编写 ST 测试：ST-MCP-10～12（端到端画布编辑链路，mock HTTP）
- [x] 所有测试通过 OpenLogos reporter 写入 `logos/resources/verify/test-results.jsonl`