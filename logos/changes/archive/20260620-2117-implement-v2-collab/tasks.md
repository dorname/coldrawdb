# 实现任务 — implement-v2-collab

## [delta] 规格变更

- [x] 新增 `logos/resources/test/core-S05-test-cases.md`

## [code] 代码实现

- [x] migration `0005_v2_collab.up.sql` / `.down.sql`
- [x] `backend/src/collab/`（service + hub + ws）
- [x] `backend/src/collab_v1.rs`（REST + WS + 6 测试 + reporter）
- [x] `backend/src/main.rs` 注册 collab 路由
- [x] `backend/Cargo.toml` 依赖（actix-web-actors / actix / actix-test）

## [merge] 人类确认点

- [ ] 用户授权执行 `openlogos merge implement-v2-collab`（纯代码变更，可跳过）

## [verify] 人类确认点

- [x] 用户授权执行 `openlogos verify implement-v2-collab`（batch 6/6 pass；全局 Gate 3.6 因 51 个历史前端用例未覆盖 FAIL，与 S03/S04 相同）
