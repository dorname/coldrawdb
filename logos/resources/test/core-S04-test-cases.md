## 1. 范围

S04：房间列表、创建、邀请、成员、Viewer。页面锚点对齐主原型 `rooms-list-page` / `invite-accept-page` / `room-editor-page`。

状态：后端已实现；生产前端部分接入。本提案 `implement-unified-prototype-spec-parity`（B 批）将 UI / 页面流用例落实为自动化，结果写入 `logos/resources/verify/test-results.jsonl`。不得将「规格已写」标为「生产已完成」。

## 2. UT 用例

### UT-S04-01 — 创建 room 201

- **位置**：`rooms_v1::tests::ut_s04_01_create_room_success`
- **步骤**：Owner 注册登录 → 创建 diagram → POST `/api/v1/rooms`
- **断言**：201；`ownerId` 匹配；`diagramId` 匹配

### UT-S04-02 — 同一 diagram 重复创建 409

- **位置**：`rooms_v1::tests::ut_s04_02_create_room_diagram_taken`
- **断言**：409；`code == ROOM_DIAGRAM_TAKEN`；`existingRoomId` 存在

### UT-S04-03 — diagram 不存在 404

- **位置**：`rooms_v1::tests::ut_s04_03_create_room_diagram_not_found`
- **断言**：404；`code == DIAGRAM_NOT_FOUND`

### UT-S04-04 — 创建邀请 201

- **位置**：`rooms_v1::tests::ut_s04_04_create_invite_success`
- **断言**：201；`token` / `inviteUrl` / `expiresAt` 存在

### UT-S04-05 — 匿名 preview 邀请 200

- **位置**：`rooms_v1::tests::ut_s04_05_preview_invite_success`
- **断言**：200；`roomName` / `diagramId` / `role` 正确

### UT-S04-06 — 接受邀请加入 200

- **位置**：`rooms_v1::tests::ut_s04_06_accept_invite_success`
- **断言**：200；Guest 成为 editor；成员数 2

### UT-S04-07 — 非成员访问 room 403

- **位置**：`rooms_v1::tests::ut_s04_07_get_room_not_a_member`
- **断言**：403；`code == NOT_A_MEMBER`

### UT-S04-08 — Owner 移除成员 204

- **位置**：`rooms_v1::tests::ut_s04_08_remove_member_success`
- **断言**：204；被移除用户 GET room → 403

### UT-S04-09 — Owner 不能离开 409

- **位置**：`rooms_v1::tests::ut_s04_09_owner_cannot_leave`
- **断言**：409；`code == OWNER_CANNOT_LEAVE`

### UT-S04-10 — Owner 归档 room 204

- **位置**：`rooms_v1::tests::ut_s04_10_archive_room_success`
- **断言**：204；GET room → 404 ROOM_NOT_FOUND

## 3. ST 用例

### ST-S04-01 — 完整 room 生命周期

- **位置**：`rooms_v1::tests::st_s04_01_room_lifecycle_flow`
- **步骤**：register×2 → login×2 → create diagram → create room → invite → preview → accept → members → 409 duplicate → remove → 403 guest → archive
- **断言**：与 `core-S04-room-lifecycle.json` 主链路一致

## UI / 页面流用例

| ID | 前置 | 操作 | 预期 | 状态 |
|---|---|---|---|---|
| ST-S04-UI-01 | 已登录 | 打开 `/rooms` | `room-list` 或空状态；`btn-create-room`；用户菜单 | 本提案 B 批实现 |
| ST-S04-UI-02 | 已登录 | 创建房间 | `POST /rooms`；进入 room-editor；`room-badge` 显示房间名 | 本提案 B 批实现 |
| ST-S04-UI-03 | Owner | 生成邀请 | 显示 invite URL；preview/accept 链路可用 | 本提案 B 批实现 |
| ST-S04-UI-04 | 另一用户 | 接受邀请 | 加入后进入同一 room-editor | 本提案 B 批实现 |
| ST-S04-UI-05 | Owner | 成员面板改角色/移除 | 列表即时更新；API PATCH/DELETE | 本提案 B 批实现 |
| ST-S04-UI-06 | Viewer | 新建表/改字段/邀请 | 写操作禁用或拦截；无写 API/WS op；只读提示 | 本提案 B 批实现 |
| ST-S04-UI-07 | 邀请过期 | 打开 invite | 失效页；无加入按钮 | 本提案 B 批实现 |
| ST-S04-UI-08 | Owner | RoomPanel 点「删除房间」→ 确认模态 → 确认 | `DELETE /rooms/{id}` 204；关闭模态回 rooms 页；列表刷新后该房间消失；403 显示「无权限删除此房间」模态不关闭 | p0-fix 定点 1 实现（e2e 链路待 wasm-pack/Playwright harness） |

## 既有 S04 用例补充约束

后端编排保持；前端验收必须使用上表 UI 用例，不得仅以 API 200 视为「已对齐主原型」。本提案 B 批负责落实上表自动化。
