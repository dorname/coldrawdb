# 实现任务 — redesign-phase-b-relationship

## [delta] 规格变更

- [x] `deltas/prd/2-product-design/1-feature-specs/core-01b-relationship.md` — 关系工具 + 确认条
- [x] `deltas/prd/2-product-design/1-feature-specs/core-01-editor-canvas.md` — 字段级连线
- [x] `deltas/test/core-PB-relationship-test-cases.md` — UT/ST 用例

## [code] 代码实现

- [x] `editor_render.rs` — `hit_test_field`、字段级 `draw_bezier`
- [x] `editor_panels.rs` — `ActiveTool` / `RelToolState` / 确认条 / InspectorReference / PK 切换
- [x] `styles.css` — 确认条 + 关系工具提示样式
- [x] 单元测试 UT-PB-* / UT-R-* 对齐
