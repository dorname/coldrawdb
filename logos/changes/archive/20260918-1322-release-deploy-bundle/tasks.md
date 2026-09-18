# 实现任务

## [delta] 规格变更

- [x] 产出 delta 文件到 `deltas/prd/3-technical-plan/3-deployment/core-01-deployment-plan.md` — 稳定版交付物改为 `coldrawdb-<tag>-deploy.zip`（compose.release 形态）；§2.1 交付物清单、§4-5 稳定版运行方式同步
- [x] 产出 delta 文件到 `deltas/test/smoke/core-smoke-test-cases.md` — `SMOKE-core-STABLE-01` 改为以 deploy.zip 解压目录为起点（无 `--build`）

## [code] 发布工作流与文档

- [x] 新增 `compose.release.yml`：引用 `ghcr.io/<owner>/coldrawdb:<tag>` 预构建镜像，无 `build:` 段，保留 nginx / backup 服务与 9080 默认端口
- [x] 更新 `.github/workflows/release.yml`：打包 `coldrawdb-<tag>-deploy.zip`（`compose.release.yml` + `nginx.conf` + `.env.sample` + `快速上手.md`，按 Release 名注入镜像 tag），Release body 指向 deploy.zip 并注明自动归档非交付物
- [x] 更新 `README.md`「稳定版运行」：推荐入口改为下载 deploy.zip，staging 形态 compose 保留为开发说明

## [deploy] 覆盖重发 v0.1.0

- [x] 推送含本变更的 `main`
- [x] 本地移动 annotated tag `v0.1.0` 至新 HEAD，并 `git push --force origin v0.1.0`
- [x] 删除旧 GitHub Release `v0.1.0`（若存在）并确认 tag 推送触发 Release（deploy.zip）+ Docker CI 重建
- [x] 确认 Release 资产含 `coldrawdb-v0.1.0-deploy.zip`、GHCR 镜像推送成功；本机无 Docker 时 SMOKE 记 SKIPPED
  - Release 资产已确认含 deploy.zip（API + 页面核验），旧 src.zip 已不在资产列表
  - GHCR 多架构推送：Docker Build and Push #16（tag v0.1.0）后台进行中，用户选择不等其完成，以 CI 运行中作为后续跟踪项（deployment-report §执行摘要 6）
  - SMOKE-core-STABLE-01：本机无 Docker daemon，记 SKIPPED（已追加至 smoke-results.jsonl）
