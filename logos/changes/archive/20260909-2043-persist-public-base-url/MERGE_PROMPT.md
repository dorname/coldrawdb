# 合并指令

## 变更提案
- 提案名称：persist-public-base-url
- 提案目录：logos/changes/persist-public-base-url/

## 提案内容

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


## 需要合并的 Delta 文件

### 1. deltas/prd/3-technical-plan/3-deployment/core-01-deployment-plan.md

- Delta 文件：`logos/changes/persist-public-base-url/deltas/prd/3-technical-plan/3-deployment/core-01-deployment-plan.md`
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
   git add -A && git commit -m "docs(persist-public-base-url): merge spec deltas"
   然后提示用户：按更新后的规格实现代码，代码完成后运行 `openlogos verify` 验收，验收通过后明确授权执行 `openlogos archive persist-public-base-url`。
