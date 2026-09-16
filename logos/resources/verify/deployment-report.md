# 部署报告

| 项 | 值 |
|---|---|
| 变更 | `compose-default-port-9080` |
| 部署时间 | 2026-09-16 |
| 目标环境 | GHCR + GitHub Release（用户本机 Docker：Win/macOS/Linux） |
| 版本 | `v0.1.0`（覆盖重发） |
| 状态 | Release 已就绪；GHCR 登录已修复并重推 tag |

## 执行摘要

1. 规格已合并：Compose 默认宿主机端口 **9080**；`PUBLIC_BASE_URL` 默认 `http://localhost:9080`
2. `openlogos verify` Gate 3.6 PASS（431/431）
3. 已推送 `main` 并 force 移动 `v0.1.0`；GitHub Release 已重建（入口说明含 `:9080`）
4. 首次 tag Docker job 在「Log in to ghcr」失败：`secrets.GH_TOKEN` 空 → **Password required**
5. 已将 `docker.yml` 改为 `GITHUB_TOKEN` + `permissions.packages: write`，并再次覆盖推送 `v0.1.0` 触发重建
6. 本机无 Docker daemon：SMOKE-core-STABLE-01 记为 `SKIPPED: no docker daemon`

## 回滚点

- `COLDRAWDB_HTTP_PORT=80` 可临时恢复旧宿主机映射
- tag / Release 可再 force 到上一 SHA

## 访问

- Release：https://github.com/dorname/coldrawdb/releases/tag/v0.1.0
- 镜像（CI 成功后）：`ghcr.io/dorname/coldrawdb:v0.1.0`
- Compose：`docker compose up -d --build` → http://localhost:9080/
