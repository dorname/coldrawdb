# V1 部署方案（How 层 — 第 3 步：部署）

## 1. 部署目标

| 目标 | 描述 |
|---|---|
| 本地开发 | 开发者本机双进程（trunk serve + cargo run） |
| Docker | 单镜像包含 frontend 静态资源 + backend 二进制 |
| Staging | 单机 Docker Compose；模拟生产链路 |
| Production | **V1 不实现**（仅 staging） |

> **V1 关键边界**：无生产部署目标。V1 部署 = 开发者机器 + staging 单机。完整生产部署待 V2 引入 OT 后端服务后重新设计。

## 2. 环境矩阵

| 维度 | 本地 dev | Docker | Staging |
|---|---|---|---|
| 操作系统 | Linux / macOS / WSL2 | 任意 | Linux (Debian 12) |
| 前端 | `trunk serve` 8080 | nginx 静态 | nginx 静态 |
| 后端 | `cargo run` 3000 | actix-web in container 3000 | actix-web in container 3000 |
| 数据库 | SQLite 文件 | SQLite 文件（volume） | SQLite 文件（volume） |
| 反向代理 | 无 | 无 | nginx |
| TLS | 无 | 无 | 暂未启用 |
| 鉴权 | 无 | 无 | 无 |
| 监控 | console | docker logs | docker logs + JSON 日志收集 |
| 数据备份 | 无 | 无 | 每日 cron 拷贝 SQLite |

## 3. 本地 dev 部署

### 3.1 启动顺序

推荐方式（一键脚本）：

```bash
# 1. 从仓库根目录启动前后端
./scripts/start-local.sh

# 2. 访问
open http://localhost:8080/editor
```

手动方式（需要两个终端）：

```bash
# 终端 1：启动后端
cd backend
cargo run --release          # 监听 :3000

# 终端 2：启动前端
cd frontend-rs
trunk serve --port 8080      # 监听 :8080
```

### 3.2 数据初始化

- 首次启动后端时自动运行 `backend/init.sql`（含 11 张表 DDL）
- 配置文件：`backend/config.toml`

```toml
# backend/config.toml
[server]
host = "0.0.0.0"
port = 3000

[database]
url = "sqlite://data/coldrawdb.db?mode=rwc"

[logging]
level = "info"
format = "json"
```

### 3.3 依赖

| 工具 | 最低版本 | 用途 |
|---|---|---|
| Rust | 1.75 | 编译前后端 |
| trunk | 0.20 | 打包 WASM |
| wasm32 target | — | `rustup target add wasm32-unknown-unknown` |
| SQLite | 3.40+ | WAL 模式（默认启用） |
| Node.js | (非必须) | trunk 内部用 |

### 3.4 使用本地启动脚本

为简化本地开发，仓库根目录提供 `scripts/start-local.sh` 和 `scripts/stop-local.sh`。

#### 3.4.1 脚本位置

```
scripts/
├── start-local.sh    # 启动后端 + 前端
├── stop-local.sh     # 停止由 start-local.sh 启动的进程
└── common.sh         # 公共函数（日志、PID、端口检测）
```

#### 3.4.2 启动脚本行为

`start-local.sh` 默认行为：

1. 检查依赖：`cargo`、`trunk`、Rust wasm32 target 是否可用
2. 检查端口占用：后端默认 `3000`，前端默认 `8080`
3. 创建 `logs/` 目录
4. 预编译后端：`cd backend && cargo build --release`，输出重定向到 `logs/backend.log`（避免冷机首次编译挤占健康检查窗口）
5. 启动后端：`cd backend && ./target/release/backend`，输出重定向到 `logs/backend.log`
6. 等待后端健康检查通过（`GET /api/v1/diagrams/health` 或 `GET /`）；若超时且日志显示仍在编译，脚本给出针对性提示
7. 启动前端：`cd frontend-rs && trunk serve --port 8080`，输出重定向到 `logs/frontend.log`
8. 将前后端 PID 写入 `logs/backend.pid` 与 `logs/frontend.pid`
9. 输出访问地址与停止命令

启动示例：

```bash
./scripts/start-local.sh
# 自定义前端端口（后端端口固定为 backend/config.toml 的 3000）
COLDRAWDB_FRONTEND_PORT=8081 ./scripts/start-local.sh
```

> 注意：`COLDRAWDB_BACKEND_PORT` 仅影响脚本侧的端口占用检查与健康检查地址，不改变后端实际绑定端口（固定由 `backend/config.toml` 的 `server.port = 3000` 决定）。显式将其设为非 3000 会导致健康检查轮询错误端口。

#### 3.4.3 停止脚本行为

`stop-local.sh` 默认行为：

1. 读取 `logs/backend.pid` 与 `logs/frontend.pid`
2. 先向前端进程发送 `TERM` 信号
3. 再向后端进程发送 `TERM` 信号
4. 等待最多 10 秒，进程仍未退出则发送 `KILL`
5. 删除 PID 文件

停止示例：

```bash
./scripts/stop-local.sh
```

#### 3.4.4 环境变量

| 变量 | 默认值 | 说明 |
|---|---|---|
| `COLDRAWDB_BACKEND_PORT` | `3000` | 脚本侧端口占用检查与健康检查地址所用端口；后端实际绑定端口固定由 `backend/config.toml` 决定（3000） |
| `COLDRAWDB_FRONTEND_PORT` | `8080` | 前端 trunk serve 端口 |
| `COLDRAWDB_BACKEND_LOG` | `logs/backend.log` | 后端日志路径（相对仓库根目录） |
| `COLDRAWDB_FRONTEND_LOG` | `logs/frontend.log` | 前端日志路径 |
| `COLDRAWDB_BACKEND_PID` | `logs/backend.pid` | 后端 PID 文件路径 |
| `COLDRAWDB_FRONTEND_PID` | `logs/frontend.pid` | 前端 PID 文件路径 |
| `COLDRAWDB_HEALTH_TIMEOUT` | `120` | 等待后端健康检查超时秒数 |

#### 3.4.5 日志与 PID

- 所有脚本日志统一输出到 `logs/` 目录
- 运行产生的 `logs/`、`*.pid` 已加入 `.gitignore`，不会被提交
- 如需排查启动失败，查看 `logs/backend.log` 与 `logs/frontend.log`

#### 3.4.5 日志与 PID

- 所有脚本日志统一输出到 `logs/` 目录
- 运行产生的 `logs/`、`*.pid` 已加入 `.gitignore`，不会被提交
- 如需排查启动失败，查看 `logs/backend.log` 与 `logs/frontend.log`

#### 3.4.6 WASM 缓存策略（Monaco 启用后，redesign-phase-e E4）

启用 Monaco Editor（`core-0a-code-editor.md`）后，前端 WASM 总体积约 +3 MB（gzipped）。为避免重复下载与首屏延迟，部署方案要求：

| 资源 | Cache-Control | 说明 |
|---|---|---|
| `*.wasm` / `editor*.js` | `public, max-age=31536000, immutable` | trunk 打包文件名带 hash，永久缓存 |
| `monaco-editor/*` chunk | `public, max-age=2592000, immutable`（30 天） | Monaco 语言包按需 lazy-load |
| `index.html` | `no-cache` | SPA 入口必须每次校验更新 |

nginx 配置示例（在 `nginx.conf` 的 `location /` 中）：

```nginx
location ~* \.(wasm|js)$ {
  add_header Cache-Control "public, max-age=31536000, immutable";
}
location ~* /monaco-editor/ {
  add_header Cache-Control "public, max-age=2592000, immutable";
}
location = / {
  add_header Cache-Control "no-cache";
}
```

Docker 镜像层复用：`Dockerfile` 的 wasm-build 阶段产物 `dist/` 在镜像 tag 不变时复用率约 95%，多 staging 间共享层可显著降低带宽。

## 4. Docker 部署

### 4.1 镜像构建

```dockerfile
# 多阶段构建
FROM rust:1.75-bookworm AS wasm-build
WORKDIR /app
RUN cargo install trunk wasm-bindgen-cli
RUN rustup target add wasm32-unknown-unknown
COPY frontend-rs ./frontend-rs
COPY index.html ./
COPY styles ./styles
RUN trunk build --release

FROM rust:1.75-bookworm AS api-build
WORKDIR /app
COPY backend ./backend
COPY Cargo.toml Cargo.lock ./
RUN cargo build --release -p backend

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y libsqlite3-0 ca-certificates
COPY --from=api-build /app/target/release/backend /usr/local/bin/backend
COPY --from=wasm-build /app/dist /var/www/
RUN mkdir -p /data
EXPOSE 3000
CMD ["backend"]
```

### 4.2 运行

```bash
docker build -t coldrawdb:v1 .
docker run -d \
  -p 3000:3000 \
  -v $(pwd)/data:/data \
  -e COLDRAWDB_DB_URL="sqlite:///data/coldrawdb.db?mode=rwc" \
  --name coldrawdb \
  coldrawdb:v1
```

> 容器内 backend 同时服务 WASM 静态文件（`/var/www`）和 API（`/api/v1/*`）。

### 4.3 配置（环境变量）

| 变量 | 默认 | 说明 |
|---|---|---|
| `COLDRAWDB_DB_URL` | `sqlite:///data/coldrawdb.db?mode=rwc` | SQLite 连接字符串 |
| `COLDRAWDB_LOG_LEVEL` | `info` | `trace` / `debug` / `info` / `warn` / `error` |
| `COLDRAWDB_BIND_ADDR` | `0.0.0.0:3000` | 后端绑定地址 |
| `COLDRAWDB_STATIC_DIR` | `/var/www` | 静态文件目录 |

## 5. Staging 部署

### 5.1 Docker Compose

```yaml
# docker-compose.yml
version: "3.9"
services:
  coldrawdb:
    image: coldrawdb:v1
    ports:
      - "3000:3000"
    volumes:
      - ./data:/data
      - ./logs:/logs
    environment:
      COLDRAWDB_DB_URL: "sqlite:///data/coldrawdb.db?mode=rwc"
      COLDRAWDB_LOG_LEVEL: "info"
    restart: unless-stopped
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:3000/api/v1/diagrams/health"]
      interval: 30s
      timeout: 5s
      retries: 3

  nginx:
    image: nginx:1.25
    ports:
      - "80:80"
    volumes:
      - ./nginx.conf:/etc/nginx/nginx.conf:ro
    depends_on:
      - coldrawdb

  backup:
    image: alpine:3.19
    volumes:
      - ./data:/data:ro
      - ./backups:/backups
    command: >
      sh -c "
        apk add --no-cache sqlite tar &&
        while true; do
          tar czf /backups/coldrawdb-$$(date +%Y%m%d-%H%M%S).tar.gz -C /data .;
          sleep 86400;
        done
      "
    restart: unless-stopped
```

### 5.2 nginx 反代

```nginx
# nginx.conf
events { worker_connections 1024; }
http {
  upstream backend { server coldrawdb:3000; }
  server {
    listen 80;
    location / { proxy_pass http://backend; }
    location /api/ { proxy_pass http://backend; }
  }
}
```

### 5.3 数据备份

- 每日 cron 打包 SQLite + 上传到对象存储
- 保留最近 7 天 + 每周一次月度快照
- 恢复：`tar xzf backup.tar.gz -C data/`

## 6. Smoke 测试入口

| 入口 | 描述 | 状态 |
|---|---|---|
| `GET /api/v1/diagrams/health` | 健康检查 | V1 实现 |
| `GET /` | 加载 index.html | V1 实现 |
| `GET /editor` | 加载编辑器 | V1 实现 |
| `POST /api/v1/diagrams` | 冒烟创建 diagram | 详见 smoke 测试用例 |

详细 smoke 用例见 `deltas/test/smoke/core-smoke-test-cases.md`。

## 7. 监控与日志

### 7.1 日志

- 格式：JSON（`tracing` + `tracing-subscriber`）
- 字段：`timestamp` / `level` / `target` / `message` / `request_id` / `user_id`(V1 空)
- 存储：`/logs/coldrawdb.log`（容器内）
- 轮转：logrotate 每日

### 7.2 指标

V1 **不实现** metrics endpoint（V2 计划）。仅日志。

### 7.3 告警

V1 **不实现**告警。仅人工巡检。

## 8. 安全

V1 部署**不包含**：
- TLS（明文 HTTP）
- 鉴权（所有 API 公开）
- 限流
- CSRF 防护
- 审计日志

> 安全强化待 V2 引入用户系统后统一设计。

## 9. 部署检查清单

部署到 staging 前：

- [ ] 镜像构建成功（`docker build`）
- [ ] 11 张表 DDL 在 staging DB 已生效
- [ ] `GET /api/v1/diagrams/health` 返回 200
- [ ] `GET /` 返回 index.html（< 1 MB）
- [ ] 创建 diagram E2E 通过（smoke 用例 SMOKE-core-01）
- [ ] 分享链接 E2E 通过（smoke 用例 SMOKE-core-02）
- [ ] 导入导出 E2E 通过（smoke 用例 SMOKE-core-03）
- [ ] 日志输出 JSON 格式正确
- [ ] 数据备份 cron 已配置

## 10. V1 边界

- ❌ Kubernetes 部署（V1 单机 docker compose）
- ❌ 生产 TLS 证书（V1 staging 明文 HTTP）
- ❌ 鉴权 / 限流 / CSRF（V1 完全公开）
- ❌ 水平扩展（V1 单实例；SQLite 不支持）
- ❌ 蓝绿发布（V1 滚动重启）
- ❌ Prometheus 指标（V1 仅日志）

## 11. 对齐参考源

- `core-01-architecture-overview.md`（系统上下文 + 部署拓扑）
- `RUST_WEB_REFACTOR_PLAN.md`（仓库根；部署章节）
- `backend/Cargo.toml`（依赖版本）
- `backend/init.sql`（11 张表 DDL）
- `.github/workflows/ci.yml`（CI 配置，参考部署镜像构建）
- `docs/phase4/PHASE4_DONE.md`（WASM 产物路径）
- `docs/drawdb-capability-checklist.md` §5

## MCP stdio 分发

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
