# 变更提案：implement-v2-collab

> module: core | created: 2026-06-20

## 变更原因

S05 OT 协作 API/DB 规格已完成（`collab.yaml` + `coldrawdb-v2-collab.sql`），且 S03/S04 已落地，需实现 collab REST + WebSocket 及对应测试。

## 变更类型

代码级

## 变更范围

- 影响的 API：`collab.yaml`（WS `/ws/rooms/{roomId}` + REST head/ops）
- 影响的 DB：`operation` / `operation_log` / `room_collab_head`（migration `0005_v2_collab`）
- 影响的编排测试：`core-S05-ot-collab.json`
- 测试用例：`core-S05-test-cases.md`（本批新增）

## 部署影响

- 是否需要部署：是（collab-server 能力并入 backend 进程）
- 是否涉及数据迁移：是（`0005_v2_collab.up.sql`）
- 是否需要 smoke：否

## 变更概述

1. 新增 `backend/src/collab/`（service + hub + ws）。
2. 新增 `backend/src/collab_v1.rs`（REST + WS 路由）。
3. 新增 migration `0005_v2_collab`。
4. 6 个集成测试 + OpenLogos reporter（UT-C-01~05、ST-C-01）。

## 本批覆盖用例

- UT-C-01 ~ UT-C-05
- ST-C-01
