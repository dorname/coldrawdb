## ADDED — §5.3.1 测试 ID 索引：UT-STUB-01/02 + ST-STUB-01（提案：fix-add-frontend-stub-leftover）

> 模块：core | 提案：fix-add-frontend-stub-leftover
> 路径：`deltas/prd/2-product-design/1-feature-specs/core-01-editor-canvas.md`
> 对齐参考源：`core-01-editor-canvas.md` §5.3.1 测试 ID 索引（已有 UT-CR-01~05 / ST-CR-01 / UT-FIX-02）

# 画布 save / select 链路测试 ID 索引（stub-leftover B1）

## 1. 范围

本 delta 在 §5.3.1 测试 ID 索引下追加 3 行，对应 `fix-add-frontend-stub-leftover` 提案 B1 批次：

- 加：`is_table_selected()` 纯函数 → UT-STUB-01
- 加：`schedule_save()` helper 副作用契约 → UT-STUB-02
- 加：Playwright e2e 强断言（HP-02 PUT 数 + revision 推进；HP-03 selected class + 右栏 h3）→ ST-STUB-01

## 2. 修改点（§5.3.1 追加）

**原 §5.3.1**（line 154-164，共 8 行）末尾追加 3 行：

| TC ID | 描述 | 对齐实现 |
|---|---|---|
| UT-STUB-01 | `is_table_selected()` 纯函数 4 case：Some(id) match / Some(testid-with-prefix) reject / None / mismatch | `frontend-rs/src/editor_panels.rs::is_table_selected` |
| UT-STUB-02 | `schedule_save()` helper 副作用契约：1 次调用 → `DebounceTrigger` 内部 handle 被设置（mock，不真发 PUT） | `frontend-rs/src/editor_panels.rs::AppRoot::schedule_save` |
| ST-STUB-01 | Playwright 5 HP 强断言：HP-02 `PUT count >= 1` + `window.__cdb_revision >= 1`；HP-03 `.cdb-list-item.cdb-is-selected` 数 = 1 + 右栏 `h3` 含表名 | `frontend-rs/scripts/e2e-smoke.mjs` |

> **关键约束（Bug B 防回归）**：UT-STUB-01 必须**显式**断言 `Some("table-list-item-xxx")` 输入应被拒（**false**），即函数契约明确禁止 data-testid 命名空间被误用，让 Bug B 根因被函数契约阻断。
>
> **关键约束（Bug A 防回归）**：UT-STUB-02 通过 mock `DebounceTrigger` 验证 `schedule_save` 触发 timer 注册但不依赖真网络；ST-STUB-01 强断言 e2e 跑出 ≥1 个真 PUT。

## 3. 验收要点

- HP-02：HP-02 FAIL（`expected >= 1 PUT, got 0`）**绝不再发生** —— `schedule_save` helper 强制每处 handler 都注册 debouncer 闭包，闭包内必走 `spawn_local(client.save(...))`
- HP-03：HP-03 FAIL（`btn-add-field` 30s 超时）**绝不再发生** —— `is_table_selected()` 纯函数在 `:713` `class:cdb-is-selected` 复用，e2e 强断言 selected class=1 + 右栏 h3 含表名
- 函数契约保护：`is_table_selected` 拒绝 testid 形式输入，未来如果再有命名空间混淆（e.g. 有人把 testid 字符串又传给 select），单元测试立即 fail

## 4. 对齐参考源

- `logos/changes/archive/20260611-2126-fix-modal-overlay-blocking/` — 前置提案（3/5 → follow-up）
- `logos/changes/archive/20260610-2122-add-frontend-completeness/` — 根因提案（4 处 stub 起源）
- `logos/spec/smoke-report.md` — HP-02/03 失败证据
- `frontend-rs/src/editor_panels.rs:715` — Bug B 改动点
- `frontend-rs/src/editor_panels.rs:1115, 1125-1127, 1156, 1173` — Bug A 改动点
- `frontend-rs/src/editor_data_access.rs:112-117` — `DiagramClient::save`（复用）
- `frontend-rs/src/editor_core.rs:150` — `store.snapshot()`（复用）
