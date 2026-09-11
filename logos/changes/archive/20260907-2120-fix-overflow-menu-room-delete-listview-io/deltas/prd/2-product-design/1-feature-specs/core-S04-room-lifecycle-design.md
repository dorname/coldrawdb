# Delta: core-S04-room-lifecycle-design.md

> 提案：fix-overflow-menu-room-delete-listview-io

## ADDED — 房间列表页删除入口（§2.4 补充）

> 锚点：追加于 §2.4「房间生命周期」状态图之后。

**房间列表页删除入口（fix-overflow-menu-room-delete-listview-io）**：

- 现状缺口：删除房间能力（`DELETE /rooms/{id}` 软删除归档）此前仅在编辑器内 RoomPanel 可达，房间列表页无入口。
- 房间卡片（`room-card-*`）追加 owner 可见的删除按钮 `room-card-delete-{id}`（卡片右上角 icon 按钮，悬停显现 + 常显焦点可达；`my_role != "owner"` 不渲染）：
  - 点击**不进入房间**，弹确认模态 `modal-delete-room-list`：标题「删除房间」、正文含房间名与「Diagram 保留，仅解除房间关联」说明、确认按钮 `btn-confirm-delete-room-list`（danger）、取消按钮；
  - 确认 → 调 `DELETE /api/v1/rooms/{id}`（复用既有端点）→ 成功后卡片从列表即时移除 + Toast `notice-toast`「已删除房间」；403 显示「无权限删除此房间」模态不关闭；其他错误「删除失败，请稍后重试」；
  - 与编辑器内 RoomPanel 删除语义完全一致（软删除归档，diagram 保留）。
- 卡片删除按钮点击不得触发卡片本身的「进入房间」跳转（事件拦截）。

## MODIFIED — 页面锚点（主原型强制）

保留并作为生产对齐目标：`rooms-list-page`、`btn-create-room`、`room-list`、`room-badge`、`btn-invite`、`room-presence`、`room-members-panel`、`invite-url`、`btn-accept-invite`、`invite-accept-page`。

本提案新增：`room-card-delete-{id}`（owner 可见删除按钮）、`modal-delete-room-list`（列表页删除确认模态）、`btn-confirm-delete-room-list`（确认删除）。

浮层关闭后不得遗留可拦截点击的遮罩。
