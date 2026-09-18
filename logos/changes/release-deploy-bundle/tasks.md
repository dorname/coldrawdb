# 实现任务

## [delta] 规格变更

- [x] 产出 delta 文件到 `deltas/prd/3-technical-plan/3-deployment/core-01-deployment-plan.md` — 稳定版交付物改为 `coldrawdb-<tag>-deploy.zip`（compose.release 形态）；§2.1 交付物清单、§4-5 稳定版运行方式同步
- [x] 产出 delta 文件到 `deltas/test/smoke/core-smoke-test-cases.md` — `SMOKE-core-STABLE-01` 改为以 deploy.zip 解压目录为起点（无 `--build`）

## [code] 发布工作流与文档

- [ ] 实现代码变更

## [deploy] 覆盖重发 v0.1.0

- [ ] 推送含本变更的 `main`
- [ ] 本地移动 annotated tag `v0.1.0` 至新 HEAD，并 `git push --force origin v0.1.0`
- [ ] 删除旧 GitHub Release `v0.1.0`（若存在）并确认 tag 推送触发 Release（deploy.zip）+ Docker CI 重建
- [ ] 确认 Release 资产含 `coldrawdb-v0.1.0-deploy.zip`、GHCR 镜像推送成功；本机无 Docker 时 SMOKE 记 SKIPPED
