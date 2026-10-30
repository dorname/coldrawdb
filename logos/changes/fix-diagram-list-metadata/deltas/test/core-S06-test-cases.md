## ADDED — UT-MCP-40 list_diagrams 保留真实摘要元数据

- **位置**：`mcp-server/tests/c2_read_tools.rs`。
- **输入**：上游列表项包含 `revision:8`、`updated_at:'2026-10-29T12:34:56Z'` 和不同的 `lastModified`；另含仅提供旧 `lastModified` 的兼容项。
- **操作**：执行 `normalize_list`。
- **断言**：新字段项的 revision/updated_at 原样保留，不退化为 0/null，updated_at 优先于 lastModified；旧字段项仍回退到 lastModified；名称过滤、排序、limit 行为不变。
- **reporter**：登记 `UT-MCP-40` 到 `logos/resources/verify/test-results.jsonl`。

### 用例登记（OpenLogos verify 解析用，追加行）

| ID | 层级 | 说明 |
|---|---|---|
| UT-MCP-40 | UT | list_diagrams 透传真实 revision/updated_at，并保留 lastModified 回退 |
