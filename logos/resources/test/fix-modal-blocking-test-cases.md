# ModalRoot 条件渲染 + canvas testid 修复测试用例规格

> 模块：core | 提案：fix-modal-overlay-blocking
> 路径：`logos/resources/test/fix-modal-blocking-test-cases.md`
> 对齐参考源：`core-05-top-menu-modals.md` §4.1 + `core-01-editor-canvas.md` §5.1 + `logos/spec/smoke-report.md`

## 1. 范围

本文件覆盖 `fix-modal-overlay-blocking` 提案 B1 批次的测试用例：
- 修 `ModalRoot` 遮罩无条件渲染 bug（5/5 HP 全部因 modal 拦截点击而失败）
- 补画布容器 testid（HP-01 / HP-05 锚点缺失）
- 修复验证：Playwright 5 HP 端到端

**对应实现**：
- `frontend-rs/src/editor_panels.rs::modals::ModalRoot`（line 1456 — 遮罩条件化）
- `frontend-rs/src/editor_panels.rs`（line 1214 — cdb-canvas-container testid）
- `backend/src/main.rs`（CORS middleware — 配合 smoke HP-02 PUT 不被 preflight 拦截）

**对应 spec 测试 ID**：
- `core-05-top-menu-modals.md` §4.1 — UT-FIX-01
- `core-01-editor-canvas.md` §5.1 — UT-FIX-02
- 本文件 §3 — ST-FIX-01

## 2. UT 用例

### UT-FIX-01 — ModalRoot 在 `kind=None` 时不渲染遮罩 div

- **位置**：`frontend-rs/src/editor_panels.rs::modals` 测试模块
- **目标**：验证 ModalRoot 的 `cdb-modal-overlay` div 受 `kind` signal 控制
- **方法**：
  - 编译时断言：`grep "class=\"cdb-modal-overlay\"" src/editor_panels.rs` 输出 ≤ 1 处（即声明点而非无条件实例化）
  - 运行时断言：挂载 `<ModalRoot kind=signal_with_none ... />` 后，DOM 中不应有 `[data-testid="modal-root"]`（因为它应该在 `kind.is_none()` 时整段不渲染）
- **断言**：
  - `cdb-modal-overlay` 出现次数 == 1（仅声明点，不在展开后出现无条件实例化）
  - `kind=Some(ModalKind::New)` 时 `data-testid="modal-root"` 可见
  - `kind=None` 时 `data-testid="modal-root"` 不存在
- **失败处理**：
  - 编译时 grep 命中 0 处 → 提示 ModalRoot 被整体删除，回滚检查
  - 编译时 grep 命中 >1 处 → 提示存在遗留的旧实例化点

### UT-FIX-02 — 画布容器带 `editor-canvas` testid

- **位置**：`frontend-rs/src/editor_panels.rs`（line 1214 附近）
- **目标**：验证 `<div class="cdb-canvas-container">` 加上 `data-testid="editor-canvas"`
- **方法**：
  - 编译时断言：`grep -A1 'class="cdb-canvas-container"' src/editor_panels.rs` 紧邻行包含 `data-testid="editor-canvas"`
  - 运行时断言：`e2e-smoke.mjs` HP-01 中 `page.locator('[data-testid="editor-canvas"]').waitFor({ state: "visible" })` 通过
- **断言**：
  - 源码中 `data-testid="editor-canvas"` 至少出现 1 次
  - HP-01 PASS（Playwright 实测）

## 3. ST 用例

### ST-FIX-01 — Playwright 5/5 HP 全 PASS（端到端）

- **位置**：`frontend-rs/scripts/e2e-smoke.mjs`
- **目标**：本地 dev 部署后跑 5 大 happy path 全部 PASS
- **方法**：
  - 后端 `cargo run` 启动（:6666，含 CORS middleware）
  - 前端 `trunk serve` 启动（:8080，dev build）
  - 跑 `node frontend-rs/scripts/e2e-smoke.mjs`
- **断言**：
  - HP-01 加载空白编辑器 — `editor-canvas` testid 可见 + `editor-ready` 可见
  - HP-02 创建表 + 1s debounce auto-save + reload 持久化 — PUT 请求成功（200/204）+ 表格 reload 后仍存在
  - HP-03 字段增改 + Share 模态 URL 格式校验 — `/editor?share=...`
  - HP-04 SQL 导入模态 + parse — `解析到 1 条语句`
  - HP-05 键盘快捷键 Ctrl+Z / Ctrl+Shift+Z — app 不崩溃
- **失败处理**：
  - 任一 HP FAIL → `git revert` 整个 fix 提交链
  - HP-02 FAIL 但 modal 已修 → 检查 CORS middleware 配置
  - HP-04/05 FAIL 但 modal 已修 → 检查 testid / 焦点锚点

## 4. 与 add-frontend-completeness 的关系

本提案的 ST-FIX-01 是 `add-frontend-completeness` 提案 [deploy] section 列出的 smoke 验证的重新执行。

| 提案 | smoke 期望 | 结果 |
|---|---|---|
| add-frontend-completeness | 5/5 PASS | ❌ 0/5 FAIL（modal 拦截）→ 归档 |
| fix-modal-overlay-blocking | 5/5 PASS | ⏳ 待本提案实施后重跑 |

## 5. 通用要求

| 维度 | 要求 |
|---|---|
| 执行方式 | `node frontend-rs/scripts/e2e-smoke.mjs`（沿用 add-frontend-completeness 留下的脚本） |
| 总耗时 | < 120s |
| 失败阈值 | 任一失败 → 阻断 archive |
| 报告输出 | `logos/spec/smoke-report.md`（覆盖前次 FAIL 报告） |
| 前置条件 | 本地 dev 双进程跑起来 + DB 已初始化（11 张表） |

## 6. V1 边界

- ❌ 跨浏览器兼容（V1 仅 chromium）
- ❌ 性能压测（V1 smoke 仅功能）
- ❌ 完整回归（V1 smoke 仅 5 HP；完整回归在 UT/ST 阶段）

## 7. 对齐参考源

- `core-05-top-menu-modals.md` §4.1（遮罩行为）+ §9.3（UT-FIX-01 索引）
- `core-01-editor-canvas.md` §5.1（testid 约束）+ §5.3.1（UT-FIX-02 索引）
- `logos/spec/smoke-report.md`（前次 0/5 FAIL 证据）
- `logos/changes/archive/20260610-2122-add-frontend-completeness/`（前置提案）
- `frontend-rs/scripts/e2e-smoke.mjs`（5 HP 脚本）
- `backend/src/main.rs`（CORS middleware 实施点）
