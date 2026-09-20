# Delta — core-S06-test-cases.md（修改）

> 模块：core | 提案：fix-remote-github-issues-7-18
> 关联 issue：#18（写工具返回 normalize） | 规格：`api/mcp-tools.yaml`「写工具返回约定」

## ADDED — UT-MCP-23 — 写工具瘦响应 normalize（契约）

- **位置**：`mcp-server/src/service.rs`（`update_diagram` / `update_table` / `update_field` / `update_reference` / `layout_diagram`）
- **GIVEN**：mock 上游 PUT 200，`data` 为**含额外字段的形状**（如整图 diagram 或 `{id, revision, extra...}`）
- **WHEN**：调用各写工具
- **THEN**：返回值经各自 `outputSchema` 严格校验通过：
  - update_* 四工具：恰好 `{ id, revision }`，无 envelope / 无整图字段
  - `layout_diagram`：恰好 `{ id, revision, tables_repositioned }`
  - 上游 `data` 缺 `id` 时以请求路径 id 补齐；`revision` 缺/非数 → 映射 `UPSTREAM_INVALID`（或规格错误码），不得产出缺字段的成功响应

## ADDED — UT-MCP-24 — update_reference 支持 color 参数

- **断言**：`action=create` + `color:"#3af"` → PUT 的 diagram 中该 reference 含 `color`；`action=update` 不传 `color` → 原值保持；`color` 超长（>64）→ `VALIDATION_ERROR`

## ADDED — ST-MCP-13 — stdio 端到端：layout_diagram structuredContent 过校验

- **GIVEN**：mock 后端含多表+关系图；PUT 返回 `{id, revision}` 瘦响应
- **WHEN**：stdio → `layout_diagram(mode=force)` → 读取 tools/call 结果
- **THEN**：`structuredContent` 通过 `layout_diagram.outputSchema` 校验；`content[0].text` 反序列化 == `structuredContent`；坐标已更新且 revision 递增；同环境不再需要 curl 手工写回
- **reporter**：结果追加 `logos/resources/verify/test-results.jsonl`（`module: "core"`，`scenario: "S06"`）

## MODIFIED — 4. 验收标准追溯

| 验收标准 | 用例 |
|---|---|
| MCP-AC-01 协议互通 | UT-MCP-02/03/11、ST-MCP-01、ST-MCP-06～09 |
| MCP-AC-02 读链路 | UT-MCP-04～06、ST-MCP-02 |
| MCP-AC-03 写链路 | UT-MCP-07～10、ST-MCP-03 |
| MCP-AC-04 一致性与安全 | UT-MCP-12～15、ST-MCP-04/05 |
| MCP-AC-05 配置可用 | UT-MCP-01/11、ST-MCP-06～09 |
| MCP-AC-06 画布编辑 | UT-MCP-16～24、ST-MCP-10～13 |
