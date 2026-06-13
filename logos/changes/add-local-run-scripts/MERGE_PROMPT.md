# 合并指令

## 变更提案
- 提案名称：add-local-run-scripts
- 提案目录：logos/changes/add-local-run-scripts/

## 提案内容

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


## 需要合并的 Delta 文件

### 1. deltas/prd/3-technical-plan/3-deployment/core-01-deployment-plan.md

- Delta 文件：`logos/changes/add-local-run-scripts/deltas/prd/3-technical-plan/3-deployment/core-01-deployment-plan.md`
- 目标目录：`logos/resources/prd/3-technical-plan/3-deployment/`
- 操作：读取 delta 中的 ADDED / MODIFIED / REMOVED 标记，合并到目标目录中对应的主文档

### 2. deltas/test/smoke/core-smoke-test-cases.md

- Delta 文件：`logos/changes/add-local-run-scripts/deltas/test/smoke/core-smoke-test-cases.md`
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
   git add -A && git commit -m "docs(add-local-run-scripts): merge spec deltas"
   然后提示用户：按更新后的规格实现代码，代码完成后运行 `openlogos verify` 验收，验收通过后明确授权执行 `openlogos archive add-local-run-scripts`。
