# Delta — core-01-deployment-plan.md（稳定版 Win/macOS/Linux 交付）

> 变更：release-stable-win-linux-mac / v0.1.0

## ADDED — §2.1 稳定版交付矩阵

在「§2. 环境矩阵」之后插入：

### 2.1 稳定版交付（Windows / macOS / Linux）

稳定版用户侧交付物为 **Docker 镜像 + Compose**，不是原生桌面安装包。

| 客户端 OS | 推荐运行时 | 获取方式 | 访问入口 |
|---|---|---|---|
| Windows | Docker Desktop（WSL2 后端） | `docker compose up -d --build` 或 `docker pull ghcr.io/<owner>/coldrawdb:<tag>` | 浏览器 `http://localhost/`（nginx:80） |
| macOS | Docker Desktop（Intel / Apple Silicon） | 同上；镜像含 `linux/arm64` | 同上 |
| Linux | Docker Engine + Compose v2 | 同上 | 同上 |

本地 Rust/trunk 双进程（§3）仅开发路径；原生 Windows 无 WSL 时必须用 Docker。

版本标签约定：

- SemVer annotated tag：`vMAJOR.MINOR.PATCH`（首个稳定版 `v0.1.0`）
- Tag 推送触发：
  1. `.github/workflows/docker.yml` → `ghcr.io/<owner>/coldrawdb:<tag>` + `:latest`，平台 `linux/amd64,linux/arm64`
  2. `.github/workflows/release.yml` → GitHub Release + `coldrawdb-<tag>-src.zip`

跨机邀请：设置 `PUBLIC_BASE_URL` 为公开 SPA 入口（Compose 默认 `http://localhost`，禁止默认裸后端 `:3000`）。

## MODIFIED — §1 部署目标

在目标表增加一行：

| 稳定版发布 | GHCR 多架构镜像 + GitHub Release；用户本机 Docker 运行 |

## MODIFIED — §2 环境矩阵「Docker」列说明

操作系统行「任意」明确为：任意安装 Docker 的 Windows / macOS / Linux 宿主机（容器内仍为 Linux）。
