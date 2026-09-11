# 合并指令

## 变更提案
- 提案名称：fix-local-start-and-invite-config
- 提案目录：logos/changes/fix-local-start-and-invite-config/

## 提案内容

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
- 影响的代码：`scripts/start-local.sh`（env 参数顺序 + 前端编译期 `COLDRAWDB_API_BASE` 接线 + 可选 `PUBLIC_BASE_URL` 透传说明）

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


## 需要合并的 Delta 文件

### 1. deltas/prd/3-technical-plan/3-deployment/core-01-deployment-plan.md

- Delta 文件：`logos/changes/fix-local-start-and-invite-config/deltas/prd/3-technical-plan/3-deployment/core-01-deployment-plan.md`
- 目标目录：`logos/resources/prd/3-technical-plan/3-deployment/`
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
   git add -A && git commit -m "docs(fix-local-start-and-invite-config): merge spec deltas"
   然后提示用户：按更新后的规格实现代码，代码完成后运行 `openlogos verify` 验收，验收通过后明确授权执行 `openlogos archive fix-local-start-and-invite-config`。
