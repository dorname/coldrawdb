## ADDED — V1 部署方案

> 模块：core | 提案：add-baseline-docs
> 路径：`logos/resources/prd/3-technical-plan/3-deployment/core-01-deployment-plan.md`
> 对齐参考源：`core-01-architecture-overview.md` §7 + Phase 4 CI 配置 + 11 张表 DDL

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

```bash
# 1. 启动后端
cd backend
cargo run                    # 监听 :3000

# 2. 启动前端（另一终端）
cd frontend-rs
trunk serve                  # 监听 :8080

# 3. 访问
open http://localhost:8080/editor
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
