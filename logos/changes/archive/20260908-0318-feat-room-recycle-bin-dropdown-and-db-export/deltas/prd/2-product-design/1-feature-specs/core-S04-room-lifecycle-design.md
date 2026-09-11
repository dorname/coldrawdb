# Delta: prd/2-product-design/1-feature-specs/core-S04-room-lifecycle-design.md

> 提案：feat-room-recycle-bin-dropdown-and-db-export

## MODIFIED — 2.4 房间生命周期

```
[Owner 创建 room] ──绑定 diagram──→ [room:active]
        │                                    │
        ├── 生成 invite ──→ [pending invite] ──被邀请人 accept──→ [room_member 新增]
        │                                    │
        ├── Owner 改 role / 移除成员 ────────┤
        └── Owner 删除 room ──→ [room:archived]（回收站；diagram 保留，仅解除 room 关联）
                                              │
                                              ├── Owner 恢复 ──→ [room:active]（diagram 已被其他 active room 占用时 409）
                                              └── Owner 彻底删除 ──→ [room:purged]（物理删除 room/member/invite；diagram 仍保留）
```

**房间列表页删除入口（fix-overflow-menu-room-delete-listview-io）**：

- 现状缺口：删除房间能力（`DELETE /rooms/{id}` 软删除归档）此前仅在编辑器内 RoomPanel 可达，房间列表页无入口。
- 房间卡片（`room-card-*`）追加 owner 可见的删除按钮 `room-card-delete-{id}`（卡片右上角 icon 按钮，悬停显现 + 常显焦点可达；`my_role != "owner"` 不渲染）：
  - 点击**不进入房间**，弹确认模态 `modal-delete-room-list`：标题「删除房间」、正文含房间名与「删除后进入回收站，可在回收站恢复或彻底删除」说明、确认按钮 `btn-confirm-delete-room-list`（danger）、取消按钮；
  - 确认 → 调 `DELETE /api/v1/rooms/{id}`（复用既有端点）→ 成功后卡片从列表即时移除 + Toast `notice-toast`「已删除房间」；403 显示「无权限删除此房间」模态不关闭；其他错误「删除失败，请稍后重试」；
  - 与编辑器内 RoomPanel 删除语义完全一致（软删除归档，diagram 保留）。
- 卡片删除按钮点击不得触发卡片本身的「进入房间」跳转（事件拦截）。
- **删除确认文案口径（本变更修正）**：所有删除确认入口（列表页 `modal-delete-room-list` 与编辑器内 RoomPanel `DeleteRoomModal`）统一为「删除后进入回收站」语义；旧文案「删除后不可恢复，房间及其所有协作数据将被永久删除」废止（与软删除实际行为矛盾）。

**回收站（feat-room-recycle-bin-dropdown-and-db-export）**：

- 房间列表页提供「回收站」入口 `btn-room-trash`（页头区，与「创建房间」同级），点击切换到回收站视图 `room-trash-view`；回收站视图提供「返回房间列表」`btn-trash-back`。
- 回收站数据源：`GET /api/v1/rooms?archived=true`（仅返回本人为 owner 的已归档房间，按归档时间倒序）。
- 回收站列表项 `room-trash-item-{id}` 展示：房间名、归档时间（`archivedAt`）；每行两个操作：
  - 「恢复」`btn-restore-room-{id}` → `POST /api/v1/rooms/{id}/restore` → 成功 Toast「已恢复房间」并从回收站移除；409 提示「该 diagram 已在其他协作房间中，无法恢复」；403 提示「无权限恢复此房间」。
  - 「彻底删除」`btn-purge-room-{id}` → 弹二次确认模态 `modal-purge-room`：标题「彻底删除房间」、正文含房间名与「彻底删除后不可恢复，房间的协作数据（成员/邀请）将被永久删除；Diagram 保留」、确认按钮 `btn-confirm-purge-room`（danger）→ `DELETE /api/v1/rooms/{id}/permanent` → 成功 Toast「已彻底删除」并从回收站移除。
- 空回收站显示空态文案「回收站为空」。
- 回收站视图内不提供「进入房间」跳转（归档房间不可进入）。

## MODIFIED — 4. 验收条件（交互级）

在既有验收条件后追加：

##### 正常：删除后从回收站恢复

- **GIVEN** 用户为 room `r-001` 的 owner，且已将其删除（归档）
- **WHEN** 用户在 `/rooms` 点击 `[data-testid="btn-room-trash"]` 进入回收站，对 `r-001` 点击 `[data-testid="btn-restore-room-r-001"]`
- **THEN**
  - 调 `POST /api/v1/rooms/r-001/restore` 返回 200
  - 回收站列表移除该项；Toast「已恢复房间」
  - 返回房间列表后 `r-001` 重新出现

##### 正常：回收站彻底删除

- **GIVEN** 回收站中存在已归档 room `r-002`（owner 为当前用户）
- **WHEN** 用户点击 `[data-testid="btn-purge-room-r-002"]`，在 `modal-purge-room` 中点击 `btn-confirm-purge-room`
- **THEN**
  - 调 `DELETE /api/v1/rooms/r-002/permanent` 返回 204
  - 回收站列表移除该项；Toast「已彻底删除」
  - 再次调 `GET /api/v1/rooms?archived=true` 不含 `r-002`；`POST /rooms/r-002/restore` 返回 404

##### 异常：恢复冲突（diagram 已绑定其他 active room）

- **GIVEN** room `r-003` 已归档，其 diagram 已绑定另一 active room
- **WHEN** 用户在回收站对 `r-003` 点击「恢复」
- **THEN** 返回 409 `ROOM_DIAGRAM_TAKEN`；提示「该 diagram 已在其他协作房间中，无法恢复」；回收站列表不变

##### 异常：active 房间不可彻底删除

- **GIVEN** room `r-004` 处于 active 状态
- **WHEN** 调 `DELETE /api/v1/rooms/r-004/permanent`
- **THEN** 返回 404 `ROOM_NOT_FOUND`（防止绕过回收站）

## MODIFIED — 页面锚点（主原型强制）

保留并作为生产对齐目标：`rooms-list-page`、`btn-create-room`、`room-list`、`room-badge`、`btn-invite`、`room-presence`、`room-members-panel`、`invite-url`、`btn-accept-invite`、`invite-accept-page`。

fix-overflow-menu-room-delete-listview-io 新增：`room-card-delete-{id}`（owner 可见删除按钮）、`modal-delete-room-list`（列表页删除确认模态）、`btn-confirm-delete-room-list`（确认删除）。

feat-room-recycle-bin-dropdown-and-db-export 新增：`btn-room-trash`（回收站入口）、`room-trash-view`（回收站视图）、`btn-trash-back`（返回房间列表）、`room-trash-item-{id}`（回收站列表项）、`btn-restore-room-{id}`（恢复）、`btn-purge-room-{id}`（彻底删除）、`modal-purge-room`（彻底删除二次确认模态）、`btn-confirm-purge-room`（确认彻底删除）。

浮层关闭后不得遗留可拦截点击的遮罩。
