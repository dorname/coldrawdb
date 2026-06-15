# 实现任务 — redesign-phase-d-command-code

## [delta] 规格变更

- [x] `deltas/prd/2-product-design/1-feature-specs/core-01e-command-palette.md` — Command Palette 主规格（新文件）
- [x] `deltas/prd/2-product-design/1-feature-specs/core-01f-code-view.md` — SQL/DBML 全屏视图主规格（新文件）
- [x] `deltas/prd/2-product-design/1-feature-specs/core-00-information-architecture.md` — 视图模式与 z-index
- [x] `deltas/prd/2-product-design/1-feature-specs/core-04-side-panel-tabs.md` — 浏览能力迁移说明
- [x] `deltas/prd/2-product-design/1-feature-specs/core-05-top-menu-modals.md` — btn-code-view 与 View 菜单
- [x] `deltas/prd/2-product-design/1-feature-specs/core-01-editor-canvas.md` — 视图切换联动
- [x] `deltas/prd/2-product-design/2-page-design/core-01-editor-prototype.html` — Palette + Code View 原型
- [x] `deltas/test/core-PD-command-code-test-cases.md` — UT/ST 用例

## [code] 代码实现

- [ ] `editor_panels.rs` — `CommandPalette` / `CodeView` / `ViewMode` / `btn-code-view` / `Ctrl+K` 接线
- [ ] `styles.css` — `.cdb-command-palette` / `.cdb-code-view` 样式
- [ ] 单元测试 UT-PD-* 对齐 + test-results.jsonl reporter
- [ ] 可选：`e2e-smoke.mjs` 增补 ST-PD-01
