# 合并指令

## 变更提案
- 提案名称：release-deploy-bundle
- 提案目录：logos/changes/release-deploy-bundle/

## 提案内容

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


## 需要合并的 Delta 文件

### 1. deltas/prd/3-technical-plan/3-deployment/core-01-deployment-plan.md

- Delta 文件：`logos/changes/release-deploy-bundle/deltas/prd/3-technical-plan/3-deployment/core-01-deployment-plan.md`
- 目标目录：`logos/resources/prd/3-technical-plan/3-deployment/`
- 操作：读取 delta 中的 ADDED / MODIFIED / REMOVED 标记，合并到目标目录中对应的主文档

### 2. deltas/test/smoke/core-smoke-test-cases.md

- Delta 文件：`logos/changes/release-deploy-bundle/deltas/test/smoke/core-smoke-test-cases.md`
- 目标目录：`logos/resources/test/smoke/`
- 操作：读取 delta 中的 ADDED / MODIFIED / REMOVED 标记，合并到目标目录中对应的主文档

## 执行要求

1. 逐个 Delta 文件处理，每处理完一个报告修改摘要
2. 对于 ADDED 标记：在主文档的指定位置插入新内容
3. 对于 MODIFIED 标记：替换主文档中同名章节的内容
4. 对于 REMOVED 标记：从主文档中删除对应章节
5. 保持主文档的原有格式和风格
6. 如果主文档有"最后更新"时间戳，同步更新
7. 所有变更完成后，列出修改清单
8. 所有变更合并完成后，自动执行 git commit（告知用户，无需确认）：
   git add -A && git commit -m "docs(release-deploy-bundle): merge spec deltas"
   然后提示用户：按更新后的规格实现代码，代码完成后运行 `openlogos verify` 验收，验收通过后明确授权执行 `openlogos archive release-deploy-bundle`。
