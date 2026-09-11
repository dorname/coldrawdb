# 变更提案：persist-public-base-url

> module: core | created: 2026-09-09

## 变更原因
本地 dev 双进程（trunk :8080 + backend :3000，无代理）下，`start-local.sh` 未为后端注入 `PUBLIC_BASE_URL`，邀请链接按 Host header 推导落到后端 `:3000`（`backend/src/rooms/mod.rs` `public_base_url()`），而后端无 SPA 路由，被邀请人打开 `/invite/{token}` 得到 404，无法加入协作空间（2026-09-09 实测复现）。

部署方案 `core-01-deployment-plan.md` §12.4 已记录该约束（"dev 下 `PUBLIC_BASE_URL` 必须指向前端入口"），但脚本未落实，属于既有规格的执行缺口。本次变更将其固化到启动脚本默认值中。

## 变更类型
代码级修复（开发脚本对齐既有部署规格 + 部署文档小节同步）

## 变更范围
- 影响的需求文档：无
- 影响的功能规格：无（S04 邀请产品逻辑不变，仅 dev 环境行为对齐）
- 影响的业务场景：S04（仅 dev 启动行为，时序与 API 不变）
- 影响的部署方案：`core-01-deployment-plan.md` §12.4（`PUBLIC_BASE_URL` 默认值由"空"改为"脚本默认注入前端入口"）
- 影响的 API：无
- 影响的 DB 表：无
- 影响的编排测试：无
- 影响的 smoke 测试：无
- 影响的代码：`scripts/common.sh`（新增默认值导出）、`scripts/tests/test-local-scripts.sh`（补用例）

## 部署影响
- 是否需要部署：否
- 部署原因：仅本地开发脚本变更；生产单入口（§12.3）前后端同源，Host 推导天然正确，不受影响
- 影响环境：本地
- 是否涉及数据迁移：否
- 是否需要回滚预案：否
- 是否需要 smoke：否

## 变更概述
在 `scripts/common.sh` 中为 `PUBLIC_BASE_URL` 增加默认值导出：`${PUBLIC_BASE_URL:-http://127.0.0.1:${COLDRAWDB_FRONTEND_PORT}}`，即默认指向本地前端入口（trunk :8080），保留环境变量覆盖能力（跨机验证时用户可显式设为局域网地址，符合 §12.4 示例）。

同步更新 `core-01-deployment-plan.md` §12.4 环境变量表中 `PUBLIC_BASE_URL` 的默认值说明，并在 `scripts/tests/test-local-scripts.sh` 补充"默认值注入"与"环境变量覆盖优先"两条脚本用例。
