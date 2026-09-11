# 实现任务

## [delta] 规格变更
- [x] 增补 `logos/resources/prd/2-product-design/1-feature-specs/core-01-editor-canvas.md` §5.3.1 测试 ID 索引：追加 UT-STUB-01/02 + ST-STUB-01 行
- [x] 新建 `logos/resources/test/fix-add-frontend-stub-leftover-test-cases.md` 测试用例规格
  - UT-STUB-01：`is_table_selected()` 纯函数 4 case
  - UT-STUB-02：`schedule_save()` helper 副作用契约
  - ST-STUB-01：Playwright e2e 强断言（HP-02 PUT≥1 + revision 推进；HP-03 selected class=1 + 右栏 h3 含表名）

## [code] 代码实现
- [x] **Bug B 1 行 diff**：`frontend-rs/src/editor_panels.rs:715` 列表项 `on:click` 改传 `Some(table_id.clone())` 替 `testid_for_select`
- [x] **Bug A 4 处 stub 修复**：在 AppRoot 内部新增 `let client = DiagramClient::new("http://127.0.0.1:3000");` + 私有 fn `schedule_save(client, store, current_diagram_id, current_title, debouncer, conflict, error)`
- [x] 4 处 handler 末尾 `debouncer.schedule(move || {});` 替换为 `schedule_save(&client, ...);`
- [x] 抽 `is_table_selected(selected: &Option<String>, id: &str) -> bool` 纯函数（用于 :713 `class:cdb-is-selected` 复用）
- [x] `frontend-rs/src/lib.rs` 解析 `window.location.pathname` 拿真 diagram_id，fallback `"default"`
- [x] `#[cfg(debug_assertions)]` 下通过 `<html data-cdb-revision>` 属性暴露测试钩子（HP-02 验证 revision 推进）
- [x] tests module 加 UT-STUB-01（`is_table_selected` × 4 case）+ UT-STUB-02（`schedule_save` 副作用契约，mock `DebounceTrigger` 不真发 PUT）

## [e2e] 测试代码
- [x] `frontend-rs/scripts/e2e-smoke.mjs` HP-02 强断言：PUT count >= 1 + `data-cdb-revision` >= 1
- [x] `frontend-rs/scripts/e2e-smoke.mjs` HP-03 强断言：`.cdb-list-item.cdb-is-selected` 数 = 1 + 右栏 `h3` 文本含表名

## [verify] 验收
- [x] `cd backend && cargo run`（:3000，含 CORS）
- [x] `cd frontend-rs && trunk serve`（:8080）
- [x] `cd frontend-rs && cargo test --lib` 跑 UT-STUB-01/02 + 所有现有 UT（51 passed / 2 ignored）
- [x] `node frontend-rs/scripts/e2e-smoke.mjs` 5/5 HP
- [x] 全部 PASS → 追加 UT-STUB-01/02 + ST-STUB-01 到 `logos/resources/verify/test-results.jsonl`
- [ ] 提醒用户授权 `openlogos verify fix-add-frontend-stub-leftover`（人类确认点）
- [ ] 提醒用户授权 `openlogos archive fix-add-frontend-stub-leftover`（人类确认点）

## 关键文件路径
- `frontend-rs/src/editor_panels.rs:715` — Bug B 单行
- `frontend-rs/src/editor_panels.rs:1115, 1125-1127, 1156, 1173` — Bug A 4 处
- `frontend-rs/src/editor_panels.rs:1069` — `current_diagram_id` signal（已有）
- `frontend-rs/src/lib.rs:21` — diagram_id 硬编码
- `frontend-rs/src/editor_data_access.rs:112-117` — `DiagramClient::save`（不改）
- `frontend-rs/src/editor_core.rs:150` — `store.snapshot()`（复用）
- `backend/src/diagrams_v1.rs:118` — PUT 路由（不改）
