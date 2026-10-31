# 实现任务

## [delta] 规格变更

- [x] 产出 `deltas/prd/2-product-design/1-feature-specs/core-01-editor-canvas.md` — 修订 R-PERF-07 精确几何缓存指纹合同
- [x] 产出 `deltas/test/core-CR-canvas-test-cases.md` — 新增 UT/ST-CR-SPRITE-02 几何一致性与缓存失效回归

## [code] 代码实现

- [ ] 修改 `frontend-rs/src/editor_render.rs`，将精确有效几何混入精灵指纹并暴露选中环/精灵尺寸探针
- [ ] 更新 `frontend-rs/scripts/test-spec-parity-d.mjs`，覆盖缩放、字号、注释模式与维度的几何一致性
- [ ] 更新 `frontend-rs/tests/openlogos_reporter.rs`，登记 UT-CR-SPRITE-02，运行定向与全量测试
