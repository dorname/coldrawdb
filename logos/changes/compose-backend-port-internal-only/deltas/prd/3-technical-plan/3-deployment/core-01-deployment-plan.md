# Delta：core-01-deployment-plan.md（compose-backend-port-internal-only）

## MODIFIED — 5.1 Docker Compose

### 5.1 Docker Compose

> 更新于 feat-docker-compose-deploy（2026-09-10）：`docker-compose.yml` 与 `nginx.conf` 已落地仓库根目录；旧 `compose.yml`（dev 挂载栈）与 `vercel.json` 已删除。
> 更新于 release-deploy-bundle（2026-09-18）：仓库根 `compose.release.yml` 为稳定版 release 形态 Compose（引用 GHCR 预构建镜像，无 `build:`），由 `release.yml` 打包为 deploy.zip 交付；根 `docker-compose.yml` 仍为 staging / 开发形态。
> 更新于 compose-backend-port-internal-only（2026-09-22）：后端容器**不再映射宿主机 3000 端口**，仅暴露于 compose 内部网络供 nginx 反代（`coldrawdb:3000`）；对外唯一入口为 nginx 9080，消除"3000 与 9080 均可访问"的双入口迷惑。调试直连见本节末尾说明。

仓库根 `docker-compose.yml`（staging 形态）：

```yaml
services:
  coldrawdb:
    image: coldrawdb:v1
    build: .
    # 不映射宿主机端口：后端仅经 compose 内部网络（coldrawdb:3000）供 nginx 反代
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
    # 不映射宿主机端口：后端仅经 compose 内部网络（coldrawdb:3000）供 nginx 反代
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
```

**端口拓扑**：宿主机仅 nginx 监听（默认 9080）；后端 3000 仅存在于 compose 内部网络（nginx.conf `upstream backend { server coldrawdb:3000; }`）。容器内 healthcheck 走 `localhost:3000` 不受影响。

**调试直连后端（非交付形态）**：排查问题时可用 `docker compose run --service-ports --rm coldrawdb` 临时获得宿主 3000 直连，或 `docker run -p 3000:3000 -v ./data:/data coldrawdb:v1` 单容器启动（无 nginx，3000 即入口）。
