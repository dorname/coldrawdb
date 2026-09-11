# Delta — core-S05-ot-collab-design.md（修改）

> module: core | proposal: align-all-docs-to-unified-prototype

## ADDED — 统一原型对齐补充：文档头

> 模块：core | 场景：S05 | 版本：V2 | 优先级：P0
> 现行原型：`core-01-editor-prototype.html`
> 历史参考：`core-05-ot-collab-prototype.html`（不作为验收入口）
> 生产状态：后端已实现；生产前端 API/页面流已部分接入；相对主原型逐项对齐待下一变更 `implement-unified-prototype-spec-parity`
> 前置：S03 + S04；Viewer 可接收不可发送 op

## MODIFIED — 1. 产品类型与原型策略

| 项 | 结论 |
|---|---|
| 生产实现 | 后端 collab REST/WS 已实现；生产前端已有部分协作接入；须对齐主原型：`ws-status`、`ot-rev`、presence、远端光标、Activity、reconnect Banner、待同步队列、Viewer 边界、失败后「仅本地编辑」 |

## MODIFIED — 11. 生产实现状态

统一主原型用确定性本地事件模拟 connected/op/ack/remote_op/sync，不建立真实 WebSocket。真实 WS 服务已存在于 `/ws/rooms/{room_id}`。

**废止表述**：「生产前端未连接；S05 必须写成后端完成、前端待接入」。

**现行表述**：

1. 后端 WS/OT 已实现
2. 生产前端已有部分接入，但不足以证明逐项达到主原型
3. 规格以主原型可见状态为基线；生产语义以真实 WS 帧与 REST 为准
4. 演示控制台不得写入生产强制功能，除非场景时序已采纳

## ADDED — 可见状态不变量（对齐主原型）

| 连接态 | UI | 写操作 |
|---|---|---|
| `connected` | `ws-status` 正常；`ot-rev` 递增 | Owner/Editor optimistic + ack |
| `reconnecting` / `syncing` | 黄色/同步 Banner；显示待同步数量 | 可本地应用但须进入可见队列 |
| `failed` | 危险 Banner | 默认暂停协作写；可选「仅本地编辑」并持续警告 |
| `viewer` | 角色 Tag | 不得入队或制造 ack；仍接收 presence/remote op |

协作合并成功 → Toast/Activity；**禁止** S01 409 模态。

锚点：`ws-status`、`ot-rev`、`remote-cursor`、`activity-feed`、`reconnect-banner`、`room-presence`。
