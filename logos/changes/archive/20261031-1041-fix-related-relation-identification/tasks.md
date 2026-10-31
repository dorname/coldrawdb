# 实现任务

## [delta] 规格变更

- [x] 产出 `deltas/prd/2-product-design/1-feature-specs/core-01b-relationship.md` — 完整悬停摘要、表关系明细、上下文/全局画布标签与碰撞避让合同
- [x] 产出 `deltas/test/core-PB-relationship-test-cases.md` — 新增 UT-PB-34、UT-PB-35、ST-PB-18、ST-PB-19，并同步悬停用例预期

## [code] 代码实现

- [x] 修改 `frontend-rs/src/editor_core.rs`，实现关系标签本机视图偏好及 UT-PB-35 存储边界覆盖
- [x] 修改 `frontend-rs/src/editor_panels.rs`，实现关系明细、表检查器列表、关系定位与标签开关，并交付 UT-PB-34
- [x] 修改 `frontend-rs/src/editor_render.rs`，实现完整悬停、上下文/全局标签、路径中点、碰撞避让与标签探针，并交付 UT-PB-35
- [x] 更新 `frontend-rs/src/styles.css`，完善多行悬停摘要与关系明细的可读样式
- [x] 更新 `frontend-rs/scripts/test-spec-parity-d.mjs`，交付 ST-PB-18、ST-PB-19，同步 ST-PB-12 并接入 OpenLogos reporter
- [x] 更新 `frontend-rs/tests/openlogos_reporter.rs`，登记 UT-PB-34、UT-PB-35，运行定向与全量测试
