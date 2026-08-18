# Delta — core-S05-ot-collab-design.md（修改）

> module: core | proposal: align-unified-prototype-and-add-mcp

## MODIFIED — 1. 产品类型与原型策略

| 项 | 结论 |
|---|---|
| 产品类型 | Web 应用（房间内编辑器 + 实时同步层） |
| 主原型 | `core-01-editor-prototype.html`（S01～S05 唯一评审入口） |
| 原型形式 | 单文件可交互 HTML（模拟双端：本地编辑 + 远端 op / 光标 / 重连） |
| 历史参考 | `core-05-ot-collab-prototype.html`（ToolRail 控件未完整绑定，不再作为验收入口） |
| 生产实现 | 后端 collab REST/WS、OT 持久化与编排已实现；`frontend-rs` WS/OT/presence 尚未接入 |
| 视觉基准 | 在统一协作编辑器上叠加 presence、远端光标、连接态 Banner 与演示控制台 |
| 痛点关联 | **P03**——消除「邮件传 JSON + 手动 merge」；多人同时改 schema 可收敛 |

## MODIFIED — 8. 原型操作指南

打开 `logos/resources/prd/2-product-design/2-page-design/core-01-editor-prototype.html`：

| 操作 | 预期 |
|---|---|
| 进入协作编辑器 | 房间编辑器 + WS 模拟器显示已连接 |
| 「模拟 Alice 创建表」 | 画布出现 orders + Activity 条目 |
| 「模拟 Alice 光标」 | 远端光标移动 |
| 「模拟断线重连」 | Banner 流程 + rev 更新 |
| 「模拟重连失败」 | 降级 Banner |
| 「Viewer 模式」 | 只读 + 仍可见远端 op |

`core-05-ot-collab-prototype.html` 中 ToolRail 控件未完整绑定，只用于历史视觉对照，不纳入现行修复与验收。

## ADDED — 11. 生产实现状态

统一主原型用确定性本地事件模拟 connected/op/ack/remote_op/sync，不建立真实 WebSocket。真实 WS 服务已经存在于 `/ws/rooms/{room_id}`，但生产前端未连接；因此 S05 状态必须写成“后端完成、前端待接入”。

