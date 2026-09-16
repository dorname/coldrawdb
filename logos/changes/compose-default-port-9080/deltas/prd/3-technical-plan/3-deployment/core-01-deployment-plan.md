# Delta — core-01-deployment-plan.md（Compose 默认宿主机端口 9080）

> 变更：compose-default-port-9080 / 覆盖重发 v0.1.0

## MODIFIED — §2.1 稳定版交付（Windows / macOS / Linux）

稳定版用户侧交付物为 **Docker 镜像 + Compose**，不是原生桌面安装包。

| 客户端 OS | 推荐运行时 | 获取方式 | 访问入口 |
|---|---|---|---|
| Windows | Docker Desktop（WSL2 后端） | `docker compose up -d --build` 或 `docker pull ghcr.io/<owner>/coldrawdb:<tag>` | 浏览器 `http://localhost:9080/`（nginx 容器 :80 → 宿主机 **9080**） |
| macOS | Docker Desktop（Intel / Apple Silicon） | 同上；镜像含 `linux/arm64` | 同上 |
| Linux | Docker Engine + Compose v2 | 同上 | 同上 |

本地 Rust/trunk 双进程（§3）仅开发路径；原生 Windows 无 WSL 时必须用 Docker。

版本标签约定：

- SemVer annotated tag：`vMAJOR.MINOR.PATCH`（首个稳定版 `v0.1.0`）
- Tag 推送触发：
  1. `.github/workflows/docker.yml` → `ghcr.io/<owner>/coldrawdb:<tag>` + `:latest`，平台 `linux/amd64,linux/arm64`
  2. `.github/workflows/release.yml` → GitHub Release + `coldrawdb-<tag>-src.zip`

跨机邀请：设置 `PUBLIC_BASE_URL` 为公开 SPA 入口（Compose 默认 `http://localhost:9080`，禁止默认裸后端 `:3000`）。

**宿主机端口**：默认 **9080**，避免 Windows / macOS 上 80 常被占用。可通过环境变量 `COLDRAWDB_HTTP_PORT` 覆盖映射（如 `COLDRAWDB_HTTP_PORT=80 docker compose up -d --build`）。

## MODIFIED — §5.1 docker-compose.yml 示例（nginx ports）

在 `nginx` service 的 `ports` 中，将：

```yaml
      - "80:80"
```

替换为：

```yaml
      - "${COLDRAWDB_HTTP_PORT:-9080}:80"
```

在 `coldrawdb` service 的 `environment` 中，`PUBLIC_BASE_URL` 默认改为：

```yaml
PUBLIC_BASE_URL: "${PUBLIC_BASE_URL:-http://localhost:9080}"
```

## MODIFIED — §12 合并块（PUBLIC_BASE_URL / nginx 入口）

约定表更新：

| 变量 | 默认 | 说明 |
|---|---|---|
| `COLDRAWDB_HTTP_PORT` | `9080` | nginx 映射到宿主机的 HTTP 端口（容器内仍为 :80） |
| `PUBLIC_BASE_URL` | `http://localhost:9080`（compose 默认；对应 nginx 宿主机映射 **9080**） | 后端生成 `invite_url` 的公开基址。跨机/域名部署时显式覆盖，如 `PUBLIC_BASE_URL=http://192.168.20.106:9080` 或 `https://coldrawdb.example.com` |

**禁止**：把默认值设为 `http://localhost:3000` 或仅后端端口——被邀请人打开后可能拿不到 SPA（公开入口一律走 nginx 反代 / 同源域名）。

- 明确：issue 截图形态 `http://{lan-ip}:9080/invite/...`（经默认 compose 映射）为**正确**；若链接不可达，应改 `PUBLIC_BASE_URL` 为真实可达入口（含端口），而非改成 `:3000`。
- 文档示例：`PUBLIC_BASE_URL=http://192.168.20.106:9080 docker compose up -d`（端口须与 `COLDRAWDB_HTTP_PORT` 一致）。
