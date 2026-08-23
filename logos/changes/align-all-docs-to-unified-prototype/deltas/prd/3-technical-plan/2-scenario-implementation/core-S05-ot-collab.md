# Delta — core-S05-ot-collab.md（修改）

> module: core | proposal: align-all-docs-to-unified-prototype

## ADDED — 统一原型对齐补充：文档头

> 版本：V2 | 前置：S03 + S04 | 现行原型：`core-01-editor-prototype.html`
> 历史参考：`core-05-ot-collab-prototype.html`（非验收入口）
> 生产状态：后端 WS/OT 已实现；生产前端已有部分协作接入；相对主原型逐项对齐待 `implement-unified-prototype-spec-parity`
> 入口页面状态：在 **`room-editor`** 内建立 WS（废止仅写 `/editor/{diagramId}?room=` 为唯一表述）
> API/DB：预期不新增；既有 `collab.yaml` + WS `/ws/rooms/{roomId}`；仅补前端连接态 / 排队 / 重连 / 本地降级映射

## MODIFIED — 1. 场景描述

- S05.1：进入 **`room-editor`** 后建立 WS
- 成功：`ws-status`「已连接 · OT 同步」；远端 500ms 内可见；**无 S01 409 模态**
- Viewer：可接收 presence / remote_op，**不可**发送 op

## MODIFIED — 2. 参与者

| 角色 | 模块 | 说明 |
|---|---|---|
| EditorUI | `frontend-rs` room-editor | Canvas / Inspector / StatusBar / Banner |
| CollabClient | `frontend-rs` | **真实** WS、op 队列、optimistic apply、sync flush |
| CollabSrv | **backend**（现行） | JWT + room_member、OT、广播；非「未落地的独立进程」前提 |
| 主原型 | HTML 本地事件模拟 | 演示 connected/op/ack/remote_op/sync；**不**建立真实 WebSocket |

## ADDED — 连接态不变量（对齐主原型）

| 连接态 | UI | 写操作（Owner/Editor） |
|---|---|---|
| `connected` | `ws-status` 正常；`ot-rev` 递增 | optimistic + 等 ack |
| `reconnecting` / `syncing` | Banner；显示**待同步数量** | 可本地应用但须入**可见队列** |
| `failed` | 危险 Banner | 默认暂停协作写；可选「仅本地编辑」并持续警告（有 S01 409 风险，须明示） |
| `viewer` | 角色 Tag | 不得入队或伪造 ack；仍收 presence/remote_op |

锚点：`ws-status`、`ot-rev`、`remote-cursor`、`activity-feed`、`reconnect-banner`、`room-presence`。

## ADDED — 统一原型对齐补充：5 / 8.3 与 S01 409

协作合并成功或 `CONFLICT_RESOLVED` → Toast / Activity；**禁止**弹出 S01 `modal-conflict`。

非 room、或用户主动「仅本地编辑」降级后的 PUT，才允许走 S01 409 路径。

## ADDED — 异常映射（前端补齐）

| 条件 | 前端 |
|---|---|
| WS close 4403 / NOT_A_MEMBER | Toast + 回 rooms 或只读降级 |
| READ_ONLY（viewer op） | 忽略发送；保持只读连接 |
| token_expired mid-session | S03 refresh 后重连；offline queue 保留 |
| sync 缺口过大 | 全量 snapshot；Activity「已同步服务器版本」 |
| 5 次重连失败 | `failed` Banner；「刷新」或「仅本地编辑」 |

## ADDED — 统一原型对齐补充：6 / 8.4 排队与重连（强调可见性）

1. 断线期间本地编辑必须进入 **可见待同步队列**（数量可在 Banner/Status 读取）
2. 重连 `sync { last_rev }` 后 flush；成功 Toast「已恢复协作」并隐藏 Banner
3. 队列不得静默丢弃

## MODIFIED — 13. V2 边界

- ✅ 依赖 S03 JWT + S04 room_member
- ✅ OT 替代 room 内 S01 409 UX
- ❌ 非 room 单人编辑仍走 S01 PUT + 409
- ✅ 主原型模拟 ≠ 生产前端逐项完成

## MODIFIED — 14. 对齐参考源

- 现行主原型：`core-01-editor-prototype.html`
- `core-S01-edit-and-save-diagram.md` — 409 仅非 OT
- `core-00-information-architecture.md` — `room-editor`
- `collab.yaml` — 既有帧契约（本提案不新增）
