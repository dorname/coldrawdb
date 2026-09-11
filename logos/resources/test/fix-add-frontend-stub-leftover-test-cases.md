# ADDED — Save handler 空闭包 + LeftPanel 侧栏 selected 链路 bug 修复测试用例规格
# 模块：core | 提案：fix-add-frontend-stub-leftover
# 路径：`logos/resources/test/fix-add-frontend-stub-leftover-test-cases.md`
# 对齐参考源：`core-01-editor-canvas.md` §5.3.1 + `logos/spec/smoke-report.md`

## 1. 范围

本文件覆盖 `fix-add-frontend-stub-leftover` 提案 B1 批次的测试用例：
- 修 Bug A：4 处 save handler `debouncer.schedule(move || {})` 空闭包 → 公共 `schedule_save()` helper
- 修 Bug B：LeftPanel 侧栏列表项 `on:click` 误传 data-testid → 传 `table.id`（1 行 diff）
- 抽 `is_table_selected()` 纯函数，函数契约明确拒绝 testid 形式输入（防 Bug B 回归）
- 强 e2e 断言：HP-02 PUT 数 + revision 推进；HP-03 selected class + 右栏 h3

**对应实现**：
- `frontend-rs/src/editor_panels.rs::AppRoot::schedule_save`（私有 fn 公共 helper）
- `frontend-rs/src/editor_panels.rs:715`（LeftPanel 列表项 1 行 diff）
- `frontend-rs/src/editor_panels.rs::is_table_selected`（纯函数）
- `frontend-rs/src/editor_panels.rs::tests` module（UT-STUB-01/02）
- `frontend-rs/src/lib.rs`（解析 `window.location.pathname`）
- `frontend-rs/scripts/e2e-smoke.mjs`（HP-02/03 强断言）

**对应 spec 测试 ID**：
- `core-01-editor-canvas.md` §5.3.1 — UT-STUB-01/02 + ST-STUB-01

## 2. UT 用例

### UT-STUB-01 — `is_table_selected()` 纯函数 4 case

- **位置**：`frontend-rs/src/editor_panels.rs::tests` module
- **目标**：验证纯函数 `is_table_selected(selected: &Option<String>, id: &str) -> bool` 行为正确 + 显式拒绝 testid 形式输入（**Bug B 防回归**）
- **方法**：
  - 在 `editor_panels.rs` 抽 `is_table_selected()` 纯函数（pub(crate) 或 pub(super) 可见性）
  - 在 :713 `class:cdb-is-selected=move || ...` 复用
  - 单元测试覆盖 4 case：
    - `is_table_selected(&Some("t1".to_string()), "t1") == true`（happy path）
    - `is_table_selected(&Some("table-list-item-t1".to_string()), "t1") == false`（**testid 形式被拒**）
    - `is_table_selected(&None, "t1") == false`（未选中）
    - `is_table_selected(&Some("t1".to_string()), "t2") == false`（mismatch）
- **断言**：
  - 4 case 全过
  - **Case 2 是 Bug B 防回归关键**：如果未来有人把 `data-testid` 字符串又传回 select 链路，单元测试立即 fail
- **失败处理**：
  - Case 2 fail → 提示"is_table_selected 接受了 testid 形式输入，命名空间混淆，参考 Bug B 根因"

### UT-STUB-02 — `schedule_save()` helper 副作用契约

- **位置**：`frontend-rs/src/editor_panels.rs::tests` module
- **目标**：验证私有 fn `schedule_save(client, store, current_diagram_id, current_title, debouncer, conflict, error)` 调 1 次 → `DebounceTrigger` 内部 handle 被设置
- **方法**：
  - mock `DebounceTrigger`（已实现 `Clone` + `Default` + `schedule` 签名固定，可直接用真实 `DebounceTrigger` 但断言 `handle.borrow().is_some()`）
  - 调 `schedule_save(...)` 1 次
  - 断言：mock debouncer 的内部 `RefCell<Option<Timeout>>` 不为空（即 timer 已注册）
  - **不真发 PUT**（避免网络依赖；真 PUT 在 ST-STUB-01 验证）
- **断言**：
  - `schedule_save` 调 1 次后 `debouncer.handle.borrow().is_some() == true`
  - 调 2 次后（debounce 语义）`handle.borrow().is_some() == true`（timer 被替换而非堆叠）
- **失败处理**：
  - 调 1 次后 handle 仍 None → 提示"schedule_save 内部未真正调 `debouncer.schedule()`，参考 Bug A 根因"

## 3. ST 用例

### ST-STUB-01 — Playwright 5/5 HP 强断言版（端到端）

- **位置**：`frontend-rs/scripts/e2e-smoke.mjs`
- **目标**：本地 dev 部署后跑 5 大 happy path 全部 PASS + 4 项强断言
- **方法**：
  - 后端 `cargo run` 启动（:6666，含 CORS middleware —— 沿用 fix-modal-overlay-blocking 实施）
  - 前端 `trunk serve` 启动（:8080，dev build）
  - 跑 `node frontend-rs/scripts/e2e-smoke.mjs`
- **断言**（**强断言**）：
  - HP-01 加载空白编辑器 — `editor-canvas` testid 可见 + `editor-ready` 可见
  - HP-02 创建表 + 1s debounce auto-save + revision 推进：
    - **`PUT count >= 1`**（接 save 链路后真有 PUT）
    - **`window.__cdb_revision >= 1`**（乐观锁 revision 推进）
    - 验证 table-list-item 在 DOM 中可见
  - HP-03 字段增改 + Share 模态 URL 格式校验：
    - **`.cdb-list-item.cdb-is-selected` 数 = 1**（点中侧栏后只有一项高亮）
    - **右栏 `h3` 文本含表名**（RightPanel 真渲染对应选中表）
    - 字段添加 + type 改为 INT
    - Share 模态 URL 含 `share=`
  - HP-04 SQL 导入模态 + parse — `解析到 1 条语句`
  - HP-05 键盘快捷键 Ctrl+Z / Ctrl+Shift+Z — app 不崩溃
- **失败处理**：
  - HP-02 PUT=0 → 提示"save 链路未接通，回查 `schedule_save` helper 是否被 4 处 handler 调"
  - HP-02 revision 不推进 → 提示"`store.revision.set(r.revision)` 缺漏，回查 `schedule_save` Ok 分支"
  - HP-03 selected class != 1 → 提示"select 链路未修通，回查 `editor_panels.rs:715` 是否改回 `table_id`"
  - HP-03 右栏 h3 不含表名 → 提示"RightPanel memo 仍不匹配，回查 `selected_table_id` 来源"
  - 任一 HP FAIL → 阻断 archive

## 4. 与 fix-modal-overlay-blocking / add-frontend-completeness 的关系

| 提案 | smoke 期望 | 结果 |
|---|---|---|
| add-frontend-completeness | 5/5 PASS | ❌ 0/5 FAIL（modal 拦截）→ 归档 |
| fix-modal-overlay-blocking | 5/5 PASS | ⚠️ 3/5 PASS（HP-02/03 揭示 Bug A/B）→ 归档（VERIFICATION_FAIL 标注） |
| **fix-add-frontend-stub-leftover** | **5/5 PASS** | **⏳ 本提案实施后重跑** |

## 5. 通用要求

| 维度 | 要求 |
|---|---|
| 执行方式 | `node frontend-rs/scripts/e2e-smoke.mjs` |
| 总耗时 | < 120s |
| 失败阈值 | 任一失败 → 阻断 archive |
| 报告输出 | `logos/spec/smoke-report.md`（覆盖前次 3/5 FAIL 报告） |
| 前置条件 | 本地 dev 双进程跑起来 + DB 已初始化（11 张表）+ `lib.rs` 暴露 `window.__cdb_revision`（仅 `#[cfg(debug_assertions)]` 下） |

## 6. V1 边界

- ❌ 跨浏览器兼容（V1 仅 chromium）
- ❌ 性能压测（V1 smoke 仅功能）
- ❌ 完整回归（V1 smoke 仅 5 HP + 4 强断言；完整回归在 UT/ST 阶段）
- ❌ 跨刷新持久化（V1 仅 1s debounce 内自动保存 + reload 验证，不覆盖离线编辑冲突）

## 7. 对齐参考源

- `core-01-editor-canvas.md` §5.3.1（测试 ID 索引）
- `logos/spec/smoke-report.md`（前次 3/5 FAIL 证据）
- `logos/changes/archive/20260611-2126-fix-modal-overlay-blocking/`（前置提案）
- `logos/changes/archive/20260610-2122-add-frontend-completeness/`（根因提案）
- `frontend-rs/scripts/e2e-smoke.mjs`（5 HP 脚本）
- `frontend-rs/src/editor_data_access.rs:112-117`（`DiagramClient::save` 复用）
- `frontend-rs/src/editor_core.rs:150`（`store.snapshot()` 复用）
- `backend/src/diagrams_v1.rs:118`（PUT 路由，不改）
