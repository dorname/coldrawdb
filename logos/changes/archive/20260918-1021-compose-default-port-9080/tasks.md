# 实现任务

## [delta] 规格变更

- [x] 产出 delta 文件到 `deltas/prd/3-technical-plan/3-deployment/core-01-deployment-plan.md` — 稳定版入口改为宿主机 9080；compose 映射与 `PUBLIC_BASE_URL` 默认值同步
- [x] 产出 delta 文件到 `deltas/test/smoke/core-smoke-test-cases.md` — `SMOKE-core-STABLE-01` 使用 `http://localhost:9080`

## [code] Compose / 文档 / 发布说明

- [x] 更新 `docker-compose.yml`：nginx `"${COLDRAWDB_HTTP_PORT:-9080}:80"`；`PUBLIC_BASE_URL` 默认 `http://localhost:9080`
- [x] 更新 `README.md`「稳定版运行」：入口与健康检查改为 `:9080`，说明可用 `COLDRAWDB_HTTP_PORT` 覆盖
- [x] 更新 `.github/workflows/release.yml` Release body：浏览器入口改为 `http://localhost:9080`

## [deploy] 覆盖重发 v0.1.0

- [x] 推送含本变更的 `main`
- [x] 本地移动 annotated tag `v0.1.0` 至新 HEAD，并 `git push --force origin v0.1.0`（覆盖远端 tag）
- [x] 删除旧 GitHub Release `v0.1.0`（若存在）并确认 tag 推送触发 Release + Docker CI 重建
- [x] 修复 `docker.yml` GHCR 登录（`GITHUB_TOKEN` + `packages: write`）并再次覆盖推送 tag
- [x] 确认 GHCR 镜像推送成功；本机无 Docker 时 SMOKE 记 SKIPPED
  - GHCR 推送：Docker Build and Push #14（commit 5cce410，tag v0.1.0）completed success，镜像 `ghcr.io/dorname/coldrawdb:v0.1.0` 就绪（multi-arch amd64/arm64）
  - SMOKE-core-STABLE-01：本机无 Docker daemon，记 SKIPPED（deployment-report §执行摘要 6）
