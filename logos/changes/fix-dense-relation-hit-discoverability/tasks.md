# 实现任务

## [delta] 规格变更

- [x] 产出 `deltas/prd/2-product-design/1-feature-specs/core-01b-relationship.md` — 补充叠线轮选提示与可达性验收口径
- [x] 产出 `deltas/test/core-PB-relationship-test-cases.md` — 修改 UT-PB-20，新增 UT-PB-33 与 ST-PB-17

## [code] 代码实现

- [ ] 修改 `frontend-rs/src/editor_render.rs`，在关系悬停摘要中增加轮选提示并交付 UT-PB-33
- [ ] 更新 `frontend-rs/scripts/test-spec-parity-d.mjs`，交付 16 表 / 19 关系的 ST-PB-17 与 reporter
- [ ] 更新 `frontend-rs/tests/openlogos_reporter.rs`，登记 UT-PB-33，运行定向与全量测试
