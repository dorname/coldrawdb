# 实现任务

## [delta] 规格变更
- [x] 产出 delta 文件到 `deltas/prd/2-product-design/1-feature-specs/core-01b-relationship.md` — §4.7 修订 R-HIT-02/05/07
- [x] 产出 delta 文件到 `deltas/test/core-PB-relationship-test-cases.md` — 新增 UT-PB-28/29，修订 8px 相关断言口径

## [code] 代码实现
- [x] `frontend-rs/src/editor_render.rs`：`REL_LINE_HIT_PX` / `REL_OVER_TABLE_PREFER_PX`；命中与 AABB 改用常量；`hit_test_reference_tier` 可返回距离或新增带距离变体
- [x] `frontend-rs/src/editor_render.rs`：默认 `pointerdown` 在表∩关系时按近线优先
- [x] `frontend-rs/src/editor_render.rs`：UT-PB-28 / UT-PB-29；更新依赖 8px 的既有 UT
- [x] `frontend-rs/tests/openlogos_reporter.rs`：登记 UT-PB-28 / UT-PB-29
