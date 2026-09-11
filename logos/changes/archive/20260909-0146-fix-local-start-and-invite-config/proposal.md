# 变更提案：fix-local-start-and-invite-config

> module: core | created: 2026-09-08

## 变更原因

用户实测 `./scripts/start-local.sh` 本地启动失败（2026-09-08），两个叠加缺陷：

1. **前端进程根本没启动**：`scripts/start-local.sh:84` 的 `exec env NO_COLOR=1 -u FORCE_COLOR trunk serve ...` 违反 GNU env 参数顺序（选项必须先于 NAME=VALUE 赋值），env 把 `-u` 当作要执行的程序名，trunk 从未运行，就绪等待 60s 超时后脚本退出（日志实证：`env: '-u': No such file or directory`，exit=127）。
2. **dev 双进程 API 断连（修复 1 后必现）**：fix-canvas-zoom-invite-comment-resize 已把前端 base_url 改为 `window.location.origin` 同源派生（UT-S04-UI-17），但本地 dev 是 trunk `:8080` + 后端 `:3000` 双进程、trunk 无代理——API 请求会打到 `:8080` 全部失败。规格 §12.2 预留的 dev 覆盖口 `COLDRAWDB_API_BASE` 未在启动脚本中接线。

同时用户反馈不清楚邀请链接如何配置——`PUBLIC_BASE_URL` 与 dev 双进程下 Host 推导会指向后端端口（SPA 在 8080）的注意事项未在部署文档写明。

## 变更类型

代码级修复（含一条部署文档 delta：补充 dev 启动环境变量说明）

## 变更范围

- 影响的需求文档：无
- 影响的功能规格：无
- 影响的业务场景：无（不改变任何功能行为，仅修复本地启动与 dev 配置接线）
- 影响的部署方案：`prd/3-technical-plan/3-deployment/core-01-deployment-plan.md`（§12 补充 dev 双进程启动的环境变量约定：start-local.sh 传 `COLDRAWDB_API_BASE`；`PUBLIC_BASE_URL` 在 dev 下应指向前端入口 8080 的原因）
- 影响的 API：无
- 影响的 DB 表：无
- 影响的编排测试：无
- 影响的 smoke 测试：无
- 影响的代码：`scripts/start-local.sh`（env 参数顺序 + 前端编译期 `COLDRAWDB_API_BASE` 接线 + 可选 `PUBLIC_BASE_URL` 透传说明）；`backend/src/main.rs`（UT-PU-21 锚点断言同步 NO_COLOR=true）；`frontend-rs/src/editor_panels.rs`（verify 期间发现的既有缺陷修复：on_enter_room / on_invite_after_accept / share 冷启动的过期加载竞态——GET /diagrams/{id} 返回时若 diagram 已切换或 store 已有本地编辑（dirty），丢弃过期加载结果，避免全量覆盖用户进房后首轮编辑，曾致 e2e 间歇 30s 超时）

## 部署影响

- 是否需要部署：否
- 部署原因：仅修复本地启动脚本与补充文档说明，无生产部署任务
- 影响环境：本地
- 是否涉及数据迁移：否
- 是否需要回滚预案：否
- 是否需要 smoke：否

## 变更概述

1. 修复 `scripts/start-local.sh` 的 GNU env 参数顺序：`env -u FORCE_COLOR NO_COLOR=1 trunk serve ...`，使前端真正能启动。
2. 在 `start-local.sh` 启动前端时设置编译期 `COLDRAWDB_API_BASE=http://127.0.0.1:${COLDRAWDB_BACKEND_PORT}`（dev 双进程覆盖口，规格 §12.2 已预留），保证 dev 下前端 API 直连后端；脚本注释说明生产单入口不需要它。
3. 在部署方案 §12 补充 dev 启动配置说明：`start-local.sh` 的环境变量约定，以及 dev 下 `PUBLIC_BASE_URL` 应指向前端入口（`http://<局域网IP>:8080`）否则邀请链接 Host 推导会指向后端端口的原因。
