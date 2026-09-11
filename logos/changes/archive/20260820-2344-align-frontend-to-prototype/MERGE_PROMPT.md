# 合并指令

## 变更提案
- 提案名称：align-frontend-to-prototype
- 提案目录：logos/changes/align-frontend-to-prototype/

## 提案内容

# 变更提案：align-frontend-to-prototype

> module: core | created: 2026-08-19

## 变更原因

上一轮 `align-prototype-docs-implementation` 已把 S03/S04/S05 的生产前端接入到真实 auth / rooms / collab REST 与 OT 状态，`openlogos verify` 与 smoke 均已通过。但生产前端的页面流仍未完全贴合现行唯一主原型 `core-01-editor-prototype.html`：登录后直接进入编辑器，房间能力以右侧面板呈现；而主原型的现行体验是登录/注册 → 房间列表页 → 协作编辑器。`/invite/{token}` 也应先呈现邀请接受页，再按登录状态完成接受，而不是被编辑器隐藏状态间接承载。

本次变更用于继续对齐生产前端与主原型的可见体验、页面结构、关键 `data-testid` 锚点和响应式行为。它不重新设计需求、API 或数据库，也不把静态原型中的本地模拟数据当作生产能力；生产行为仍必须通过真实后端 API/WS 或明确的只读/降级状态体现。

## 变更类型

代码级 + 测试级 + 实现状态文档同步更新。不改变已合并需求、产品设计、API 或数据库契约。

## 变更范围

- 影响的需求文档：
  - `logos/resources/prd/1-product-requirements/core-00-scenario-overview.md`（仅追溯，不改需求语义）
- 影响的功能规格：
  - `logos/resources/prd/2-product-design/2-page-design/core-01-editor-prototype.html`（视觉与交互基线，不改原型）
  - `logos/resources/prd/2-product-design/1-feature-specs/core-S03-user-auth-design.md`
  - `logos/resources/prd/2-product-design/1-feature-specs/core-S04-room-lifecycle-design.md`
  - `logos/resources/prd/2-product-design/1-feature-specs/core-S05-ot-collab-design.md`
  - `logos/resources/implementation/core-frontend-alignment-acceptance.md`
  - `logos/resources/implementation/core-implementation-checklist.md`
- 影响的业务场景：
  - S03 用户注册 / 登录 / Token 续期
  - S04 创建 / 加入协作房间
  - S05 OT 实时协作
  - S01/S02 作为回归边界，保证保存、分享加载、409 处理不退化
- 影响的 API：
  - `auth.yaml`、`rooms.yaml`、`collab.yaml`：仅作为调用契约和测试依据，不新增端点
  - `diagrams.yaml`、`bridge.yaml`：仅作为 S01/S02/IO 回归边界
- 影响的 DB 表：无新增或迁移；继续使用已合并 V1/V2 表
- 影响的编排测试：不新增后端 API 编排 JSON；新增或更新前端 OpenLogos reporter 用例与必要 e2e 脚本

## 部署影响

- 是否需要部署：是
- 部署原因：生产前端页面流、静态资源、路由与浏览器交互发生变化，需要在本地 / staging 等价环境验证登录、房间列表、邀请、编辑器和协作状态入口。
- 影响环境：本地 / 测试 / 预发
- 是否涉及数据迁移：否
- 是否需要回滚预案：是。可回滚本提案前端提交，保留上一轮已验证的编辑器入口与 API 接入能力。
- 是否需要 smoke：是

## UI/UX 变更声明

```yaml
ui_impact: true
design_system_mode: generated
design_system_fallback_reason: ""
pages:
  - id: auth
    prototype: core-01-editor-prototype.html
    description: 登录/注册页补齐主原型的双区布局、状态文案、表单错误和进入房间列表的路径
  - id: rooms
    prototype: core-01-editor-prototype.html
    description: 新增生产房间列表页，支持加载房间、创建房间、进入编辑器与用户菜单
  - id: invite
    prototype: core-01-editor-prototype.html
    description: `/invite/{token}` 独立接受页，展示 preview、登录要求、接受成功后的房间跳转
  - id: collab-editor
    prototype: core-01-editor-prototype.html
    description: 房间内编辑器继续对齐 room-badge、presence、ws-status、ot-rev、activity-feed、只读与重连提示
```

## 变更概述

本次以主原型的页面流作为前端实现基线：未登录默认进入 auth，登录或注册成功进入 `rooms-list-page`，用户选择或创建房间后进入 `room-editor-page`；`?share=` 仍绕过 auth guard 并保持匿名只读加载；`/invite/{token}` 必须可见地展示邀请信息，并在登录后调用真实 accept API。

代码实现按批交付，每批同时包含业务代码、UT/ST/e2e 或等价 reporter 覆盖、OpenLogos 结果写入。第一批对齐 auth / invite 入口，第二批对齐 rooms 列表页和创建/进入房间，第三批对齐协作编辑器可见状态、只读/重连提示与响应式布局，第四批做 S01/S02/IO/PU 回归和状态收口。

用户已授权继续执行本变更的后续确认点；仍按 OpenLogos 记录 `merge`、`verify`、部署、`smoke`、`archive` 与 `git push` 的实际命令和结果。


## 需要合并的 Delta 文件

### 1. deltas/test/core-V2-production-frontend-test-cases.md

- Delta 文件：`logos/changes/align-frontend-to-prototype/deltas/test/core-V2-production-frontend-test-cases.md`
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
   git add -A && git commit -m "docs(align-frontend-to-prototype): merge spec deltas"
   然后提示用户：按更新后的规格实现代码，代码完成后运行 `openlogos verify` 验收，验收通过后明确授权执行 `openlogos archive align-frontend-to-prototype`。
