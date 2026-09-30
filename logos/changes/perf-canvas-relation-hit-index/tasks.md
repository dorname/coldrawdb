# 实现任务

## [delta] 规格变更
- [x] 产出 delta 文件到 `deltas/prd/2-product-design/1-feature-specs/core-01b-relationship.md` — §4.7 新增 R-HIT-07 关系线命中 AABB 预过滤
- [x] 产出 delta 文件到 `deltas/test/core-PB-relationship-test-cases.md` — 新增 UT-PB-25 / UT-PB-26

## [code] 代码实现
- [ ] `frontend-rs/src/editor_render.rs`：新增 `reference_aabb` 纯函数，覆盖 bezier / orthogonal / straight 三种线型的控制点包围盒
- [ ] `frontend-rs/src/editor_render.rs`：`hit_test_reference_tier` 在 `dist_to_reference` 前加入外扩 AABB 预过滤
- [ ] `frontend-rs/src/editor_render.rs`：新增 UT-PB-25 / UT-PB-26 单元测试（含 `dist_to_reference` 调用计数探针）
- [ ] `frontend-rs/tests/openlogos_reporter.rs`：注册 UT-PB-25 / UT-PB-26
