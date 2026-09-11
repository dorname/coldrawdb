# 合并指令

## 变更提案
- 提案名称：fix-backend-v1-scope-routing
- 提案目录：logos/changes/fix-backend-v1-scope-routing/

## 提案内容

# 变更提案：fix-backend-v1-scope-routing

> module: core | created: 2026-08-20

## 变更原因

生产前端注册/登录时调用 `POST /api/v1/auth/register` 返回 HTTP 404。排查发现后端 `backend/src/main.rs` 中 `/api/v1` 前缀被重复注册为 5 个独立 `web::scope("/api/v1")`，actix-web 只匹配第一个 scope，导致 `auth`、`rooms`、`collab`、`bridge` 等后续 scope 下的端点全部无法访问。该问题会阻断 S03/S04/S05 所有 V2 API 的线上可用性。

## 变更类型

代码级修复 + smoke 测试补充。

## 变更范围

- 影响的需求文档：无（不改需求语义）
- 影响的功能规格：无
- 影响的业务场景：S03 用户注册/登录/Token 续期、S04 房间生命周期、S05 OT 实时协作
- 影响的 API：`auth.yaml`、`rooms.yaml`、`collab.yaml`、`bridge.yaml`、`diagrams.yaml` 中所有 `/api/v1/*` 端点（仅修复路由挂载，不改契约）
- 影响的 DB 表：无
- 影响的编排测试：无
- 影响的 smoke 测试：`logos/resources/test/smoke/core-smoke-test-cases.md`（补充 auth 端点可达性用例）

## 部署影响

- 是否需要部署：是
- 部署原因：后端路由挂载方式错误导致 V2 API 不可用，必须在运行中的后端进程验证修复效果
- 影响环境：本地 / 测试 / 预发
- 是否涉及数据迁移：否
- 是否需要回滚预案：是。可回滚本提交并重启旧后端进程；数据库 schema 未变更
- 是否需要 smoke：是

## UI/UX 变更声明

```yaml
ui_impact: false
design_system_mode: generated
design_system_fallback_reason: ""
pages: []
```

## 变更概述

本次修复将 `backend/src/main.rs` 中重复的 `/api/v1` scope 合并为单一 scope，通过统一的 `api_v1_routes` 配置函数依次挂载 `diagrams_v1`、`auth_v1`、`rooms_v1`、`collab_v1`、`phase3_bridge` 路由，使所有 V2 API 端点同时可达。同时补充一个 smoke 用例，验证 `/api/v1/auth/register` 和 `/api/v1/auth/login` 返回非 404。修复后需重新编译后端、部署并跑 smoke。


## 需要合并的 Delta 文件

### 1. deltas/test/smoke/core-smoke-test-cases.md

- Delta 文件：`logos/changes/fix-backend-v1-scope-routing/deltas/test/smoke/core-smoke-test-cases.md`
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
   git add -A && git commit -m "docs(fix-backend-v1-scope-routing): merge spec deltas"
   然后提示用户：按更新后的规格实现代码，代码完成后运行 `openlogos verify` 验收，验收通过后明确授权执行 `openlogos archive fix-backend-v1-scope-routing`。
