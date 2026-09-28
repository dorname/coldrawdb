# Delta — core-S06-test-cases.md（fix-issues-42-44-mcp-and-relation-hit）

## ADDED — UT 用例（追加，#42）

| ID | 场景 | 预期 |
|---|---|---|
| UT-MCP-36 | mock 上游 409 返回**非 JSON** 文本但含 `USE_OP_CHANNEL` 字样 → `update_diagram` | 识别为房间图收编，走 WS op 通道回退（不落入笼统 VALIDATION_ERROR）（R-MCPERR-02） |
| UT-MCP-37 | mock 上游 502 返回 HTML 错误页 → `update_diagram` | 错误 details 含 `http_status=502`、真实 `content_type`、`body_excerpt`（≤200 字符摘要）；message 可辨认网关错误（R-MCPERR-01） |
| UT-MCP-38 | 读取 `import_schema` tool 描述 | 明确含「仅新建/不覆盖已有 id」语义文案（R-MCPERR-04） |

- **reporter**：上述 UT 均写入 `logos/resources/verify/test-results.jsonl`

## MODIFIED — 用例登记（OpenLogos verify 解析用，追加行）

| ID | 层级 | 说明 |
|---|---|---|
| UT-MCP-36 | UT | 非 JSON 409 含 USE_OP_CHANNEL 字样仍走 op 通道（#42） |
| UT-MCP-37 | UT | 非 JSON 上游错误 details 携带 status/content_type/body_excerpt（#42） |
| UT-MCP-38 | UT | import_schema tool 描述「仅新建」文案（#42） |
