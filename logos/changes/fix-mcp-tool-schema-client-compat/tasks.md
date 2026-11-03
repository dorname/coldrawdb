# 实现任务

## [delta] 规格变更

- [x] 产出 `deltas/api/mcp-tools.yaml` — 拍平 `update_table` / `update_reference` 的 inputSchema（删除 `allOf/if/then/else/oneOf/not/anyOf`，条件规则移入 description），并登记「工具 schema 客户端兼容性约束」条款
- [x] 产出 `deltas/test/core-S06-test-cases.md` — 登记 UT-MCP-50：全工具 inputSchema/outputSchema 顶层客户端兼容守卫与关键属性保留

## [code] 代码实现

- [x] 在 `mcp-server/tests/c6_write_contract.rs` 交付 UT-MCP-50 — 遍历 `McpService::tools()` 断言 12 个工具 schema 顶层无高级条件关键字且 `update_table`/`update_reference` 关键属性完整，接 reporter
- [x] merge 后重建 `mcp-server/target/release/coldrawdb-mcp` 并以真实 stdio `tools/list` 探针复核 12 工具全部可见、schema 为平铺结构
