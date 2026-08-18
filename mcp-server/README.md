# coldrawdb MCP 服务

`coldrawdb-mcp` 是独立的本地 stdio adapter，让 Claude、Codex、Cursor 和 OpenCode 通过同一组 MCP 工具访问 coldrawdb。它只调用固定的 diagram HTTP API，不直接访问 SQLite，也不提供任意 SQL、shell、文件或通用 HTTP 工具。

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

可选配置为 `COLDRAWDB_ACCESS_TOKEN` 和 `COLDRAWDB_REQUEST_TIMEOUT_SECS`。Token 只能经环境变量或客户端安全配置注入，不得写入仓库。

可复制模板位于：

- `examples/claude.mcp.json`
- `examples/codex.config.toml`
- `examples/cursor.mcp.json`
- `examples/opencode.json`

把模板中的 `/ABS/PATH/coldrawdb-mcp` 替换为 release 二进制绝对路径。四套模板均连接同一个 stdio 服务；无需维护客户端专属业务实现。

## 工具

- 读取：`list_diagrams`、`get_diagram`、`export_schema`
- 写入：`create_diagram`、`update_diagram`、`delete_diagram`、`import_schema`

`update_diagram` 必须携带最新 `expected_revision`；409 会返回 `REVISION_CONFLICT`，不会自动覆盖。`delete_diagram` 同时具有 `destructiveHint: true` 和本地 `confirm=true` 双重约束。

## 当前边界

MVP 仅提供本地 stdio transport，不监听 MCP HTTP 端口。当前 diagram API 尚未强制 JWT，因此只允许连接可信的本地或内网 backend；不得将该服务包装后直接暴露到公网。
