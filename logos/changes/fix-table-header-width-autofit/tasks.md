# 实现任务

## [delta] 规格变更
- [x] 产出 delta 文件到 `deltas/prd/2-product-design/1-feature-specs/core-01-editor-canvas.md` — §5.12 R-LOD 规则表新增 R-LOD-11：表头行宽度需求不受 Tier 上限钳制（绝对上限 720px）
- [x] 产出 delta 文件到 `deltas/test/core-CR-canvas-test-cases.md` — 新增 UT-CR-HEADER-WIDTH-01 / ST-CR-HEADER-WIDTH-01

## [code] 代码实现
- [ ] `frontend-rs/src/editor_render.rs`：新增 `TABLE_WIDTH_MAX_HEADER = 720` 常量；`estimate_content_width_for` 拆分表头行需求与字段行需求；`resolve_table_width_for` auto 宽改为 `max(clamp(字段行需求), min(表头行需求, 720))`
- [ ] `frontend-rs/src/editor_render.rs`：新增 UT-CR-HEADER-WIDTH-01 单元测试（表头需求超 480 时 Detail 宽度突破上限、短表头仍回 480 内、超 720 钳制、显式 width 不受影响）
- [ ] `frontend-rs/scripts/test-spec-parity-d.mjs`：新增 ST-CR-HEADER-WIDTH-01 e2e（长中英文注释表头完整显示不截断）
- [ ] `frontend-rs/tests/openlogos_reporter.rs`：注册 UT-CR-HEADER-WIDTH-01 / ST-CR-HEADER-WIDTH-01
