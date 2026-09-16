# 实现任务

## [delta] 规格变更

- [x] 产出 delta 文件到 `deltas/prd/3-technical-plan/3-deployment/core-01-deployment-plan.md` — 稳定版三平台交付矩阵 + `v*` tag / GHCR 发布约定
- [x] 产出 delta 文件到 `deltas/test/smoke/core-smoke-test-cases.md` — 增补稳定版 Compose 健康检查冒烟（SMOKE-core-STABLE-01）

## [code] 发布流水线

- [x] 新增 `.github/workflows/release.yml` — tag `v*` 时创建 GitHub Release 并附源码 zip
- [x] 确认 `.github/workflows/docker.yml` 已对 tag 推送 `linux/amd64,linux/arm64`（无需改则文档引用即可）
- [ ] 刷新本地 `dist/coldrawdb-0.1.0-src.zip`（`git archive`）供 Release 附件对照

## [deploy] 发布执行

- [ ] 合并规格后打 annotated tag `v0.1.0` 并 `git push origin main --tags`（人类确认）
- [ ] 确认 GHCR 出现 `ghcr.io/dorname/coldrawdb:v0.1.0`（amd64+arm64）
- [ ] 确认 GitHub Release `v0.1.0` 含源码 zip
- [ ] Docker 可用时执行 Compose 冒烟；不可用时在 deployment-report 记录环境限制并以 CI 证据代替
