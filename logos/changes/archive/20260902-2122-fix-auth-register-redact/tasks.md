# 实现任务 — fix-auth-register-redact

> 变更类型：代码级修复（无 delta、无 deploy）。

## [code] 代码实现

- [ ] `frontend-rs/src/editor_data_access.rs`: 新增 `pub fn redact_auth_error_by_status(status: u16) -> String`，从 `auth_error_message` 抽出 (status) → 脱敏串的子集（401/409/429/5xx/默认）
- [ ] `frontend-rs/src/editor_data_access.rs`: 重写 `auth_error_display(AuthError::Server(status, _))` 调用 `redact_auth_error_by_status(status)`，不再透传 message
- [ ] `frontend-rs/src/editor_data_access.rs`: 新增 `ut_s03_err_02_auth_error_display_redacts_server_message`（AuthError::Server(409, "alice@example.com leaked") → 不含邮箱字面量）
- [ ] `frontend-rs/src/editor_data_access.rs`: 新增 `ut_s03_err_03_auth_error_display_redacts_5xx`（AuthError::Server(503, "...") → "服务暂时不可用，请稍后重试"）
- [ ] `frontend-rs/src/editor_panels.rs`: register 错误设置 `email_error` 后 await 微任务（用 `request_animation_frame` 或 leptos `Effect::new`），让 DOM 提交完成
- [ ] `frontend-rs/scripts/test-spec-parity-a.mjs:243-244`: click submit 后 `await page.waitForFunction` 等待 `auth-email-error` textContent 非空（最长 5s）再断言

## [verify] 验收

- [ ] `cd frontend-rs && cargo test --lib` 全绿（新增 2 个 ut_s03_err 用例）
- [ ] `bash scripts/run-verify-tests.sh` 走到 spec-parity-a 阶段 ST-S03-UI-03 pass（不期待整体 verify 通过——其它批可能仍 fail）
- [ ] `openlogos verify` Gate 3.6：若仅 spec-parity-a 是剩余 fail 项，则修复后通过

## 完成记录

> 实现完成后填入 commit hash 与验证证据。