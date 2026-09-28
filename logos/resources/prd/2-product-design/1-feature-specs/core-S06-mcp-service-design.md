# S06：MCP 服务能力 — 功能设计

> 模块：core | 场景：S06 | 版本：V3 | 优先级：P1
> 输入：`core-S06-mcp-service-requirements.md` | 时序：`core-S06-ai-client-mcp.md` | 契约：`mcp-tools.yaml`

## 1. 产品形态

S06 是无 GUI 的本地 adapter。Claude、Codex、Cursor 或 OpenCode 启动 `coldrawdb-mcp` 子进程，通过 stdin/stdout 交换 MCP JSON-RPC；adapter 使用 HTTP 调用已运行的 coldrawdb backend。stdout 严禁混入日志，所有诊断信息写 stderr。

## 2. 生命周期

| 状态 | 进入条件 | 允许行为 | 退出条件 |
|---|---|---|---|
| starting | 客户端创建进程 | 读取并校验环境变量，初始化 HTTP client | 配置有效→initializing；无效→退出码 2 |
| initializing | 收到 initialize | 协商协议版本，返回 serverInfo/capabilities/instructions | 收到 initialized→ready |
| ready | 初始化完成 | tools/list、tools/call | stdin EOF、SIGTERM→stopping |
| stopping | 客户端关闭 | 停止接收新调用，等待当前调用完成或超时 | 退出码 0 |
| failed | 协议/内部不可恢复错误 | stderr 输出脱敏诊断 | 非 0 退出 |

`instructions` 前 512 字符必须包含：服务只操作 `COLDRAWDB_BASE_URL`；写入前读取最新 revision；删除具有破坏性；不得把工具当作任意 SQL 通道。

## 3. 工具目录

| 工具 | 目的 | 上游/实现 | annotations |
|---|---|---|---|
| `list_diagrams` | 列出图表摘要 | `GET /diagrams/queryAll`（遗留只读） | readOnly=true, destructive=false, idempotent=true |
| `get_diagram` | 获取完整图表 | `GET /api/v1/diagrams/{id}` | readOnly=true, destructive=false, idempotent=true |
| `create_diagram` | 创建空图表 | `POST /api/v1/diagrams` | readOnly=false, destructive=false, idempotent=false |
| `update_diagram` | 全量保存并 revision 校验 | `PUT /api/v1/diagrams/{id}` | readOnly=false, destructive=false, idempotent=false |
| `delete_diagram` | 软删除图表 | `DELETE /api/v1/diagrams/{id}` | readOnly=false, destructive=true, idempotent=true |
| `import_schema` | 导入 drawdb JSON | `POST /api/v1/diagrams/import` | readOnly=false, destructive=false, idempotent=false |
| `export_schema` | 导出 JSON/DBML/SQL | GET 完整图表 + adapter 纯函数序列化 | readOnly=true, destructive=false, idempotent=true |

## 4. 交互原则

### 4.1 读调用

- `list_diagrams` 默认最多返回 100 条摘要；adapter 负责稳定排序和本地 `limit` 截断。
- `get_diagram` 返回完整结构化 JSON，同时提供简短 text 摘要，避免客户端只能解析自然语言。
- `export_schema` 返回 `{diagram_id, revision, format, mime_type, content}`；相同输入和 revision 必须字节一致。

### 4.2 写调用

- `create_diagram` 只接受 `name` 和可选 `database`，与实际后端 `CreateReq` 对齐；复杂初始模型使用 `import_schema`。
- `update_diagram` 必须显式提供 `expected_revision` 与完整 `diagram`，adapter 不自动读取后覆盖。
- 非房间图的 `update_diagram`、`update_table`、`update_field`、`update_reference`、`layout_diagram` 仍经 `PUT /api/v1/diagrams/{id}` 写回。
- 房间绑定图的上述写工具在 PUT 返回 409 且 `details.code=USE_OP_CHANNEL` 时，不得把该响应当成最终结果。adapter 使用 `GET /api/v1/rooms` 查找 `diagramId` 匹配且未归档的房间，以当前物化文档为基准对目标文档做 diff，再连接 `ws(s)://…/ws/rooms/{roomId}?token=` 按序提交 op，直到收到 ack。成功响应仍是 `{id, revision}`（`layout_diagram` 另含 `tables_repositioned`）。
- 房间写入前若物化 `revision` 与 `expected_revision` 不一致，返回 `REVISION_CONFLICT`，不提交 op。无差异则保持当前 revision，不连接后空写。
- `delete_diagram` 必须显式提供 `confirm: true`；这不能替代客户端审批，只是服务侧第二道防误触。
- `import_schema` MVP 仅接受 `format: "drawdb_json"`，传给 API 的 payload 必须为 JSON object。

### 4.3 错误体验

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

## 5. 配置

| 环境变量 | 必填 | 默认 | 校验 |
|---|---|---|---|
| `COLDRAWDB_BASE_URL` | 是 | 无 | http/https；移除尾部 `/`；禁止 userinfo |
| `COLDRAWDB_ACCESS_TOKEN` | 否 | 无 | 仅内存使用，不输出、不写文件 |
| `COLDRAWDB_REQUEST_TIMEOUT_SECS` | 否 | 30 | 1～120 的整数 |
| `RUST_LOG` | 否 | warn | 日志写 stderr；敏感字段脱敏 |

## 6. 安全设计

- adapter 不依赖 sqlite/SeaORM，不接受数据库路径。
- HTTP client 只拼接契约中的固定路径；diagram id 作为单一 path segment 编码。
- 不提供 URL、header、method 等“通用 HTTP”工具参数。
- Token 只加入 Authorization header；Debug、Error、reporter 均使用脱敏包装类型。
- stdio transport 不监听端口；远程主机不得通过网络直接访问 MCP 进程。
- 当前 diagram API 未强制 S03 鉴权，此事实必须在 README/部署文档中展示，不能以可选 Token 暗示权限已经生效。

## 7. 可观测性

成功的 `tools/call` 不写 stderr。失败时 stderr 恰好一行 JSON，字段为 `event=tool_call`、`tool`、`duration_ms`、`status=error`、`code`、`message`。禁止记录 diagram 正文、Authorization、Cookie 或 token。stdout 仅允许 MCP 帧。

## 8. 四客户端配置

以下示例中的 `/ABS/PATH/coldrawdb-mcp` 必须替换为绝对路径，Token 推荐由客户端允许的环境变量转发机制提供。

### 8.1 Claude Code（项目 `.mcp.json`）

```json
{
  "mcpServers": {
    "coldrawdb": {
      "type": "stdio",
      "command": "/ABS/PATH/coldrawdb-mcp",
      "args": [],
      "env": { "COLDRAWDB_BASE_URL": "http://localhost:3000" }
    }
  }
}
```

等价 CLI：`claude mcp add --transport stdio --env COLDRAWDB_BASE_URL=http://localhost:3000 coldrawdb -- /ABS/PATH/coldrawdb-mcp`。

### 8.2 Codex（项目 `.codex/config.toml` 或用户配置）

```toml
[mcp_servers.coldrawdb]
command = "/ABS/PATH/coldrawdb-mcp"
args = []
env = { COLDRAWDB_BASE_URL = "http://localhost:3000" }
startup_timeout_sec = 10
tool_timeout_sec = 30
default_tools_approval_mode = "writes"
```

等价 CLI：`codex mcp add coldrawdb --env COLDRAWDB_BASE_URL=http://localhost:3000 -- /ABS/PATH/coldrawdb-mcp`。配置格式依据 OpenAI 官方 MCP 文档：<https://developers.openai.com/codex/mcp.md>。

### 8.3 Cursor（项目 `.cursor/mcp.json`）

```json
{
  "mcpServers": {
    "coldrawdb": {
      "type": "stdio",
      "command": "/ABS/PATH/coldrawdb-mcp",
      "args": [],
      "env": { "COLDRAWDB_BASE_URL": "http://localhost:3000" }
    }
  }
}
```

### 8.4 OpenCode（`opencode.json`）

```json
{
  "$schema": "https://opencode.ai/config.json",
  "mcp": {
    "coldrawdb": {
      "type": "local",
      "command": ["/ABS/PATH/coldrawdb-mcp"],
      "enabled": true,
      "environment": { "COLDRAWDB_BASE_URL": "http://localhost:3000" }
    }
  }
}
```

## 9. 官方兼容性依据（核验日期：2026-08-18）

- Claude Code：<https://code.claude.com/docs/en/mcp.md>（stdio/HTTP，项目 `.mcp.json`）
- Codex：<https://developers.openai.com/codex/mcp.md>（stdio/Streamable HTTP，`config.toml`）
- Cursor：<https://cursor.com/docs/mcp.md>（stdio/SSE/Streamable HTTP，`.cursor/mcp.json`）
- OpenCode：<https://opencode.ai/docs/mcp-servers/>（`type: local`、command 数组、environment）

---

## 合并自 fix-open-issues-26-33（2026-09-27）

## ADDED — §4.4 大图分批写入约定（fix-open-issues-26-33 / #26）

> 背景：MCP 客户端对单次工具调用参数体积有限制，大图全量 `update_diagram`（数十表 + 关系 + 布局/颜色）会触发「载荷太大」失败。本节约定不绕开 MCP 的可靠写入路径；MVP 不引入 partial update API 或新 batch 工具。

### 4.4.1 载荷软上限

- adapter 对 `update_diagram` 在**发送前**估算序列化参数体积（`{"id","expected_revision","diagram"}` JSON 字节数）。
- 软上限 `COLDRAWDB_MCP_PAYLOAD_SOFT_LIMIT_BYTES`，默认 **256 KiB**（262144），可用环境变量覆盖；下限 16 KiB（防配置误伤）。
- 未超软上限：按既有路径直写，不拦截、不告警。
- 超软上限：**不发送上游请求**，返回 `PAYLOAD_TOO_LARGE`（见 §4.3 错误表新增行），details 至少含：
  - `payload_bytes`：本次估算体积
  - `soft_limit_bytes`：生效上限
  - `suggestion`：固定文案，引导分批（「按表/字段/关系分片：update_table / update_field / update_reference，布局用 layout_diagram；每批携带最新 expected_revision 串行执行」）
  - `batch_hint`：`{ "tables": <表数>, "references": <关系数>, "suggested_batch": "update_table|update_reference" }`（按变更构成给主建议）

### 4.4.2 分批指引与 revision 串行规则

- 分批写入 = 多次细粒度写工具调用，每批都是一次完整的「GET 最新 → 修改 → 带 `expected_revision` 写回」闭环（adapter 内部对细粒度工具已是此语义，见 §4.2）。
- **串行规则**：同一 diagram 的分批必须串行——第 N 批成功返回新 revision 后，第 N+1 批才能以该 revision 发起；禁止并行分批（会必然 409）。
- **失败语义**：任一批失败不影响已成功批次；失败批的 diagram 当前 revision 保持可读（`get_diagram` 取最新），修正参数后以最新 revision 重试该批即可；不存在「半批」状态（每批自身是一次原子 PUT/op 提交）。
- 房间绑定图同样适用：细粒度工具经 op 通道时 revision 语义不变（§4.2）。

### 4.4.3 工具描述外露

- `update_diagram` 的 description 必须写明软上限默认值与「超限请分批」指引；四个细粒度写工具的 description 标注其作为大图分批写入的官方分片手段。

## MODIFIED — §4.3 错误体验（错误表追加行）

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
| `update_diagram` 参数体积超软上限（本地拦截，未发请求） | PAYLOAD_TOO_LARGE | false | 按 details.suggestion 分批写入；或调大 `COLDRAWDB_MCP_PAYLOAD_SOFT_LIMIT_BYTES`（自担客户端限制风险） |

错误结果使用 `isError: true`，structuredContent 至少包含 `code`、`message`、`retryable`，可选 `request_id`、`details`；不得包含 Authorization、Cookie、WebSocket URL 中的 token 或完整上游响应头。`PAYLOAD_TOO_LARGE` 属本地拦截，无 `request_id`。

## 合并自 fix-issues-42-44-mcp-and-relation-hit（2026-09-28）

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
