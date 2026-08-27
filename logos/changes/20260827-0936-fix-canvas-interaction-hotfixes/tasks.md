# 任务清单 — fix-canvas-interaction-hotfixes

> created: 2026-08-27 | status: done

## 子问题 1 — 选中关系后画布无法拖动 ✅
- [x] `on_pointerdown` rel_tool_active 未命中字段 → 回落 pan（commit a17b710）
- [x] `on_pointerup` 新增 endpoint_drag 显式分支
- [x] 新增 `on_pointercancel` 闭包 + 监听
- [x] `on_pointerdown` 开头防御性清 stale drag_state

## 子问题 2 — 登录持久化 + 表位置读取 ✅
- [x] `persist_auth_session / restore_auth_session / clear_auth_session` (commit e0413fb)
- [x] login 成功 → persist
- [x] refresh_session → persist
- [x] logout / 401 → clear
- [x] AppRoot 启动异步验证 token 恢复 session
- [x] backend `row_f64` 依次 `<f64> / <i64> / <String>`，覆盖 INTEGER/REAL/TEXT

## 子问题 3 — 滚轮缩放漂移 + pan 方向错 ✅
- [x] `on_wheel` 显式算 `anchor = mouse - rect.left`，`pan = anchor - diag*zoom`（commit 3b41cd4）

## 验证（待用户终端跑）
- [ ] `cd frontend-rs && trunk build --release`
- [ ] `cd ../backend && cargo build --release`
- [ ] `openlogos verify` 期望 Gate 3.6 PASS
- [ ] `openlogos smoke` 期望 Gate 3.8 PASS
- [ ] 行为验收：3 个场景手动复测

## 归档
- [ ] `openlogos verify && openlogos smoke` 跑通
- [ ] 走 `/openlogos:merge fix-canvas-interaction-hotfixes`
- [ ] 走 `/openlogos:archive fix-canvas-interaction-hotfixes`