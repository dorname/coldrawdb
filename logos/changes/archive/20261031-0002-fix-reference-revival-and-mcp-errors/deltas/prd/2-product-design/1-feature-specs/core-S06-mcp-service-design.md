## MODIFIED — 4.3 错误体验

| 条件 | MCP 错误码 | retryable | 用户动作 |
|---|---|---|---|
| 配置缺失/URL 非 http(s) | CONFIG_INVALID | false | 修正客户端配置并重启 |
| HTTP 连接失败 | UPSTREAM_UNREACHABLE | true | 按 message/details 中的 BASE_URL origin 启动 backend 或修正地址 |
| 请求超时 | UPSTREAM_TIMEOUT | true | 检查上游负载或网络后重试 |
| 400/422 | VALIDATION_ERROR | false | 根据 details 修正参数 |
| 401，或房间写缺少 token | UNAUTHENTICATED | false | 更新 Access Token |
| 403，或房间成员只读 | PERMISSION_DENIED / READ_ONLY | false | 使用有写权限的身份 |
| 404，或找不到绑定房间 | NOT_FOUND | false | 重新列出 diagram / 房间 |
| 409 且 details 含 current_revision | REVISION_CONFLICT | false | 读取最新 diagram，人工合并后重试 |
| 409 且 details.code=USE_OP_CHANNEL | 不作为最终错误；转入 op 通道 | false | 无需用户改参；通道失败时再返回对应码 |
| 5xx | UPSTREAM_ERROR | true | 稍后重试并使用 request_id 排查 |

工具错误结果使用 `isError: true`，`content[0].text` 为完整 `ToolError` 的 JSON 序列化，至少包含 `code`、`message`、`retryable`，可选 `request_id`、`details`。HTTP 连接失败的 message 与 `details.base_url` 均包含实际 BASE_URL 的脱敏 origin（scheme/host/port，不含 userinfo、query、fragment 或 token）。

错误结果**不得包含 `structuredContent`**：各工具的 `outputSchema` 只约束成功响应，若把 `ToolError` 放入 `structuredContent`，Cursor 等客户端会按成功 schema 校验并以「缺少 id」等二次错误掩盖真实原因。成功结果仍保持 `structuredContent` 与 `content[0].text` 表示同一对象，协议不变。

所有错误文本和 details 均不得包含 Authorization、Cookie、WebSocket URL 中的 token 或完整上游响应头。
