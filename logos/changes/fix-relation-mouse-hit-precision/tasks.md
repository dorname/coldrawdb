# 实现任务

## [delta] 规格变更
- [x] 产出 delta 文件到 `deltas/prd/2-product-design/1-feature-specs/core-01b-relationship.md` — §4.7 新增 R-HIT-06 端点记号命中热区
- [x] 产出 delta 文件到 `deltas/test/core-PB-relationship-test-cases.md` — 新增 UT-PB-24 / ST-PB-14

## [code] 代码实现
- [ ] `frontend-rs/src/editor_render.rs`：`dist_to_reference` 对 bezier/orthogonal/straight 三种线型补充端点记号距离（半径 `REL_ENDPOINT_SIZE`）
- [ ] `frontend-rs/src/editor_render.rs`：新增 UT-PB-24 单元测试
- [ ] `frontend-rs/scripts/test-spec-parity-d.mjs`：新增 ST-PB-14 e2e
- [ ] `frontend-rs/tests/openlogos_reporter.rs`：注册 UT-PB-24 / ST-PB-14