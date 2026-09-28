# Delta — core-S06-mcp-service-design.md（fix-issues-42-44-mcp-and-relation-hit）

## ADDED — §4.5 update_diagram 错误可诊断与房间图回退鲁棒性（fix-issues-42-44-mcp-and-relation-hit / #42）

**背景**：#42——`ApiClient::request` 对非 JSON 响应体一律吞成「上游返回非 JSON 响应」，丢失 status/Content-Type/body；且 409 `USE_OP_CHANNEL` 识别要求 body 为特定 JSON 结构（`details.code`），body 非 JSON 时房间图 WS op 通道回退不触发，用户被迫只能 `import_schema` 新建。

合同条款：

- **R-MCPERR-01 错误可诊断**：上游非 2xx 时，无论 body 是否为合法 JSON，`ToolError.details` 必须携带：
  - `http_status`（整数）
  - `content_type`（响应 Content-Type，缺省为空串）
  - `body_excerpt`（body 文本截断前 200 字符；body 为合法 JSON 时为原 JSON 摘要）
  错误 message 不再只有「非 JSON」；网关 502/504 HTML 页、代理拦截页必须可被用户从错误中直接辨认。
- **R-MCPERR-02 USE_OP_CHANNEL 鲁棒识别**：409 的识别路径依次为——①body 为 JSON 且 `details.code == "USE_OP_CHANNEL"`（既有）；②body 非 JSON 但文本包含 `USE_OP_CHANNEL` 字样。任一路径命中即走 WS op 通道回退（`write_room_diagram`），不得落入笼统 VALIDATION_ERROR。
- **R-MCPERR-03 op 通道失败归因**：op 通道写入失败时错误码必须为协作通道语义（`UPSTREAM_UNAVAILABLE` / 既有 collab 错误码），message 指明「协作通道」而非「非 JSON」。
- **R-MCPERR-04 import_schema 仅新建文案**：`import_schema` tool 描述明确「仅创建新图表，不覆盖已有 id；覆盖/更新已有图表请用 update_diagram / update_table 等增量工具」。

验收：UT-MCP-36 / UT-MCP-37 / UT-MCP-38。
