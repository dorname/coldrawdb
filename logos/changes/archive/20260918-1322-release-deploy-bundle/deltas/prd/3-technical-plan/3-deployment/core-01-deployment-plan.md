# Delta: core-01-deployment-plan.md — release-deploy-bundle（稳定版交付物改为 deploy.zip）

## MODIFIED — 2.1 稳定版交付（Windows / macOS / Linux）

### 2.1 稳定版交付（Windows / macOS / Linux）

稳定版用户侧交付物为 **Docker 镜像 + Compose**，不是原生桌面安装包。

| 客户端 OS | 推荐运行时 | 获取方式 | 访问入口 |
|---|---|---|---|
| Windows | Docker Desktop（WSL2 后端） | 下载 Release 资产 `coldrawdb-<tag>-deploy.zip` → 解压 → `docker compose up -d` | 浏览器 `http://localhost:9080/`（nginx 容器 :80 → 宿主机 **9080**） |
| macOS | Docker Desktop（Intel / Apple Silicon） | 同上；镜像含 `linux/arm64` | 同上 |
| Linux | Docker Engine + Compose v2 | 同上 | 同上 |

本地 Rust/trunk 双进程（§3）仅开发路径；原生 Windows 无 WSL 时必须用 Docker。

`coldrawdb-<tag>-deploy.zip` 内含（由 `.github/workflows/release.yml` 打包）：

- `compose.yml` — release 形态 Compose（仓库根 `compose.release.yml`，引用预构建镜像，无本地 `build:` 步骤）
- `nginx.conf` — 反代配置（nginx 容器挂载依赖）
- `.env.sample` — 环境变量示例
- `快速上手.md` — 解压 → `docker compose up -d` → 打开 `http://localhost:9080/`

版本标签约定：

- SemVer annotated tag：`vMAJOR.MINOR.PATCH`（首个稳定版 `v0.1.0`）
- Tag 推送触发：
  1. `.github/workflows/docker.yml` → `ghcr.io/<owner>/coldrawdb:<tag>` + `:latest`，平台 `linux/amd64,linux/arm64`
  2. `.github/workflows/release.yml` → GitHub Release + `coldrawdb-<tag>-deploy.zip`（**不再上传源码 zip**；GitHub 自动生成的 `Source code (zip/tar.gz)` 归档非正式交付物，发布说明中注明）

跨机邀请：设置 `PUBLIC_BASE_URL` 为公开 SPA 入口（Compose 默认 `http://localhost:9080`，禁止默认裸后端 `:3000`）。

**宿主机端口**：默认 **9080**，避免 Windows / macOS 上 80 常被占用。可通过环境变量 `COLDRAWDB_HTTP_PORT` 覆盖映射（如 `COLDRAWDB_HTTP_PORT=80 docker compose up -d`）。

**稳定版 Compose 形态说明**：稳定版部署使用 release 形态 Compose（引用预构建镜像 `ghcr.io/<owner>/coldrawdb:<tag>`，无 `build:` 段）；仓库根 `docker-compose.yml` 保留为 staging / 开发形态（本地构建 + `--build`），不随发布交付。

## MODIFIED — 5. Staging 部署

## 5. Staging 部署

> 更新于 feat-docker-compose-deploy（2026-09-10）：`docker-compose.yml` 与 `nginx.conf` 已落地仓库根目录；旧 `compose.yml`（dev 挂载栈）与 `vercel.json` 已删除。
> 更新于 release-deploy-bundle（2026-09-18）：仓库根 `compose.release.yml` 为稳定版 release 形态 Compose（引用 GHCR 预构建镜像，无 `build:`），由 `release.yml` 打包为 deploy.zip 交付；根 `docker-compose.yml` 仍为 staging / 开发形态。

仓库根 `docker-compose.yml`（staging 形态）：

### 5.1 Docker Compose

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
      - "${COLDRAWDB_HTTP_PORT:-9080}:80"
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

仓库根 `compose.release.yml`（稳定版 release 形态，随 deploy.zip 交付；镜像 tag 由 `release.yml` 按 Release 名注入）：

```yaml
# release 形态：引用 GHCR 预构建镜像，无需本地构建。
services:
  coldrawdb:
    image: ghcr.io/<owner>/coldrawdb:<tag>
    ports:
      - "3000:3000"
    volumes:
      - ./data:/data
      - ./logs:/logs
    environment:
      COLDRAWDB_DB_URL: "sqlite:///data/coldrawdb.db?mode=rwc"
      COLDRAWDB_LOG_LEVEL: "info"
      PUBLIC_BASE_URL: "${PUBLIC_BASE_URL:-http://localhost:9080}"
    restart: unless-stopped
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:3000/api/v1/diagrams/health"]
      interval: 30s
      timeout: 5s
      retries: 3
      start_period: 10s

  nginx:
    image: nginx:1.25
    ports:
      - "${COLDRAWDB_HTTP_PORT:-9080}:80"
    volumes:
      - ./nginx.conf:/etc/nginx/nginx.conf:ro
    depends_on:
      coldrawdb:
        condition: service_healthy
    restart: unless-stopped

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

仓库根 `nginx.conf`：`/api/` 与 `/ws/` 透传后端，其余回源后端静态服务（后端对非 API 路径回源 `index.html`，§12.3）：

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
