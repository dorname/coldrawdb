# Delta — core-01-deployment-plan.md（S06 MCP 部署增量）

> module: core | proposal: align-unified-prototype-and-add-mcp

## ADDED — MCP stdio 分发

### 构建产物

`coldrawdb-mcp` 是独立 release 二进制，不监听网络端口。构建版本必须记录 Git commit、Rust toolchain、MCP SDK 版本；四客户端只配置同一绝对路径。

### 启动前置

1. coldrawdb backend 已在 `COLDRAWDB_BASE_URL` 运行。
2. 客户端进程具有执行 `coldrawdb-mcp` 的权限。
3. 如使用 `COLDRAWDB_ACCESS_TOKEN`，通过环境变量或客户端安全配置注入，不写入仓库。
4. stdout 未被 shell wrapper、banner 或日志污染。

### 环境矩阵

| 环境 | MCP transport | backend | 凭据 | 允许 |
|---|---|---|---|---|
| 本地 | stdio | localhost | 可选 | 是 |
| 测试 | stdio | 隔离 backend | 测试 Token，可选 | 是 |
| 预发 | stdio | 内网 backend | 安全注入 | 需用户授权 |
| 公网生产 | 无 | — | — | 本次禁止 |

### Smoke

1. initialize 成功，serverInfo.name=`coldrawdb-mcp`。
2. tools/list 恰好七个工具，delete destructiveHint=true。
3. `list_diagrams` 与 `get_diagram` 成功，stderr 无 Token，stdout 只有 JSON-RPC。
4. 写 smoke 默认不执行；若获额外批准，使用专用临时 diagram 并在同次 smoke 删除。

### 回滚

- 从客户端配置移除/禁用 `coldrawdb` MCP 项。
- 停止由客户端托管的 stdio 子进程。
- 移除或回退二进制；不需要数据库 migration 回滚。
- 不修改 backend 时，Web/API 服务继续运行，不受 MCP 回滚影响。

### 后续远程部署门槛

只有 diagram API 强制 S03 鉴权、完成细粒度授权和安全评审后，才可另案增加 Streamable HTTP、Bearer/OAuth、TLS、速率限制与远程审计。本 delta 不提供 HTTP MCP 监听器。
