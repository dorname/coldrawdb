# Delta — core-S04-room-lifecycle-design.md（修改）

> module: core | proposal: align-all-docs-to-unified-prototype

## ADDED — 统一原型对齐补充：文档头

> 模块：core | 场景：S04 | 版本：V2 | 优先级：P0
> 现行原型：`core-01-editor-prototype.html`
> 历史参考：`core-04-collab-prototype.html`（不作为验收入口）
> 生产状态：后端已实现；生产前端 API/页面流已部分接入；相对主原型逐项对齐待下一变更 `implement-unified-prototype-spec-parity`
> 前置：S03 已登录；后续：S05 OT

## MODIFIED — 1. 产品类型与原型策略

| 项 | 结论 |
|---|---|
| 生产实现 | 后端 rooms API/DB 与编排已实现；生产前端已有部分房间/邀请接入；须对齐主原型：房间列表、创建 Modal、邀请、成员 SideSheet、角色即时权限、进入 room-editor |

## MODIFIED — 8. 生产实现状态

主原型使用本地模拟房间数据。真实 room CRUD、邀请、角色权限由 backend 提供。

准确状态：

1. 后端与编排已完成
2. 生产前端已有部分调用与页面流
3. 不得仅因 `data-testid` 存在于规格就标记全栈完成
4. 逐项 UI/权限对齐合同见本提案与验收矩阵；实现见下一变更

## ADDED — 页面锚点（主原型强制）

保留并作为生产对齐目标：`rooms-list-page`、`btn-create-room`、`room-list`、`room-badge`、`btn-invite`、`room-presence`、`room-members-panel`、`invite-url`、`btn-accept-invite`、`invite-accept-page`。

浮层关闭后不得遗留可拦截点击的遮罩。

## ADDED — 角色切换反馈

角色切换必须即时更新 ToolRail、Inspector、邀请按钮、StatusBar 与可发送写操作的能力；Viewer 禁用须同时阻断事件并给出原因 Toast。
