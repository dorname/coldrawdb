# Delta — core-S04-test-cases.md（fix-issue-45-room-rename / #45）

## ADDED — UT-S04-19 — rename_room 后端校验与权限

- **类型**：后端 service/handler 测试（`backend` crate）
- **覆盖**：`PATCH /api/v1/rooms/{roomId}`（renameRoom）
- **断言**：
  1. owner 改名 → 200，返回 RoomDetail.name 为新名；GET 回读一致
  2. editor / viewer 改名 → 403 `FORBIDDEN`
  3. 非成员改名 → 403 `NOT_A_MEMBER`
  4. name trim 后为空 / 超过 64 字符 → 422 `VALIDATION_ERROR`
  5. 房间不存在或已归档 → 404 `ROOM_NOT_FOUND`
  6. 改名不触碰绑定 diagram 文档（diagram 标题不变）
- reporter 登记 `UT-S04-19`

## ADDED — UT-S04-UI-18 — 重命名入口锚点与权限可见性

- **类型**：前端锚点测试（`frontend-rs` lib）
- **断言**：
  1. 房间卡片存在 `room-card-rename-{id}` 入口锚点，且仅 `my_role == "owner"` 渲染
  2. `modal-rename-room` / `input-rename-room` / `btn-confirm-rename-room` 锚点存在
  3. 前端预校验：trim 后空名或 >64 字符时确认按钮不放行并提示「名称需为 1–64 字符」
  4. `room-card-rename` 点击不触发卡片「进入房间」跳转（事件拦截，与 delete 按钮同口径）
- reporter 登记 `UT-S04-UI-18`

## ADDED — ST-S04-UI-17 — 重命名协作空间 e2e

- **类型**：e2e（`frontend-rs/scripts/test-spec-parity-d.mjs`）
- **步骤**：
  1. owner 创建房间（默认名）进入房间列表
  2. 点击 `room-card-rename-{id}` → 模态输入新名「评审空间-新」→ 确认
  3. 断言卡片标题即时变为新名 + Toast「已重命名」
  4. 进入房间 → `room-badge` 显示新名；刷新页面后仍为新名
  5. 修改 AppBar diagram 标题 → 返回列表，房间卡片名不受影响（字段解耦）
- reporter 登记 `ST-S04-UI-17`

## MODIFIED — 用例 ID 清单（附录）

| ID | 标题 |
|---|---|
| UT-S04-19 | rename_room 后端校验与权限（owner/403/422/404/字段解耦） |
| UT-S04-UI-18 | 重命名入口锚点与 owner 可见性 + 前端预校验 |
| ST-S04-UI-17 | 重命名协作空间 e2e：即时一致 + 刷新保持 + 字段解耦 |
