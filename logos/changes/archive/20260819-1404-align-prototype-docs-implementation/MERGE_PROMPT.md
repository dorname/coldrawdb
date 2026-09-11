# 合并指令

## 变更提案
- 提案名称：align-prototype-docs-implementation
- 提案目录：logos/changes/align-prototype-docs-implementation/

## 提案内容

# 变更提案：align-prototype-docs-implementation

> module: core | created: 2026-08-19

## 变更原因
现行主原型 `core-01-editor-prototype.html` 已覆盖 S01～S05 的完整体验链路，且 S03/S04/S05 的产品设计、API、数据库与后端实现已经合并；但 `logos/resources/implementation/core-implementation-checklist.md` 仍明确记录生产前端待接入项，包括 S03 auth API/界面、S04 room/invite/member API/界面、S05 WebSocket/OT/presence。

本次变更用于把生产前后端实现与已合并文档、统一原型和测试矩阵对齐，消除“原型可演示、生产前端未闭环”的差距，并补齐缺失的 S03 编排测试与前端 reporter 证据。

## 变更类型
代码级为主；测试与实现状态文档同步更新。不改变已合并需求、产品设计、API 或数据库契约。

## 变更范围
- 影响的需求文档：
  - `logos/resources/prd/1-product-requirements/core-00-scenario-overview.md`（仅追溯，不改需求语义）
  - `logos/resources/prd/1-product-requirements/core-S06-mcp-service-requirements.md`（不在本次实现范围，仅保持状态不回退）
- 影响的功能规格：
  - `logos/resources/prd/2-product-design/2-page-design/core-01-editor-prototype.html`
  - `logos/resources/prd/2-product-design/1-feature-specs/core-S03-user-auth-design.md`
  - `logos/resources/prd/2-product-design/1-feature-specs/core-S04-room-lifecycle-design.md`
  - `logos/resources/prd/2-product-design/1-feature-specs/core-S05-ot-collab-design.md`
  - `logos/resources/implementation/core-implementation-checklist.md`
- 影响的业务场景：
  - S03 用户注册 / 登录 / Token 续期
  - S04 创建 / 加入协作房间
  - S05 OT 实时协作
  - S01/S02 作为回归边界，保证保存、分享加载、409 处理不退化
- 影响的 API：
  - `auth.yaml`：`/api/v1/auth/register|login|refresh|logout|me`
  - `rooms.yaml`：`/api/v1/rooms*`、`/api/v1/rooms/invites*`
  - `collab.yaml`：`/api/v1/rooms/{roomId}/collab/*`、`/ws/rooms/{roomId}`
  - `diagrams.yaml`：仅作为 S01/S02 保存与分享回归边界
- 影响的 DB 表：
  - `user`
  - `auth_token`
  - `room`
  - `room_member`
  - `room_invite`
  - `operation`
  - `operation_log`
  - `room_collab_head`
- 影响的编排测试：
  - 新增 `logos/resources/scenario/core-S03-user-auth.json`
  - 回归 `core-S04-room-lifecycle.json`
  - 回归 `core-S05-ot-collab.json`
  - 保持 `core-S01-diagram-save.json`、`core-S02-shared-link-load.json` 通过

## 部署影响
- 是否需要部署：是
- 部署原因：生产前端将开始调用已存在的 V2 auth/rooms/collab 后端能力，并新增 WebSocket 客户端路径；需要在本地与预发验证前端静态资源、REST、WS 与 reporter 输出。
- 影响环境：本地 / 测试 / 预发
- 是否涉及数据迁移：否。本次使用已合并的 V2 migrations，不新增表结构。
- 是否需要回滚预案：是。可回滚前端 V2 入口开关，保留 V1 单人编辑与 S02 分享加载链路。
- 是否需要 smoke：是

## UI/UX 变更声明

```yaml
ui_impact: true
design_system_mode: generated
design_system_fallback_reason: ""
pages:
  - id: auth
    prototype: core-01-editor-prototype.html
    description: 登录、注册、会话续期、退出登录入口生产接入
  - id: rooms
    prototype: core-01-editor-prototype.html
    description: 房间列表、创建房间、邀请、接受邀请、成员与角色管理生产接入
  - id: collab-editor
    prototype: core-01-editor-prototype.html
    description: 房间内编辑器、presence、WS 状态、远端操作、重连与 viewer 只读状态生产接入
```

## 变更概述
本次先以已合并的 S03/S04/S05 规格为边界，不重新设计需求或 API。后续实现批次需要在 `frontend-rs` 中补齐 auth/rooms/collab 数据访问层、生产 UI 入口、WebSocket/OT/presence 状态与 OpenLogos reporter 测试；在 `backend` 中只允许做契约兼容、错误码、CORS/WS 握手或测试缺口修正，不重写已通过的领域实现。

代码实现必须按批闭环：每批同时交付业务代码、对应 UT/ST/e2e 测试、写入 `logos/resources/verify/test-results.jsonl` 或既有 reporter 目标的 OpenLogos 结果。S01/S02 的保存、分享加载、导入导出、409 冲突与现有设计系统不可回退。

本提案完成后需要用户确认，之后才能产出 delta 并进入源码实现；`openlogos merge`、`openlogos verify`、部署、`openlogos smoke`、`openlogos archive` 与 `git push` 仍是人工确认点。


## 需要合并的 Delta 文件

### 1. deltas/scenario/core-S03-user-auth.json

- Delta 文件：`logos/changes/align-prototype-docs-implementation/deltas/scenario/core-S03-user-auth.json`
- 目标目录：`logos/resources/scenario/`
- 操作：读取 delta 中的 ADDED / MODIFIED / REMOVED 标记，合并到目标目录中对应的主文档

### 2. deltas/test/core-V2-production-frontend-test-cases.md

- Delta 文件：`logos/changes/align-prototype-docs-implementation/deltas/test/core-V2-production-frontend-test-cases.md`
- 目标目录：`logos/resources/test/`
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
   git add -A && git commit -m "docs(align-prototype-docs-implementation): merge spec deltas"
   然后提示用户：按更新后的规格实现代码，代码完成后运行 `openlogos verify` 验收，验收通过后明确授权执行 `openlogos archive align-prototype-docs-implementation`。
