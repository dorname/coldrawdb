# 变更提案：fix-auth-register-redact

> module: core | created: 2026-09-01
> 父提案：[`fix-global-entity-id-uniqueness`](../fix-global-entity-id-uniqueness/) 的解除条件首选路径

## 变更原因

**黑板条目 3 阻塞点**：开 `openlogos verify` 时 `frontend-rs/scripts/test-spec-parity-a.mjs:235` `ST-S03-UI-03` "重复邮箱显示脱敏字段错误" 失败：
- mock `/api/v1/auth/register` 返回 409 + `code: "EMAIL_EXISTS"` + `message: "reviewer@example.com 已注册；token=server-secret"`
- 测试期望：`auth-email-error` 元素 textContent 匹配 `/无法创建账户/` 且不含 `reviewer@example.com`/`token`/`secret`
- 实际：`textContent` 为空串

**根因**：`frontend-rs/src/editor_data_access.rs:175-181` `auth_error_display(AuthError::Server(_, message))` 直接透传 message 字段。虽然 `AuthClient::register()` (724 行) 构造 `AuthError::Server` 时已用 `auth_error_message(s, body)` 包装为脱敏串，但 `auth_error_display` 没有状态机兜底——若任何未来调用者绕过 `auth_error_message` 直接传原始 message，会再次泄露；且 **现有 `auth_error_message` (159 行) 已经有一份完整的 (status, code) → 脱敏串 映射但 `auth_error_display` 没用它**。

**测试为何拿到空串**：日志层面 `email_error.set(Some(message))` 确实被设值（`editor_panels.rs:2661`），但前端 mock 返回的 409 message 里 code 是 `EMAIL_EXISTS`（大写）—— `auth_error_message` (164 行) `to_ascii_uppercase()` 后能匹配 (409, "EMAIL_EXISTS") → "无法创建账户…" ✅。所以 `email_error` 实际**应该**有值。失败更可能是 Playwright race condition（点击 submit 后 `textContent` 在 signal 推送之前查询）或 mock 拦截未生效。修复同时加固 `auth_error_display` 防御并补 Playwright 同步等待。

## 变更类型

代码级修复（无 delta、无 deploy）

## 变更范围

- `frontend-rs/src/editor_data_access.rs`：
  - `auth_error_display(AuthError::Server(status, _))` 改为按 `status` 走脱敏映射（不再透传 message，防御性二次脱敏）
  - 抽取 `(status, _) → redacted message` 公共函数 `redact_auth_error_by_status(status) -> String`，供 `auth_error_display` 和未来外部调用复用
- `frontend-rs/src/editor_panels.rs`：
  - `email_error.set(Some(...))` 后增加微任务同步，确保 DOM 提交前 signal 已稳定（消除 Playwright race）
- `frontend-rs/src/editor_data_access.rs` 测试模块：
  - 新增 `ut_s03_err_02_auth_error_display_redacts_server_message` 覆盖 `AuthError::Server(409, "raw-email-leaked")` → 不含邮箱字面量
  - 新增 `ut_s03_err_03_auth_error_display_redacts_5xx` 覆盖 `AuthError::Server(503, "...")` → "服务暂时不可用"
- `frontend-rs/scripts/test-spec-parity-a.mjs:243-244`：register 提交后 `await page.waitForFunction` 等待 `auth-email-error` textContent 非空再断言，消除 race

## 部署影响

- 是否需要部署：否
- 是否涉及数据迁移：否
- 是否需要回滚预案：否（纯前端 WASM）
- 是否需要 smoke：否

## UI/UX 变更声明

```yaml
ui_impact: false
design_system_mode: generated
design_system_fallback_reason: ""
pages: []
```

## 变更概述

加固注册路径脱敏防御：让 `auth_error_display` 不再透传任何 `AuthError::Server` 的 message 字段——一律按 HTTP status 走脱敏映射，与 `auth_error_message` 的 (status, code) → 安全串映射语义对齐。同时补 Playwright 同步等待，消除现有测试 race condition。

**已否决的备选方案**：① 把 `AuthError::Server` 改成携带原始 body 而非已脱敏 message——侵入大、跨多调用点；② 在后端做脱敏——已做（`auth_v1.rs:86` 返回 `EMAIL_EXISTS` + 静态 message），前端需独立防御以防上游绕过。

**范围外**：其它 `AuthError::Network/Parse` 的硬编码文案保留——它们不依赖外部输入，无泄露风险。

## 验收

- `cd frontend-rs && cargo test --lib` 全绿（含新增 `ut_s03_err_02/03`）
- `bash scripts/run-verify-tests.sh` 的 spec-parity-a 阶段 ST-S03-UI-03 pass
- 复跑 `openlogos verify` Gate 3.6 通过（前置依赖 fix-global-entity-id-uniqueness 范围内已全绿）