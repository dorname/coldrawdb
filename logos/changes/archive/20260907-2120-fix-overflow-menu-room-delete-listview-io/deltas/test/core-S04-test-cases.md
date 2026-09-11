# Delta: core-S04-test-cases.md

> 提案：fix-overflow-menu-room-delete-listview-io

## ADDED — ST-S04-UI-13 用例行（UI 表）

| ST-S04-UI-13 | 已登录，房间列表含 owner 房间 | 房间卡片点删除按钮 → 确认模态 → 确认 | `room-card-delete-{id}` 仅 owner 卡片渲染；点击不进入房间；`modal-delete-room-list` 含房间名；确认后 `DELETE /rooms/{id}`；卡片即时从列表移除 + Toast「已删除房间」 | fix-overflow-menu-room-delete-listview-io |

## ADDED — UT-S04-UI-13 — 房间卡片删除锚点测试

- **位置**：`frontend-rs/src/editor_panels.rs`（RoomsListPage 组件）
- **断言**（`include_str!` 锚点口径）：
  1. 存在 `room-card-delete-` testid 前缀（删除按钮）
  2. 存在 `modal-delete-room-list`（列表页删除确认模态）
  3. 删除按钮渲染受 `can_delete_room` / `my_role == "owner"` 门控
