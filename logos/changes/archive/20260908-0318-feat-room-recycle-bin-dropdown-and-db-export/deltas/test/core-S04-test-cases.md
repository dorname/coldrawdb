# Delta: test/core-S04-test-cases.md

> 提案：feat-room-recycle-bin-dropdown-and-db-export

## ADDED — UT 用例（回收站）

### UT-S04-11 — 恢复已归档 room 200

- **位置**：`rooms_v1::tests::ut_s04_11_restore_room_success`
- **步骤**：register+login → create diagram → create room → `DELETE /rooms/{id}`（204 归档）→ `POST /rooms/{id}/restore`
- **断言**：200；`GET /rooms`（active 列表）重新包含该房间；`GET /rooms?archived=true` 不再包含；`GET /rooms/{id}` 200 且可正常读取

### UT-S04-12 — 恢复冲突 409

- **位置**：`rooms_v1::tests::ut_s04_12_restore_room_conflict`
- **步骤**：room A（diagram d）归档 → 对同一 diagram d 创建 room B（active）→ `POST /rooms/A/restore`
- **断言**：409 `ROOM_DIAGRAM_TAKEN`，响应含 `existingRoomId=B`；A 保持归档（`GET /rooms?archived=true` 仍含 A）

### UT-S04-13 — 彻底删除硬删级联

- **位置**：`rooms_v1::tests::ut_s04_13_permanent_delete_room`
- **步骤**：create room → invite（产生 room_invite 行）→ 归档 → `DELETE /rooms/{id}/permanent`
- **断言**：204；随后 `POST /rooms/{id}/restore` → 404；`GET /rooms?archived=true` 不含该房间；room 行物理不存在（库内直查）；diagram 仍存在可 GET

### UT-S04-14 — 回收站权限与状态门控

- **位置**：`rooms_v1::tests::ut_s04_14_recycle_bin_guards`
- **断言**：
  1. 非 owner（普通成员/非成员）调 restore / permanent → 403 `FORBIDDEN`
  2. active 房间直接调 permanent → 404 `ROOM_NOT_FOUND`（防绕过回收站）
  3. `GET /rooms?archived=true` 仅返回本人为 owner 的归档房间（他人 owner 的归档房间不出现）
  4. 未带 token → 401

## ADDED — UI / 页面流用例（回收站）

在 UI 用例表后追加：

| ID | 前置 | 操作 | 预期 | 状态 |
|---|---|---|---|---|
| ST-S04-UI-14 | 已登录，已删除（归档）一个 owner 房间 | `/rooms` 点 `btn-room-trash` → 回收站对该房间点恢复 | 回收站视图 `room-trash-view` 展示归档房间名与归档时间；恢复调 `POST /rooms/{id}/restore`；Toast「已恢复房间」；返回列表后房间重现 | feat-room-recycle-bin-dropdown-and-db-export |
| ST-S04-UI-15 | 回收站内有归档房间 | 点 `btn-purge-room-{id}` → `modal-purge-room` 确认 | 模态正文含「不可恢复」与「Diagram 保留」；确认调 `DELETE /rooms/{id}/permanent`；Toast「已彻底删除」；列表移除 | feat-room-recycle-bin-dropdown-and-db-export |

### UT-S04-UI-15 — 回收站锚点测试

- **位置**：`frontend-rs/src/editor_panels.rs`（RoomsListPage 组件）
- **断言**（`include_str!` 锚点口径）：
  1. 存在 `data-testid="btn-room-trash"` 与 `room-trash-view`、`btn-trash-back`
  2. 存在 `btn-restore-room-` / `btn-purge-room-` testid 前缀与 `modal-purge-room` / `btn-confirm-purge-room`
  3. RoomClient 含 `restore_room` 与 `permanently_delete_room`（或等价命名）方法，分别指向 `/restore` 与 `/permanent`
  4. DeleteRoomModal 文案不再含「删除后不可恢复」，改为含「回收站」语义

## MODIFIED — 变更记录

在变更记录追加一行：

| feat-room-recycle-bin-dropdown-and-db-export | 新增 UT-S04-11~14（回收站后端）、UT-S04-UI-15（锚点）、ST-S04-UI-14/15（e2e）；删除确认文案口径修正为「进入回收站」 |
