## MODIFIED — 4.2 写调用

- `create_diagram` 只接受 `name` 和可选 `database`，与实际后端 `CreateReq` 对齐；复杂初始模型使用 `import_schema`。
- `update_diagram` 必须显式提供 `expected_revision` 与完整 `diagram`，adapter 不自动读取后覆盖。
- 非房间图的 `update_diagram`、`update_table`、`update_field`、`update_reference`、`layout_diagram` 仍经 `PUT /api/v1/diagrams/{id}` 写回。
- 房间绑定图的上述写工具在 PUT 返回 409 且 `details.code=USE_OP_CHANNEL` 时，不得把该响应当成最终结果。adapter 使用 `GET /api/v1/rooms` 查找 `diagramId` 匹配且未归档的房间，以当前物化文档为基准对目标文档做 diff，再连接 `ws(s)://…/ws/rooms/{roomId}?token=` 按序提交 op，直到收到 ack。成功响应仍是 `{id, revision}`（`layout_diagram` 另含 `tables_repositioned`）。
- 房间写入前若物化 `revision` 与 `expected_revision` 不一致，返回 `REVISION_CONFLICT`，不提交 op。无差异则保持当前 revision，不连接后空写。
- `delete_diagram` 必须显式提供 `confirm: true`；这不能替代客户端审批，只是服务侧第二道防误触。
- `import_schema` MVP 仅接受 `format: "drawdb_json"`，传给 API 的 payload 必须为 JSON object。

## MODIFIED — 4.3 错误体验

| 条件 | MCP 错误码 | retryable | 用户动作 |
|---|---|---|---|
| 配置缺失/URL 非 http(s) | CONFIG_INVALID | false | 修正客户端配置并重启 |
| 连接失败/超时 | UPSTREAM_UNAVAILABLE / UPSTREAM_TIMEOUT | true | 启动 backend 或检查 URL |
| 400/422 | VALIDATION_ERROR | false | 根据 details 修正参数 |
| 401，或房间写缺少 token | UNAUTHENTICATED | false | 更新 Access Token |
| 403，或房间成员只读 | PERMISSION_DENIED / READ_ONLY | false | 使用有写权限的身份 |
| 404，或找不到绑定房间 | NOT_FOUND | false | 重新列出 diagram / 房间 |
| 409 且 details 含 current_revision | REVISION_CONFLICT | false | 读取最新 diagram，人工合并后重试 |
| 409 且 details.code=USE_OP_CHANNEL | 不作为最终错误；转入 op 通道 | false | 无需用户改参；通道失败时再返回对应码 |
| 5xx | UPSTREAM_ERROR | true | 稍后重试并使用 request_id 排查 |

错误结果使用 `isError: true`，structuredContent 至少包含 `code`、`message`、`retryable`，可选 `request_id`、`details`；不得包含 Authorization、Cookie、WebSocket URL 中的 token 或完整上游响应头。

## MODIFIED — 7. 可观测性

成功的 `tools/call` 不写 stderr。失败时 stderr 恰好一行 JSON，字段为 `event=tool_call`、`tool`、`duration_ms`、`status=error`、`code`、`message`。禁止记录 diagram 正文、Authorization、Cookie 或 token。stdout 仅允许 MCP 帧。
