## MODIFIED — 2. What：功能需求

| ID | 需求 | 验收 |
|---|---|---|
| FR-S06-01 | 提供 MCP stdio 服务并完成 initialize、initialized、tools/list | 四客户端配置可启动服务；tools/list 返回七个稳定工具 |
| FR-S06-02 | 提供 `list_diagrams`、`get_diagram` | 返回结构化摘要或完整 diagram；不存在时返回可诊断错误 |
| FR-S06-03 | 提供 `create_diagram`、`update_diagram`、`delete_diagram`，以及画布写工具 | 非房间图 update 走 `PUT /api/v1/diagrams/{id}` 且必须携带 `expected_revision`；房间绑定图在 PUT 返回 `USE_OP_CHANNEL` 后改经 `/ws/rooms/{roomId}` 提交 op，成功响应仍为 `{id, revision}` |
| FR-S06-04 | 提供 `import_schema` | MVP 接受 drawdb JSON；SQL/DBML 文本导入仅在解析能力落地后开放对应枚举值 |
| FR-S06-05 | 提供 `export_schema` | 从完整 diagram 在 adapter 内确定性生成 JSON、DBML 或七类 SQL |
| FR-S06-06 | 提供四客户端配置 | Claude、Codex、Cursor、OpenCode 示例可复制，均指向同一二进制 |
| FR-S06-07 | 标注工具副作用 | 读工具 `readOnlyHint=true`；删除工具 `destructiveHint=true`；写工具不伪装为只读 |
| FR-S06-08 | 透传上游错误 | 网络、400/401/403/404/409/422/5xx 映射为稳定 MCP 错误码并保留安全的 request_id/details；房间写收编 409 的 `details.code=USE_OP_CHANNEL` 不得映射为 `REVISION_CONFLICT` |

## MODIFIED — 3. 非功能需求

| ID | 约束 | 指标 |
|---|---|---|
| NFR-S06-01 | 启动性能 | 本地 release 二进制 initialize P95 ≤ 1s |
| NFR-S06-02 | 调用超时 | 默认 30s，可通过 `COLDRAWDB_REQUEST_TIMEOUT_SECS` 配置为 1～120s |
| NFR-S06-03 | 安全与日志 | stdout 只写 MCP 帧；成功 tool_call 不写 stderr；失败时 stderr 仅一行 JSON，含 `code` 与 `message`；Token、Authorization、Cookie 必须脱敏 |
| NFR-S06-04 | 数据边界 | 禁止 SQLite 文件访问、任意 SQL 执行和 shell 执行工具 |
| NFR-S06-05 | 兼容 | MCP 客户端协商版本，不硬编码拒绝未知的新版本；工具 schema 保持向后兼容 |
| NFR-S06-06 | 可测试 | 每个工具至少一个 UT；主链、409 和配置兼容具有 ST；全部写 OpenLogos reporter |

## MODIFIED — 5. 验收场景

- GIVEN coldrawdb API 正常且客户端已配置 stdio 服务，WHEN 客户端完成 MCP 初始化，THEN 能发现七个工具及正确 annotations。
- GIVEN 已存在 diagram，WHEN 调用 `get_diagram` 与 `export_schema(format="dbml")`，THEN 返回同一 revision 对应的完整模型和确定性 DBML。
- GIVEN `expected_revision` 已过期，WHEN 调用 `update_diagram`，THEN 返回 `REVISION_CONFLICT` 和 `current_revision`，不自动覆盖。
- GIVEN diagram 绑定活跃协作房间且 `expected_revision` 等于当前物化 revision，WHEN 调用写工具，THEN adapter 经 WebSocket 提交 op 并返回新的 `revision`；viewer 得到 `READ_ONLY`；无 token 得到 `UNAUTHENTICATED` 且输出不含 token。
- GIVEN 用户未批准删除，WHEN 客户端看到 `destructiveHint=true`，THEN 应保留人工批准；服务端不得通过改名或组合工具绕过。
- GIVEN 配置包含 Token，WHEN 启动、调用、报错和退出，THEN stdout/stderr 与 reporter 均不出现 Token 明文；成功调用不产生 stderr 诊断行。
