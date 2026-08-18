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
- `delete_diagram` 必须显式提供 `confirm: true`；这不能替代客户端审批，只是服务侧第二道防误触。
- `import_schema` MVP 仅接受 `format: "drawdb_json"`，传给 API 的 payload 必须为 JSON object。

### 4.3 错误体验

| 条件 | MCP 错误码 | retryable | 用户动作 |
|---|---|---|---|
| 配置缺失/URL 非 http(s) | CONFIG_INVALID | false | 修正客户端配置并重启 |
| 连接失败/超时 | UPSTREAM_UNAVAILABLE / UPSTREAM_TIMEOUT | true | 启动 backend 或检查 URL |
| 400/422 | VALIDATION_ERROR | false | 根据 details 修正参数 |
| 401 | UNAUTHENTICATED | false | 更新 Access Token |
| 403 | PERMISSION_DENIED | false | 使用有权限的身份 |
| 404 | NOT_FOUND | false | 重新列出 diagram |
| 409 | REVISION_CONFLICT | false | 读取最新 diagram，人工合并后重试 |
| 5xx | UPSTREAM_ERROR | true | 稍后重试并使用 request_id 排查 |

错误结果使用 `isError: true`，structuredContent 至少包含 `code`、`message`、`retryable`，可选 `request_id`、`details`；不得包含 Authorization、Cookie 或完整上游响应头。

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

stderr 结构化字段：`event`、`tool`、`duration_ms`、`status`、`upstream_status`、`request_id`。禁止记录工具完整输入中的 diagram 内容；仅记录 diagram_id、revision、表/字段计数。stdout 仅允许 MCP 帧。

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

