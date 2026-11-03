# Delta — core-S06-test-cases.md（fix-mcp-tool-schema-client-compat）

> 模块：core | 场景：S06 | 关联：Claude Code 丢弃含高级 schema 关键字的工具（10/12）

## ADDED — UT-MCP-50 — 工具 schema 客户端兼容守卫

- **位置**：`mcp-server/tests/c6_write_contract.rs`，纯本地断言 `McpService::tools()`，不依赖后端。
- **GIVEN**：`McpService::tools()` 返回的全部工具（应为 12 个）。
- **WHEN / THEN**：
  1. 遍历每个工具的 `inputSchema` 与 `outputSchema`，顶层键不得包含 `allOf`、`oneOf`、`anyOf`、`if`、`then`、`else`、`not`（客户端兼容约束；含任一即该客户端会丢弃工具）。
  2. `update_table.inputSchema.properties` 必须仍包含 `id`、`table_id`、`action`、`name`、`fields`（拍平不丢字段），`fields.items` 的 `$ref` 已解析为内联 schema（含 `name`/`type` 属性）。
  3. `update_reference.inputSchema.properties` 必须仍包含 `id`、`action`、`ref_id`、`references`、四个端点字段与 `cardinality`/`on_delete`/`on_update`；`references.items` 解析后含 `start_table_id` 等端点属性。
  4. 工具总数为 12，且包含 `update_table` 与 `update_reference`（回归锚点：客户端清单不得再退化为 10）。
- **reporter**：测试真实运行后登记 `UT-MCP-50` 到 `logos/resources/verify/test-results.jsonl`。

## ADDED — 用例登记（OpenLogos verify 解析用，fix-mcp-tool-schema-client-compat）

| ID | 层级 | 说明 |
|---|---|---|
| UT-MCP-50 | UT | 全工具 inputSchema/outputSchema 顶层客户端兼容守卫与拍平后关键属性保留 |

## MODIFIED — 4. 验收标准追溯

| 验收标准 | 用例 |
|---|---|
| MCP-AC-01 协议互通 | UT-MCP-02/03/11、ST-MCP-01、ST-MCP-06～09、UT-MCP-50 |
| MCP-AC-03 写链路 | UT-MCP-07～10、UT-MCP-28、UT-MCP-41～44、UT-MCP-50、ST-MCP-03/14/18 |
