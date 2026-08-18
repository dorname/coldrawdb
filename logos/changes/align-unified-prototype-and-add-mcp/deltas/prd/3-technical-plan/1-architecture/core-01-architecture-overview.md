# Delta — core-01-architecture-overview.md（S06 架构增量）

> module: core | proposal: align-unified-prototype-and-add-mcp

## ADDED — MCP adapter 边界

```text
Claude / Codex / Cursor / OpenCode
              │ MCP stdio（JSON-RPC）
              ▼
       coldrawdb-mcp（新 Rust 服务）
       ├─ protocol/tool schema
       ├─ config + redaction
       ├─ fixed-path HTTP client
       └─ pure export serializer
              │ HTTP + optional Bearer
              ▼
       既有 coldrawdb backend
              │ SeaORM / SQL
              ▼
             SQLite
```

### 模块职责

| 模块 | 职责 | 禁止 |
|---|---|---|
| MCP protocol | initialize、tools/list、tools/call、annotations | 业务持久化 |
| config | BASE_URL/Token/timeout 校验、secret redaction | 把 Token 写入日志 |
| HTTP adapter | 只调用白名单 diagram 路径、错误映射 | 任意 URL/method/header |
| export serializer | Diagram → JSON/DBML/SQL 纯函数 | 网络、文件、数据库副作用 |
| backend | revision、事务、持久化 | 由 MCP 绕过 |

### 依赖方向

`protocol → application tools → HTTP port`；具体 HTTP client 实现依赖 port。export serializer 是纯领域服务。MCP crate 不依赖 `backend` crate、SeaORM 或 sqlite，以编译依赖和安全测试固定这一边界。

### 当前认证事实

auth/rooms/collab 已实现，但 `/api/v1/diagrams*` 尚未挂 JWT middleware。`COLDRAWDB_ACCESS_TOKEN` 仅做 header 透传，为后续兼容保留；MVP 的实际保护来自本地 stdio 与 backend 网络边界。Streamable HTTP 在权限前置未完成前禁止部署。

### 技术选型

- Rust MCP SDK：实现阶段选择与项目 toolchain 兼容、支持 stdio 和 tool annotations 的版本；若采用 `rmcp 3.1.x`，最低 Rust 版本需在构建 delta 中显式提升并验证。
- HTTP：异步 client，默认 30s timeout，不对写请求自动重试。
- 序列化：serde/serde_json；工具契约以 `mcp-tools.yaml` 为源。
- 测试：mock HTTP + 真实 stdio client + 隔离 backend orchestration。

