# 实现任务

## [delta] 规格变更
- [x] 产出 delta 文件到 `deltas/prd/2-product-design/1-feature-specs/core-01-editor-canvas.md` — 修订 R-LOD-02，放宽表维度卡宽上限
- [x] 产出 delta 文件到 `deltas/test/core-CR-canvas-test-cases.md` — 新增 UT-CR-TOPO-WIDTH-01 / ST-CR-TOPO-WIDTH-01

## [code] 代码实现
- [ ] `frontend-rs/src/editor_render.rs`：新增 `TABLE_WIDTH_MAX_TOPOLOGY` 常量，修改 `resolve_table_width_for` 按 tier 选择上限
- [ ] `frontend-rs/src/editor_render.rs`：新增 `UT-CR-TOPO-WIDTH-01` 单元测试
- [ ] `frontend-rs/scripts/test-spec-parity-d.mjs`：新增 `ST-CR-TOPO-WIDTH-01` e2e 测试
- [ ] `frontend-rs/tests/openlogos_reporter.rs`：注册新增 UT/ST ID
