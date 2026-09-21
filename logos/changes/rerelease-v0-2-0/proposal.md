# 变更提案：rerelease-v0-2-0

> module: core | created: 2026-09-21

## 变更原因

既有 GitHub Release `v0.2.0`（tag 指向 `b1c91ce0`）之后，主线又合入表头对比度修复等 5 个提交（含 #24）。需要**同版本号再发布**，使 `v0.2.0` tag / Release / GHCR 镜像与当前 `main` HEAD 一致。

## 变更类型

代码级（发布工程；无业务逻辑与版本号变更——`Cargo.toml` 已为 `0.2.0`）

## 变更范围

- 影响的需求文档：无
- 影响的功能规格：无
- 影响的业务场景：无
- 影响的 API：无
- 影响的 DB 表：无
- 影响的编排测试：无
- 影响的 smoke 测试：无

## 部署影响

- 是否需要部署：否（重建 GitHub Release + 触发 CI 推送 GHCR 镜像；不改本机/生产 compose 编排）
- 部署原因：无运行环境部署任务；镜像由 tag 触发的 `docker.yml` 推送
- 影响环境：GitHub Release / GHCR
- 是否涉及数据迁移：否
- 是否需要回滚预案：否（可再次删 tag/Release 重建）
- 是否需要 smoke：否

## UI/UX 变更声明

```yaml
ui_impact: false
design_system_mode: generated
design_system_fallback_reason: ""
pages: []
```

## 变更概述

1. 删除远程 Release `v0.2.0` 与远程/本地 tag `v0.2.0`。
2. 在当前 `main` HEAD（含对比度修复）重建附注 tag `v0.2.0` 并推送。
3. 由 `release.yml` / `docker.yml` 自动重建 `coldrawdb-v0.2.0-deploy.zip` 与 GHCR 多架构镜像。
4. 核对 Release 指向新 commit，且含 deploy.zip。
