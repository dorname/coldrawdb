# Delta: core-01-deployment-plan.md

> 提案：feat-docker-compose-deploy

## MODIFIED — 4. Docker 部署

> 更新于 feat-docker-compose-deploy（2026-09-10）：本节从「设计」转为「实现落地」——根 `Dockerfile` 已替换为下方 Rust 多阶段构建（旧 JS 栈 Dockerfile 已移除）；后端已实现静态文件服务 + SPA 回源 + health 端点 + `COLDRAWDB_*` 环境变量覆盖。

### 4.1 镜像构建

```dockerfile
# 多阶段构建（trunk 入口为 frontend-rs/index.html，dist 产物在 frontend-rs/dist）
FROM rust:1.83-bookworm AS wasm-build
WORKDIR /app
RUN cargo install --locked trunk wasm-bindgen-cli
RUN rustup target add wasm32-unknown-unknown
COPY frontend-rs ./frontend-rs
RUN cd frontend-rs && trunk build --release

FROM rust:1.83-bookworm AS api-build
WORKDIR /app
COPY backend ./backend
RUN cargo build --release --manifest-path backend/Cargo.toml

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y libsqlite3-0 ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=api-build /app/backend/target/release/backend /usr/local/bin/backend
COPY --from=api-build /app/backend/config.toml /etc/coldrawdb/config.toml
COPY --from=wasm-build /app/frontend-rs/dist /var/www/
RUN mkdir -p /data
ENV COLDRAWDB_BIND_ADDR="0.0.0.0:3000" \
    COLDRAWDB_STATIC_DIR="/var/www" \
    COLDRAWDB_DB_URL="sqlite:///data/coldrawdb.db?mode=rwc"
EXPOSE 3000
CMD ["backend"]
```

> 容器内 backend 同时服务 WASM 静态文件（`COLDRAWDB_STATIC_DIR`，默认 `/var/www`）和 API（`/api/v1/*`）；非 API 前端路由回源 `index.html`（§12.3）；`*.wasm`/`*.js` 响应头 `Cache-Control: public, max-age=31536000, immutable`，`index.html` 为 `no-cache`（§3.4.6）。

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

### 4.3 配置（环境变量）

| 变量 | 默认 | 说明 |
|---|---|---|
| `COLDRAWDB_DB_URL` | `sqlite:///data/coldrawdb.db?mode=rwc` | SQLite 连接字符串（覆盖 config.toml `[database]`） |
| `COLDRAWDB_LOG_LEVEL` | `info` | `trace` / `debug` / `info` / `warn` / `error` |
| `COLDRAWDB_BIND_ADDR` | `0.0.0.0:3000` | 后端绑定地址（覆盖 config.toml `[server]`；容器内必须 0.0.0.0） |
| `COLDRAWDB_STATIC_DIR` | `/var/www` | 静态文件目录；目录不存在或为空时后端退化为纯 API 模式（`GET /` 返回 200 存活文案，不 panic） |

## MODIFIED — 5. Staging 部署

> 更新于 feat-docker-compose-deploy（2026-09-10）：`docker-compose.yml` 与 `nginx.conf` 已落地仓库根目录；旧 `compose.yml`（dev 挂载栈）与 `vercel.json` 已删除。

### 5.1 Docker Compose

仓库根 `docker-compose.yml`（staging 形态）：

```yaml
services:
  coldrawdb:
    image: coldrawdb:v1
    build: .
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
      coldrawdb:
        condition: service_healthy

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

仓库根 `nginx.conf`：`/api/` 与 `/ws/` 透传后端，其余回源后端静态服务（后端对非 API 路径回源 `index.html`）：

```nginx
events { worker_connections 1024; }
http {
  upstream backend { server coldrawdb:3000; }
  server {
    listen 80;
    location /api/ { proxy_pass http://backend; }
    location /ws/ { proxy_pass http://backend; proxy_http_version 1.1; proxy_set_header Upgrade $http_upgrade; proxy_set_header Connection "upgrade"; }
    location / { proxy_pass http://backend; }
  }
}
```

### 5.3 数据备份

- 每日 cron 打包 SQLite + 上传到对象存储
- 保留最近 7 天 + 每周一次月度快照
- 恢复：`tar xzf backup.tar.gz -C data/`

## MODIFIED — 6. Smoke 测试入口

| 入口 | 描述 | 状态 |
|---|---|---|
| `GET /api/v1/diagrams/health` | 健康检查（200 + `{"status":"ok"}`） | V1 实现（feat-docker-compose-deploy 落地） |
| `GET /` | 加载 index.html（静态服务 + SPA 回源） | V1 实现（feat-docker-compose-deploy 落地） |
| `GET /editor` | 加载编辑器（回源 index.html） | V1 实现（feat-docker-compose-deploy 落地） |
| `POST /api/v1/diagrams` | 冒烟创建 diagram | 详见 smoke 测试用例 |

详细 smoke 用例见 `logos/resources/test/smoke/core-smoke-test-cases.md`。

## MODIFIED — 9. 部署检查清单

部署到 staging 前：

- [ ] 镜像构建成功（`docker build`；CI docker.yml PR/push 阶段 build-only 验证防腐坏）
- [ ] DB schema 由 migration 自动执行（首次启动后端时）
- [ ] `GET /api/v1/diagrams/health` 返回 200 `{"status":"ok"}`
- [ ] `GET /` 返回 index.html（< 1 MB）
- [ ] 创建 diagram E2E 通过（smoke 用例 SMOKE-core-01）
- [ ] 分享链接 E2E 通过（smoke 用例 SMOKE-core-02）
- [ ] 导入导出 E2E 通过（smoke 用例 SMOKE-core-03）
- [ ] 日志输出 JSON 格式正确
- [ ] 数据备份 cron 已配置
- [ ] `PUBLIC_BASE_URL` 已配置，或 Host 推导在反代下含正确 scheme 与端口（`Forwarded` / `X-Forwarded-Proto` 已透传）
- [ ] 生成邀请链接跨机可打开：`/invite/{token}` 页面 200 且 `GET /api/v1/rooms/invites/{token}` preview 200
