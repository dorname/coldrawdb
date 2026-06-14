# 实现任务 — redesign-phase-c-import-export

## [delta] 规格变更

- [x] `deltas/prd/2-product-design/1-feature-specs/core-01d-import-export.md` — 导入/导出抽屉主规格（新文件）
- [x] `deltas/prd/2-product-design/1-feature-specs/core-05-top-menu-modals.md` — AppBar IO 按钮与模态降级
- [x] `deltas/prd/2-product-design/1-feature-specs/core-01-editor-canvas.md` — EmptyGuide 导入入口
- [x] `deltas/prd/2-product-design/1-feature-specs/core-00-information-architecture.md` — IO 抽屉布局增量
- [x] `deltas/prd/2-product-design/1-feature-specs/core-03-bridge-io.md` — 客户端导出 + bridge 导入对接
- [x] `deltas/prd/2-product-design/2-page-design/core-01-editor-prototype.html` — 抽屉 HTML 原型
- [x] `deltas/test/core-PC-import-export-test-cases.md` — UT/ST 用例

## [code] 代码实现

- [x] `editor_panels.rs` — `IoDrawerKind` / `ImportDrawer` / `ExportDrawer` / AppBar + EmptyGuide 接线
- [x] `editor_data_access.rs` — `import_local()` bridge 客户端
- [x] `styles.css` — `.cdb-io-drawer` 样式
- [x] 单元测试 UT-PC-* 对齐 + test-results.jsonl reporter
- [ ] 可选：`e2e-smoke.mjs` 增补 ST-PC-01
