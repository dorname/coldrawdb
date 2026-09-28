# Delta — core-S04-room-lifecycle-design.md（fix-issue-45-room-rename / #45）

## MODIFIED — §2.4 房间生命周期（状态图补 rename 分支）

在状态图 `├── Owner 改 role / 移除成员 ────────┤` 一行**之后**追加一行：

```
        ├── Owner 重命名 room.name ─────────┤
```

## ADDED — §2.4 末尾追加「房间重命名（fix-issue-45-room-rename）」

**房间重命名（fix-issue-45-room-rename，issue #45）**：

- 背景缺口：此前 `room.name` 仅 `POST /rooms` 创建时写入，无任何更新端点；AppBar 可编辑标题是 diagram 标题（`on_title_blur` 只存 diagram），与协作空间名是两个独立字段，互不联动。
- **语义解耦**：diagram 标题（AppBar `diagram-title`，随 diagram 文档保存）与协作空间名（`room.name`，房间列表卡片 / `room-badge` / 邀请预览展示）为两个独立字段；改任一不影响另一个。
- **改名入口**：房间列表卡片追加 owner 可见的「重命名」按钮 `room-card-rename-{id}`（与 `room-card-delete-{id}` 同区；`my_role != "owner"` 不渲染）：
  - 点击**不进入房间**（事件拦截，同删除按钮口径），弹模态 `modal-rename-room`：标题「重命名协作空间」、输入框 `input-rename-room`（预填当前名）、确认按钮 `btn-confirm-rename-room`（主按钮）、取消按钮；
  - 确认 → 前端 trim 后非空且 ≤64 字符才放行（否则原位提示「名称需为 1–64 字符」）→ 调 `PATCH /api/v1/rooms/{id}` body `{name}` → 成功后卡片标题即时更新 + Toast `notice-toast`「已重命名」；403 提示「仅房间 owner 可重命名」；422 提示校验文案；其他错误「重命名失败，请稍后重试」。
- **即时一致**：改名成功后房间列表卡片立即显示新名；编辑器内 `room-badge` 名称来源于 room detail，重进/刷新即为新名；邀请预览（`GET /rooms/invites/{token}` 的 room_name）随之更新。
- **权限**：仅 owner 可改名（`PATCH /rooms/{id}` 非 owner → 403 `FORBIDDEN`；非成员 → 403 `NOT_A_MEMBER`）；viewer/editor 无入口、调用即拒。
- **校验**：name trim 后 1–64 字符（与 `CreateRoomRequest.name` 同一口径）；非法 → 422 `VALIDATION_ERROR`。

## MODIFIED — §9.4 锚点（追加 rename 锚点）

在锚点清单中 `room-badge` 之后追加 `room-card-rename-{id}`、`modal-rename-room`、`input-rename-room`、`btn-confirm-rename-room`。
