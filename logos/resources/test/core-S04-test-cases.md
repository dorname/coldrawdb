## 1. 范围

本文件覆盖场景 S04（协作房间生命周期）的 UT 与 ST 用例规格。

**对应实现**：`backend/src/rooms_v1.rs` + `backend/src/rooms/*`

**API 契约**：`logos/resources/api/rooms.yaml`

**DDL**：`logos/resources/database/coldrawdb-v2-rooms.sql`

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
