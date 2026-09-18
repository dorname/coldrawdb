# 变更提案：release-deploy-bundle

> module: core | created: 2026-09-18
> 关联版本：覆盖重发 `v0.1.0`（同 tag 移动到含本变更的 commit）

## 变更原因

用户反馈：v0.1.0 Release 只放了 `coldrawdb-v0.1.0-src.zip`（源码包）+ GitHub 自动生成的 `Source code (zip/tar.gz)` 共三个源码资产，与「稳定版一键部署」的产品定位不符——正式交付物应是**预构建 Docker 镜像 + Compose 组成的一键部署压缩包**，用户解压后无需源码、无需本地构建即可启动。当前设计合同（部署方案 §2.1 与 §4-5）定义稳定版交付为「镜像 + Release 源码 zip + 用户本机 `--build` 构建」，需要修订为 deploy bundle 形态。

## 变更类型

部署级变更（发布产物形态 + 部署方案 / smoke 用例 / 发布工作流 / 文档；无业务 API / DB 变更）

## 变更范围

- 影响的需求文档：无
- 影响的功能规格：无
- 影响的业务场景：无新场景编号（发布交付物形态不变更运行时行为）
- 影响的部署方案：`prd/3-technical-plan/3-deployment/core-01-deployment-plan.md` — §2.1 稳定版交付物清单与获取方式、§4-5 稳定版 Compose 形态说明
- 影响的 API：无
- 影响的 DB 表：无
- 影响的编排测试：无
- 影响的 smoke 测试：`test/smoke/core-smoke-test-cases.md` — §8.5 SMOKE-core-STABLE-01 改为以 `coldrawdb-<tag>-deploy.zip` 为起点
- 影响的运行时配置（非 delta，归 `[code]`）：`.github/workflows/release.yml`（改为打 deploy.zip 并上传 Release）、新增 `compose.release.yml`（release 形态 compose：引用 GHCR 镜像，无 `build:`）、README「稳定版运行」
- 不涉及 `docker-compose.yml`（staging / 开发形态保持不变）

## 部署影响

- 是否需要部署：是
- 部署原因：合并后推送 `main`，**强制移动** annotated tag `v0.1.0` 并推送远端（`git push --force --tags`），删除并重建 GitHub Release `v0.1.0`，使下载与镜像对应 deploy.zip 形态
- 影响环境：GitHub Release（用户本机 Docker：Win/macOS/Linux）+ GHCR（docker.yml 不变）
- 是否涉及数据迁移：否
- 是否需要回滚预案：是（可将 tag 移回上一 commit，Release 重建为旧形态）
- 是否需要 smoke：是（SMOKE-core-STABLE-01 以 deploy.zip 起点的健康检查；无 Docker 时记录跳过并以 CI 证据代替）

## 变更概述

1. Release 正式交付物由「源码 zip」改为 **`coldrawdb-<tag>-deploy.zip`**：含 `compose.yml`（release 形态，引用 `ghcr.io/<owner>/coldrawdb:<tag>` 预构建镜像，无本地构建步骤）、`nginx.conf`、`.env.sample`、`快速上手.md`（解压 → `docker compose up -d` → 打开 `http://localhost:9080/`）。
2. `release.yml` 改为按 Release 名注入精确版本号打包 deploy zip；Release body 改为指向 deploy.zip，并注明 GitHub 自动归档的 `Source code (zip/tar.gz)` 非正式交付物。
3. 仓库根 `docker-compose.yml` 保持 staging / 开发形态不变（`build: .` + 9080），新增 `compose.release.yml` 供打包。
4. 同步部署规格、SMOKE-core-STABLE-01、README「稳定版运行」。
5. verify PASS 后**覆盖重发** `v0.1.0`（人类确认后执行）。
