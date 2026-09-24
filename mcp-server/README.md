# coldrawdb MCP 服务

`coldrawdb-mcp` 是独立的本地 stdio 服务，让 Claude、Codex、Cursor 和 OpenCode 通过同一组 MCP 工具访问 coldrawdb。它通过固定的后端 HTTP API 与房间 WebSocket 读写图表，不直接访问 SQLite，也不提供任意 SQL、shell、文件或通用 HTTP 工具。

## 构建

```bash
./scripts/build-mcp.sh
```

构建产物位于 `mcp-server/target/release/coldrawdb-mcp`。客户端配置必须使用该文件的绝对路径。

## 配置

必须提供：

```text
COLDRAWDB_BASE_URL=http://localhost:3000
```

通过 Compose 的 nginx 入口访问时，可将基址改为 `http://localhost:9080`；基址不包含 `/api/v1`。

`COLDRAWDB_ACCESS_TOKEN` 用于后端鉴权，房间图协作写入需要有效 Token 和房间写入权限。`COLDRAWDB_REQUEST_TIMEOUT_SECS` 可选，默认 30 秒，允许 1～120 秒。Token 只能经环境变量或客户端安全配置注入，不得写入仓库。

可复制模板位于：

- `examples/claude.mcp.json`
- `examples/codex.config.toml`
- `examples/cursor.mcp.json`
- `examples/opencode.json`

把模板中的 `/ABS/PATH/coldrawdb-mcp` 替换为 release 二进制绝对路径。四套模板均连接同一个 stdio 服务；无需维护客户端专属业务实现。

## 工具

当前提供 11 个工具，契约见 [`mcp-tools.yaml`](../logos/resources/api/mcp-tools.yaml)。

- 读取：`list_diagrams`、`get_diagram`、`export_schema`
- 写入：`create_diagram`、`update_diagram`、`delete_diagram`、`import_schema`
- 画布编辑：`update_table`、`update_field`、`update_reference`、`layout_diagram`

`update_diagram` 必须携带最新 `expected_revision`；修订号冲突返回 `REVISION_CONFLICT`，不会自动覆盖。`delete_diagram` 同时具有 `destructiveHint: true` 和本地 `confirm=true` 双重约束。

更新房间图时，若 HTTP PUT 返回 `USE_OP_CHANNEL`，服务会查找绑定的活跃房间、核对 revision，再将文档差异转为 WebSocket 操作，等待服务端确认。该路径同样用于画布编辑工具的保存。

## 当前边界

客户端接入仅提供本地 stdio 传输，不监听 MCP HTTP 端口。房间协作使用后端 WebSocket，不改变客户端的 stdio 接入方式。当前图表 API 尚未统一强制 JWT，因此只允许连接可信的本地或内网后端；不得将该服务包装后直接暴露到公网。
