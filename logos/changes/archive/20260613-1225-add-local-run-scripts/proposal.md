# 变更提案：add-local-run-scripts

> module: core | created: 2026-06-13

## 变更原因

当前项目本地开发需要开发者手动在两个终端分别启动后端（`cargo run`）和前端（`trunk serve`），步骤繁琐且没有统一的停止方式。为了降低本地开发门槛，需要一组统一的本地启动/停止脚本，实现一键启停前后端服务。

## 变更类型

部署级变更

## 变更范围

- 影响的需求文档：无
- 影响的功能规格：无
- 影响的业务场景：S01（编辑保存）、S02（分享加载）—— 本地验证入口更稳定
- 影响的部署方案：`logos/resources/prd/3-technical-plan/3-deployment/core-01-deployment-plan.md` §3 本地 dev 部署
- 影响的 API：无
- 影响的 DB 表：无
- 影响的编排测试：无
- 影响的 smoke 测试：`logos/resources/test/smoke/core-smoke-test-cases.md`（新增脚本启停验证用例）

## 部署影响

- 是否需要部署：否
- 部署原因：本次变更仅添加本地开发脚本和更新部署文档，不执行外部环境部署
- 影响环境：本地
- 是否涉及数据迁移：否
- 是否需要回滚预案：否
- 是否需要 smoke：是（验证 start/stop 脚本能正确启停前后端服务）

## 变更概述

本提案为项目增加本地开发一键启动/停止脚本。脚本位于仓库根目录 `scripts/` 下，包含 `start-local.sh` 和 `stop-local.sh`：`start-local.sh` 会按正确顺序拉起后端（端口 3000）和前端（端口 8080），并将日志写入 `logs/`；`stop-local.sh` 会根据 PID 文件安全终止前后端进程。同时更新部署方案文档中本地 dev 部署章节，补充脚本使用说明、配置参数和日志路径；并在 smoke 测试用例中新增对脚本启停链路的验证。
