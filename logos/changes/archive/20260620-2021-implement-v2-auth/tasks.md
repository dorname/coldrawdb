# 实现任务 — implement-v2-auth

## [delta] 规格变更

- [x] 新增 `logos/resources/test/core-S03-test-cases.md`

## [code] 代码实现

- [x] migration `0003_v2_auth.up.sql` / `.down.sql`
- [x] `backend/src/auth/`（password / jwt / service）
- [x] `backend/src/auth_v1.rs`（5 端点 + 8 测试 + reporter）
- [x] `backend/src/main.rs` 注册 auth 路由
- [x] `backend/Cargo.toml` 依赖（argon2 / jsonwebtoken / uuid / sha2 / hex / rand / chrono）

## [merge] 人类确认点

- [ ] 用户授权执行 `openlogos merge implement-v2-auth`（纯代码变更，无 delta merge 需求时可跳过）

## [verify] 人类确认点

- [ ] 用户授权执行 `openlogos verify implement-v2-auth`
