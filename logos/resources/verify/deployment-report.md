# 部署报告

| 项 | 值 |
|---|---|
| 变更 | `release-deploy-bundle` |
| 部署时间 | 2026-09-18 |
| 目标环境 | GitHub Release（用户本机 Docker：Win/macOS/Linux）+ GHCR |
| 版本 | `v0.1.0`（覆盖重发） |
| 状态 | Release 已就绪（deploy.zip 资产就位）；GHCR 多架构推送（Docker #16）后台进行中 |

## 执行摘要

1. 规格已合并：稳定版交付物由「源码 zip」改为 **`coldrawdb-<tag>-deploy.zip`**（compose.yml 引用 GHCR 预构建镜像 + nginx.conf + .env.sample + 快速上手.md）
2. `openlogos verify` Gate 3.6 PASS（431/431，Coverage 100%）
3. 已推送 `main`（`9cc9f29`），force 移动 annotated tag `v0.1.0`（`c2ad9b7...89a9f2f`）
4. Release #4 自动重建（10s 完成），body 指向 deploy.zip 并注明自动归档非交付物
5. 旧 `coldrawdb-v0.1.0-src.zip` 资产已从 Release 移除（删除时 PAT 权限不足，但 softprops 覆盖 Release 时已清理；API 资产列表与页面均已确认只剩 deploy.zip）
6. Docker Build and Push #16（tag v0.1.0，多架构推 GHCR）后台进行中（上次同型构建耗时约 3h，预计 2026-09-18 上午完成）
7. 本机无 Docker daemon：SMOKE-core-STABLE-01 记 `SKIPPED: no docker daemon`（已追加至 smoke-results.jsonl），以 CI 证据代替

## 回滚点

- tag / Release 可 force 回上一 SHA（`c2ad9b7`，compose-default-port-9080 完成态）
- 或临时在 Release 页面手动上传旧源码 zip

## 访问

- Release：https://github.com/dorname/coldrawdb/releases/tag/v0.1.0
- 镜像（CI 完成后）：`ghcr.io/dorname/coldrawdb:v0.1.0`（linux/amd64 + linux/arm64）
- 使用：下载 deploy.zip → 解压 → `docker compose up -d` → http://localhost:9080/
