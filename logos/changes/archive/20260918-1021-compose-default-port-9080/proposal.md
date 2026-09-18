# 变更提案：compose-default-port-9080

> module: core | created: 2026-09-16
> 关联版本：覆盖重发 `v0.1.0`（同 tag 移动到含本变更的 commit）

## 变更原因

稳定版 Compose 默认将 nginx 映射到宿主机 **80**。Windows / macOS（及部分 Linux）上 80 常被占用，导致 `docker compose up` 失败或无法访问入口。用户明确要求默认入口改为 **9080**，并**覆盖原 `v0.1.0` tag / Release**，使已发布稳定版合同与可运行入口一致。

## 变更类型

部署级变更（Compose 宿主机端口默认值 + 文档 / smoke / 发布说明；无业务 API / DB 变更）

## 变更范围

- 影响的需求文档：无
- 影响的功能规格：无（邀请链接仍由 `PUBLIC_BASE_URL` / Host 推导；仅默认公开入口 URL 变更为含 `:9080`）
- 影响的业务场景：无新场景编号
- 影响的部署方案：`prd/3-technical-plan/3-deployment/core-01-deployment-plan.md` — 稳定版矩阵、compose 端口映射、`PUBLIC_BASE_URL` 默认值与 nginx 入口说明（宿主机 **9080**，容器内仍可保持 nginx:80）
- 影响的 API：无
- 影响的 DB 表：无
- 影响的编排测试：无
- 影响的 smoke 测试：`test/smoke/core-smoke-test-cases.md` — `SMOKE-core-STABLE-01` 改为 `http://localhost:9080`
- 影响的运行时配置（非 delta，归 `[code]`）：`docker-compose.yml`、`README.md`、`.github/workflows/release.yml` 发布说明正文

## 部署影响

- 是否需要部署：是
- 部署原因：合并后推送 `main`，**强制移动** annotated tag `v0.1.0` 并推送远端（`git push --force` tags），删除并重建 GitHub Release `v0.1.0`，触发 GHCR / Release CI，使下载与镜像对应新默认端口
- 影响环境：用户本机 Docker（Win/macOS/Linux）+ GitHub Packages（GHCR）+ GitHub Release
- 是否涉及数据迁移：否
- 是否需要回滚预案：是（可将 tag 移回上一 commit，或临时用 `COLDRAWDB_HTTP_PORT=80` 恢复旧映射）
- 是否需要 smoke：是（Compose 入口改为 `:9080` 的健康检查；无 Docker 时记录跳过并以 CI 证据代替）

## 变更概述

1. Compose 默认宿主机 HTTP 入口改为 **9080**：`"${COLDRAWDB_HTTP_PORT:-9080}:80"`，保留环境变量覆盖。
2. 默认 `PUBLIC_BASE_URL` 改为 `http://localhost:9080`，与 SPA 入口一致。
3. 同步部署规格、SMOKE-core-STABLE-01、README、Release 说明中的 URL。
4. 发布：在 verify PASS 后**覆盖**远端 `v0.1.0` tag 与 Release（人类确认后执行）。
