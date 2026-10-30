## MODIFIED — UT-MCP-14（HTTP 连接错误码命名）

| UT-MCP-14 | 连接失败/超时 | 分别映射 `UPSTREAM_UNREACHABLE` / `UPSTREAM_TIMEOUT`；写操作不自动重试 |

## ADDED — Issue #57 MCP 连接诊断与错误协议用例

### UT-MCP-39 — HTTP 连接失败包含 BASE_URL 诊断

- **位置**：`mcp-server/tests/c10_error_diagnosis.rs`。
- **操作**：`COLDRAWDB_BASE_URL` 指向未监听的本地端口，调用 `get_diagram`。
- **断言**：返回 `UPSTREAM_UNREACHABLE`、`retryable=true`；message 与 `details.base_url` 均包含配置的 origin；不包含 token、userinfo、query 或 fragment；不映射为成功 outputSchema 的字段错误。

### ST-MCP-17 — stdio 工具错误不再触发成功 outputSchema 校验

- **位置**：`mcp-server/tests/c10_error_diagnosis.rs`，真实 `coldrawdb-mcp` stdio 子进程。
- **操作**：以未监听端口为 BASE_URL，完成 initialize 后调用 `get_diagram`。
- **断言**：JSON-RPC 结果为 `result.isError=true`；不存在 `result.structuredContent`；`content[0].text` 可解析为完整 ToolError，含 `UPSTREAM_UNREACHABLE` 与 BASE_URL；不会出现成功 schema 所需的 `id` 伪字段，stdout 无其他非协议内容。
- **reporter**：分别登记 `UT-MCP-39`、`ST-MCP-17` 到 `logos/resources/verify/test-results.jsonl`。

### 用例登记（OpenLogos verify 解析用，追加行）

| ID | 层级 | 说明 |
|---|---|---|
| UT-MCP-39 | UT | HTTP 连接失败返回 UPSTREAM_UNREACHABLE 和脱敏 BASE_URL |
| ST-MCP-17 | ST | stdio 错误结果仅文本 content，无 structuredContent |

既有错误测试中读取 `result.structuredContent` 的断言同步改为解析 `result.content[0].text`；成功响应断言保持不变。
