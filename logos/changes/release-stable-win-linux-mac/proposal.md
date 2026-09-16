# 变更提案：release-stable-win-linux-mac

> module: core | created: 2026-09-16
> 目标版本：`v0.1.0`

## 变更原因

发布支持 **Windows / macOS / Linux** 的稳定版。产品为浏览器端自托管应用，正式交付物为 **Docker Compose / GHCR 多架构镜像**（`linux/amd64` + `linux/arm64`），三平台通过 Docker Desktop / Engine 获取并运行同一套容器。

当前已具备：Compose 部署方案、tag 触发的多架构推送 CI、README 三平台入口说明、源码 zip 打包能力。缺：版本标签、GitHub Release、部署规格中的「稳定版」合同、以及可核对的发布冒烟证据。

## 变更类型

部署级变更（含发布流水线与文档；无业务 API / DB 变更）

## 变更范围

- 影响的需求文档：无（不改产品 Why）
- 影响的功能规格：无强制；README 已含三平台入口（非方法论文件，可随发布微调）
- 影响的业务场景：无新场景编号
- 影响的部署方案：`prd/3-technical-plan/3-deployment/core-01-deployment-plan.md` — 增补「稳定版交付矩阵」与 `v*` 标签发布约定
- 影响的 API：无
- 影响的 DB 表：无
- 影响的编排测试：无
- 影响的 smoke 测试：`test/smoke/core-smoke-test-cases.md` — 增补稳定版 Compose / 健康检查冒烟（Docker 可用时）
- 影响的 CI：新增 `.github/workflows/release.yml`（tag → GitHub Release + 源码 zip）；沿用既有 `docker.yml` 推送 GHCR

## 部署影响

- 是否需要部署：是
- 部署原因：打 `v0.1.0` 标签触发 GHCR 多架构镜像推送，并创建 GitHub Release；可选本机 Compose 冒烟
- 影响环境：GitHub Packages（GHCR）+ 用户本机 Docker（Win/macOS/Linux）
- 是否涉及数据迁移：否
- 是否需要回滚预案：是（删除/撤回 tag 与 Release；镜像保留 `:latest` 可回退上一 tag）
- 是否需要 smoke：是（Compose 健康检查 + SPA 可达；无 Docker 时记录环境跳过并保留 CI 证据）

## 变更概述

1. 规格：部署方案明确稳定版 = Docker 交付，环境矩阵覆盖 Win/macOS/Linux。
2. CI：tag `v*` 时创建 GitHub Release，并附 `coldrawdb-<tag>-src.zip`。
3. 发布：在 `main` 打 annotated tag `v0.1.0` 并 push tags，等待 `docker.yml` 推送 `ghcr.io/dorname/coldrawdb:v0.1.0`（amd64+arm64）。
4. 冒烟：Docker 可用时 `docker compose up` + health；不可用时以 CI 双架构推送成功 + README 入口作为替代证据，并在 smoke 报告标注环境限制。
