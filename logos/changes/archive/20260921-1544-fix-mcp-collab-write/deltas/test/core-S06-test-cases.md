## ADDED — UT-MCP-25 — USE_OP_CHANNEL 不映射为 REVISION_CONFLICT

- **位置**：`mcp-server/src/error.rs`
- **GIVEN**：上游 HTTP 409，body.details.code 为 `USE_OP_CHANNEL`
- **WHEN**：映射为 ToolError
- **THEN**：`code` 为 `USE_OP_CHANNEL`，不是 `REVISION_CONFLICT`；`details.code` 保留。普通 409（仅 `current_revision`）仍为 `REVISION_CONFLICT`

## ADDED — UT-MCP-26 — 房间图经 op 通道写入

- **位置**：`mcp-server` 协作 diff / 会话逻辑
- **GIVEN**：活跃房间 `diagramId` 匹配；基准表名与目标表名不同；假 socket 先回 `connected` 再回 `ack.serverRev`
- **WHEN**：规划并提交房间写入
- **THEN**：产生不含 fields 的 `table.update`；提交后 revision 为 ack 的 serverRev；`READ_ONLY` 在第一帧失败且不再继续发送；空 diff 保持当前 revision；无 token 为 `UNAUTHENTICATED` 且错误文本不含 token

## ADDED — UT-MCP-27 — 成功调用不写 stderr

- **位置**：`mcp-server` tool_call 日志
- **GIVEN**：tool_call 结果为成功或失败
- **WHEN**：决定是否写 stderr
- **THEN**：`status=ok` 不产生日志行；`status=error` 的 JSON 含 `code` 与 `message`，不含 diagram 正文

## MODIFIED — 4. 验收标准追溯

| 验收标准 | 用例 |
|---|---|
| MCP-AC-01 协议互通 | UT-MCP-02/03/11、ST-MCP-01、ST-MCP-06～09 |
| MCP-AC-02 读链路 | UT-MCP-04～06、ST-MCP-02 |
| MCP-AC-03 写链路 | UT-MCP-07～10、ST-MCP-03 |
| MCP-AC-04 一致性与安全 | UT-MCP-12～15、UT-MCP-25～27、ST-MCP-04/05 |
| MCP-AC-05 配置可用 | UT-MCP-01/11、ST-MCP-06～09 |
| MCP-AC-06 画布编辑 | UT-MCP-16～24、ST-MCP-10～13 |
| MCP-AC-07 房间写入与日志 | UT-MCP-25～27 |

### 用例登记（OpenLogos verify 解析用）

| ID | 输入/操作 | 断言 |
|---|---|---|
| UT-MCP-23 | mock 上游 PUT 200 且 `data` 含额外字段 | update_* 四工具恰好 `{id,revision}`；`layout_diagram` 恰好 `{id,revision,tables_repositioned}`；缺 `id` 以路径补齐；`revision` 缺/非数→`UPSTREAM_INVALID` |
| UT-MCP-24 | `update_reference` 带 / 不带 `color` | create 含 `color`；update 不传保持原值；`color` >64 → `VALIDATION_ERROR` |
| UT-MCP-25 | 上游 409 `details.code=USE_OP_CHANNEL` | 映射 `USE_OP_CHANNEL`，不是 `REVISION_CONFLICT` |
| UT-MCP-26 | 房间 diff + 假 socket 提交 op | `table.update` 不含 fields；ack 后 revision 为 serverRev；READ_ONLY / 空 diff / 无 token 符合断言 |
| UT-MCP-27 | tool_call 成功与失败 | 成功不产生 stderr 行；失败 JSON 含 code 与 message |
| ST-MCP-13 | stdio → `layout_diagram(mode=force)` | `structuredContent` 过 `outputSchema`；`content[0].text` == `structuredContent`；坐标更新且 revision 递增 |
