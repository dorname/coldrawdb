# 实现任务

> module: core | 提案：fix-modal-overlay-blocking
> 1 个批次闭环（B1），对应 3 个真实 bug（modal 遮罩 / canvas testid / backend CORS）。
> 严禁在 [delta] section 写代码任务，严禁在 [code] section 写规格任务。

---

## [delta] 规格变更

- [ ] 产出 delta 到 `deltas/prd/2-product-design/1-feature-specs/core-05-top-menu-modals.md` — 修「模态打开时显示遮罩」为「仅模态打开时显示遮罩」+ 补 B1 索引
- [ ] 产出 delta 到 `deltas/prd/2-product-design/1-feature-specs/core-01-editor-canvas.md` — §5 加「画布 testid = editor-canvas」+ B1 索引
- [ ] 新增 `logos/resources/test/fix-modal-blocking-test-cases.md`（详细定义 UT-FIX-01/02 + ST-FIX-01）

---

## [code] 批次 1 — ModalRoot 条件渲染 + canvas testid + backend CORS

**前置依赖**：无

**覆盖测试用例**（执行前先列 ID — 取自 `logos/resources/test/fix-modal-blocking-test-cases.md`）：
- **UT-FIX-01** — ModalRoot 在 `kind=None` 时不渲染遮罩 div（Rust 单测，模拟 `kind` signal 切换，断言 DOM 不含 `cdb-modal-overlay`）
- **UT-FIX-02** — `cdb-canvas-container` 包含 `data-testid="editor-canvas"`（编译期检查：通过 grep 源码确认）
- **ST-FIX-01** — Playwright e2e：5/5 HP 全部 PASS（沿用 `e2e-smoke.mjs`）

**Delta 配套**（merge 时同步合并）：
- 新增 `logos/resources/test/fix-modal-blocking-test-cases.md`
- 修改 `core-05-top-menu-modals.md` 修 1 句描述
- 修改 `core-01-editor-canvas.md` §5 补 1 条

**实施步骤**：
- [ ] 修改 `frontend-rs/src/editor_panels.rs::modals::ModalRoot`：把 `<div class="cdb-modal-overlay">` 包到 `{move || kind.get().map(|_| view! { <div ...>...</div> })}` 里，行为等价
- [ ] 修改 `frontend-rs/src/editor_panels.rs`：在 `<div class="cdb-canvas-container">` 加 `data-testid="editor-canvas"`
- [ ] 修改 `frontend-rs/src/editor_panels.rs` 头部 doc comment：`data-testid` 清单补 `editor-canvas`
- [ ] 修改 `backend/Cargo.toml`：新增 `actix-cors = "0.7"`
- [ ] 修改 `backend/src/main.rs`：
  - 引入 `actix_cors::Cors`
  - 在 HttpServer 上 wrap `.wrap(Cors::permissive())`（dev 模式），生产由 config.toml 切换
- [ ] UT-FIX-01：编写 `editor_panels::tests` 中 ModalRoot kind=None 单元测试
- [ ] UT-FIX-02：grep 验证 `editor-canvas` testid 出现在源码中（编译期断言，UT 中加一行 `assert!(src.contains("data-testid=\"editor-canvas\""))`）
- [ ] 更新 `frontend-rs/scripts/e2e-smoke.mjs`：
  - HP-01 / HP-05 用 `editor-canvas` testid（已是当前代码，**无需改**）
  - HP-02 在 modal-root 不存在的情况下才能点 btn-create-table（修好 modal 后自动恢复）
  - HP-03 同上
  - HP-04 同上
- [ ] 写入 OpenLogos reporter 到 `logos/resources/verify/test-results.jsonl`（UT-FIX-01/02 pass，ST-FIX-01 待 smoke）
- [ ] `cd frontend-rs && trunk build` 确认 wasm 体积不增长超 5MB
- [ ] 重启后端 + trunk serve，跑 `node scripts/e2e-smoke.mjs` 期望 5/5 PASS
- [ ] 失败 → 按 [deploy] rollback 计划 revert

**回滚条件**：
- ModalRoot 改成条件渲染后，任何模态打不开 → 回退
- 加 CORS 后 HP-02 仍然 FAIL（CORS 误配）→ 改 `permissive()` 为更严，或补 origin
- wasm 体积超 5MB → 拆分样式为按需加载

---

## [deploy] 部署任务

> ⚠️ 人类确认点：仅在 B1 完成 + 单元测试 + e2e smoke 全 PASS 后执行。
> 沿用 add-frontend-completeness 的本地 dev + Playwright 方案。

- [ ] 启动后端：`cd backend && cargo run`（监听 `:6666`）
- [ ] 启动前端：`cd frontend-rs && trunk serve`（监听 `:8080`）
- [ ] 浏览器 / Playwright 访问 `http://localhost:8080/`
- [ ] **人类确认** 后运行 `node frontend-rs/scripts/e2e-smoke.mjs`：期望 5/5 PASS
  - HP-01 加载空白编辑器（含 editor-canvas testid）
  - HP-02 创建表 + 1s debounce PUT（CORS 已通）→ reload 持久化
  - HP-03 字段 + Share 模态 URL
  - HP-04 SQL 导入模态 + parse
  - HP-05 键盘快捷键
- [ ] smoke 失败 → `git revert` 整个 fix 提交链
- [ ] smoke 通过 → 提醒用户授权 `openlogos archive fix-modal-overlay-blocking`

---

## 部署决策一致性自检

| 检查项 | 状态 |
|---|---|
| `proposal.md` 声明"是否需要部署：是" | ✅ |
| `tasks.md` 存在 `[deploy]` section | ✅ |
| `proposal.md` 声明"是否需要 smoke：是" | ✅ |
| `[deploy]` section 存在 | ✅ |
| `[code]` section 未混部署命令 | ✅（cargo run / trunk serve 都在 [deploy]） |
