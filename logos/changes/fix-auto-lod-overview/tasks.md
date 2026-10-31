# 实现任务

## [delta] 规格变更

- [x] 产出 `deltas/prd/2-product-design/1-feature-specs/core-01-editor-canvas.md` — 定义 Auto/Table/Field 三态、自动阈值和显式覆盖
- [x] 产出 `deltas/prd/2-product-design/1-feature-specs/core-07-design-tokens.md` — 恢复自动 LOD 阈值 token 登记
- [x] 产出 `deltas/test/core-CR-canvas-test-cases.md` — 修订 UT/ST-CR-LOD-01，新增 UT/ST-CR-LOD-02

## [code] 代码实现

- [x] 修改 `frontend-rs/src/editor_core.rs` 与 `editor_render.rs`，实现三态持久化及渲染/命中共用 zoom 判档
- [x] 更新 `frontend-rs/scripts/test-spec-parity-d.mjs`，交付自动全景与显式覆盖回归并修正低缩放字段测试前置
- [x] 更新 `frontend-rs/tests/openlogos_reporter.rs`，登记 UT-CR-LOD-02，运行定向与全量测试
