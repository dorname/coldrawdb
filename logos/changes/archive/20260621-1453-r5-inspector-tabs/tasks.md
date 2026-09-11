# 实现任务

## [delta] 规格变更
- [x] 产出 delta → `deltas/prd/2-product-design/1-feature-specs/core-01-editor-canvas.md` — R5 Inspector Tab 栅格 + 字段 Tab
- [x] 产出 delta → `deltas/prd/2-product-design/1-feature-specs/core-07-design-tokens.md` — Inspector Tab Token
- [x] 产出 delta → `deltas/test/core-SP-side-panel-test-cases.md` — 8 Tab 图标栏验收
- [x] 产出 delta → `deltas/prd/2-product-design/2-page-design/core-01-editor-prototype.html` — 原型 Tab 结构

## [code] 代码实现
- [x] `editor_panels.rs` — 图标 Tab 栅格、Fields Tab 合并、移除 RightPanel 分割
- [x] `styles.css` — `.cdb-tabs--icon-grid`、移除 45% 分割
- [x] 更新 UT-SP-09 断言（8 Tab + tab-fields）
