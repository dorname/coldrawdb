# 实现任务 — implement-v2-rooms

## [delta] 规格变更

- [x] 新增 `logos/resources/test/core-S04-test-cases.md`

## [code] 代码实现

- [x] migration `0004_v2_rooms.up.sql` / `.down.sql`
- [x] `backend/src/rooms/`（service）
- [x] `backend/src/rooms_v1.rs`（11 端点 + 11 测试 + reporter）
- [x] `backend/src/main.rs` 注册 rooms 路由

## [merge] 人类确认点

- [ ] 用户授权执行 `openlogos merge implement-v2-rooms`（纯代码变更，无 delta merge 需求时可跳过）

## [verify] 人类确认点

- [ ] 用户授权执行 `openlogos verify implement-v2-rooms`
