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

凭据有三种互斥模式：

- **静态 Token**：仅配置 `COLDRAWDB_ACCESS_TOKEN`，沿用 Bearer 与 WebSocket token 注入，不会尝试 refresh。
- **动态登录**：仅成对配置 `COLDRAWDB_AUTH_EMAIL` / `COLDRAWDB_AUTH_PASSWORD`。首次需要鉴权时通过既有 `POST /api/v1/auth/login` 建立进程内会话；业务 API 401 时最多 refresh 一次并重试原请求一次，轮换后的 refresh cookie 也只保存在内存。
- **匿名**：两者均未配置，可访问无需鉴权的 API；进入房间写回退时会明确返回 `ROOMS_AUTH_REQUIRED`，`details.reason=TOKEN_MISSING`。

静态 Token 与任一动态变量混配，或 email/password 只配置一项，都会在启动时返回 `CONFIG_INVALID`。密码只进入固定 login 请求体；access token 只进入 Authorization header 与既有 WebSocket 握手 query；refresh cookie 只进入固定 refresh 请求。不得把真实凭据写入仓库，也不要求复制 httpOnly refresh token。

`COLDRAWDB_REQUEST_TIMEOUT_SECS` 可选，默认 30 秒，允许 1～120 秒。模板中的动态凭据空值表示匿名占位；实际使用时通过客户端允许的安全环境变量机制注入，不要提交真实值。

可复制模板位于：

- `examples/claude.mcp.json`
- `examples/codex.config.toml`
- `examples/cursor.mcp.json`
- `examples/opencode.json`

把模板中的 `/ABS/PATH/coldrawdb-mcp` 替换为 release 二进制绝对路径。四套模板均连接同一个 stdio 服务；无需维护客户端专属业务实现。

## 工具

当前提供 12 个工具，契约见 [`mcp-tools.yaml`](../logos/resources/api/mcp-tools.yaml)。

- 读取：`list_diagrams`、`get_diagram`、`export_schema`
- 写入：`create_diagram`、`update_diagram`、`delete_diagram`、`import_schema`
- 画布编辑：`update_table`、`update_field`、`update_reference`、`layout_diagram`
- 数据字典：`update_dictionary`

`update_diagram` 必须携带最新 `expected_revision`；修订号冲突返回 `REVISION_CONFLICT`，不会自动覆盖。`delete_diagram` 同时具有 `destructiveHint: true` 和本地 `confirm=true` 双重约束。

更新房间图时，若 HTTP PUT 返回 `USE_OP_CHANNEL`，服务会查找绑定的活跃房间、核对 revision，再将文档差异转为 WebSocket 操作，等待服务端确认。该路径同样用于画布编辑工具的保存。房间写入需要有效凭据和房间写权限：rooms 或 WebSocket 最终 401 返回 `ROOMS_AUTH_REQUIRED / TOKEN_INVALID_OR_EXPIRED`；403、只读成员错误和 5xx 保留原语义。动态模式下，仅 WebSocket 握手 401 且尚未发送 op 时允许 refresh 并重连一次；op 已发送后的失败不会自动重放。

## 大图无损分片

特定客户端实测单次工具参数约在 63 KB 附近出现边界；这不是 MCP 协议或服务端的统一限制。为降低客户端差异风险，建议每次工具参数保守控制在约 48 KiB。`update_diagram` 的 `COLDRAWDB_MCP_PAYLOAD_SOFT_LIMIT_BYTES` 仍默认为 256 KiB，48 KiB 只是分片建议，不会成为新的本地拒绝阈值。

四套客户端共用同一写入顺序：

1. 用 `create_diagram` 创建空图；复杂既有模型也可先用 `import_schema`，随后重新读取实际 ID。
2. 对每张表串行调用 `update_table(action=create)`，一次传入完整 `fields`。服务会为表和字段生成冲突安全的 `auto-` 实际 ID；保存响应中的 `table_id` 与 `field_id_map`，不要继续用调用方旧 ID。
3. 仅用实际 ID 构造关系，串行调用 `update_reference(action=create, references=[...])`，按数组控制每片大小。外部 `cardinality` 会写入后端 `type_`，级联动作规范化为大写下划线。
4. 后续可继续使用 `update_field`、`layout_diagram` 等细粒度工具。

非房间的单个整表或关系批次使用一次快照 PUT；房间整表对应一个 `table.create` op，关系批次对应多个逐条确认的 op。跨表、跨关系批和跨调用没有整体事务或补偿回滚，也不保证 revision 只增加 1。失败时重新读取最新 revision，只修正失败批；已成功的写入和已确认 op 不会自动撤销。

## 当前边界

客户端接入仅提供本地 stdio 传输，不监听 MCP HTTP 端口。房间协作使用后端 WebSocket，不改变客户端的 stdio 接入方式。当前图表 API 尚未统一强制 JWT，因此只允许连接可信的本地或内网后端；不得将该服务包装后直接暴露到公网。
