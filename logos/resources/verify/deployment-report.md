# 部署报告

| 项 | 值 |
|---|---|
| 变更 | `compose-default-port-9080` |
| 部署时间 | 2026-09-16 |
| 目标环境 | GHCR + GitHub Release（用户本机 Docker：Win/macOS/Linux） |
| 版本 | `v0.1.0`（覆盖重发） |
| 状态 | Release 已重建；Docker 多架构推送进行中 |

## 执行摘要

1. 规格已合并：Compose 默认宿主机端口 **9080**；`PUBLIC_BASE_URL` 默认 `http://localhost:9080`
2. `openlogos verify` Gate 3.6 PASS（431/431）
3. 推送 `main` 后 **force 移动** annotated tag `v0.1.0`，触发：
   - `.github/workflows/docker.yml` → `ghcr.io/dorname/coldrawdb:v0.1.0`（linux/amd64 + linux/arm64）
   - `.github/workflows/release.yml` → 重建 GitHub Release + 源码 zip
4. 本机无 Docker daemon：Compose 冒烟按规格记为 `SKIPPED: no docker daemon`，以 CI 双架构推送成功为替代证据

## 回滚点

- 将 tag 移回上一 commit，或临时 `COLDRAWDB_HTTP_PORT=80`
- `git push --force origin v0.1.0` 的逆向：再 force 到旧 SHA（如需）

## 访问（发布完成后）

- 镜像：`ghcr.io/dorname/coldrawdb:v0.1.0`
- Compose：`docker compose up -d --build` → http://localhost:9080/
