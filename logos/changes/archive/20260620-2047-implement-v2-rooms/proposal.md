# 变更提案：implement-v2-rooms

> module: core | created: 2026-06-20

## 变更原因

S04 协作房间 API/DB 规格已完成（`rooms.yaml` + `coldrawdb-v2-rooms.sql`），且 S03 鉴权已落地，需实现 V2 后端 11 个 `/api/v1/rooms/*` 端点及对应测试。

## 变更类型

代码级

## 变更范围

- 影响的 API：`rooms.yaml` 11 端点
- 影响的 DB：`room` + `room_member` + `room_invite`（migration `0004_v2_rooms`）
- 影响的编排测试：`core-S04-room-lifecycle.json`（实现后运行）
- 测试用例：`core-S04-test-cases.md`（本批新增）

## 部署影响

- 是否需要部署：是（V2 rooms 为 staging 新能力）
- 是否涉及数据迁移：是（`0004_v2_rooms.up.sql`）
- 是否需要 smoke：否（rooms 纳入后续 E2E）

## 变更概述

1. 新增 `backend/src/rooms/` 服务层 + `rooms_v1.rs` HTTP 路由。
2. 新增 migration `0004_v2_rooms`。
3. 集成 S03 JWT Bearer + room_member 权限校验。
4. 11 个集成测试 + OpenLogos reporter（UT-S04-01~10、ST-S04-01）。

## 本批覆盖用例

- UT-S04-01 ~ UT-S04-10
- ST-S04-01
