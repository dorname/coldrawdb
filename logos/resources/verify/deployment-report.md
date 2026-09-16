# 部署报告

| 项 | 值 |
|---|---|
| 变更 | `release-stable-win-linux-mac` |
| 部署时间 | 2026-09-16 |
| 目标环境 | GHCR + GitHub Release（用户本机 Docker：Win/macOS/Linux） |
| 版本 | `v0.1.0` |
| 状态 | 执行中 |

## 执行摘要

1. 规格已合并（稳定版三平台交付矩阵 + SMOKE-core-STABLE-01）
2. `openlogos verify` Gate 3.6 PASS
3. 推送 `main` 与 annotated tag `v0.1.0`，触发：
   - `.github/workflows/docker.yml` → `ghcr.io/dorname/coldrawdb:v0.1.0`（linux/amd64 + linux/arm64）
   - `.github/workflows/release.yml` → GitHub Release + 源码 zip
4. 本机无 Docker daemon：Compose 冒烟按规格记为 `SKIPPED: no docker daemon`，以 CI 双架构推送成功为替代证据

## 回滚点

- `git push origin :refs/tags/v0.1.0` 并删除 GitHub Release（如需）
- GHCR 保留历史 tag；客户端可改拉上一版本

## 访问（发布完成后）

- 镜像：`ghcr.io/dorname/coldrawdb:v0.1.0`
- Compose：克隆 tag 后 `docker compose up -d --build` → http://localhost/
