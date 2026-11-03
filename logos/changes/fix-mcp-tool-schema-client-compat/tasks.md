# 实现任务

## [delta] 规格变更

- [x] 产出 `deltas/api/mcp-tools.yaml` — 拍平 `update_table` / `update_reference` 的 inputSchema（删除 `allOf/if/then/else/oneOf/not/anyOf`，条件规则移入 description），并登记「工具 schema 客户端兼容性约束」条款
- [x] 产出 `deltas/test/core-S06-test-cases.md` — 登记 UT-MCP-50：全工具 inputSchema/outputSchema 顶层客户端兼容守卫与关键属性保留

## [code] 代码实现

- [x] 在 `mcp-server/tests/c6_write_contract.rs` 交付 UT-MCP-50 — 遍历 `McpService::tools()` 断言 12 个工具 schema 顶层无高级条件关键字且 `update_table`/`update_reference` 关键属性完整，接 reporter
- [x] merge 后重建 `mcp-server/target/release/coldrawdb-mcp` 并以真实 stdio `tools/list` 探针复核 12 工具全部可见、schema 为平铺结构

## [code] 勘误：ST-PB-04 harness 竞态修复（verify 阶段暴露，详见 proposal.md 勘误补充）

- [x] 修复 `frontend-rs/scripts/test-spec-parity-d.mjs` ST-PB-04 — 在点击连线与 Delete/Backspace 派发之间加双 rAF + 短静默窗稳态等待（#67 先例模式），消除残留 Inspector 表单短路 waitFor 导致的选中竞速；断言语义零变化
- [x] `SPEC_PARITY_ONLY=ST-PB-04` 连续复跑 ≥2 次确认稳定 PASS 后，重跑 `openlogos verify` 全量验收
