# 实现任务

## [delta] 规格变更

- [x] 产出 delta 到 `deltas/prd/2-product-design/1-feature-specs/core-01b-relationship.md` — R-HIT-02 收紧 LOW；新增 R-HIT-08
- [x] 产出 delta 到 `deltas/test/core-PB-relationship-test-cases.md` — 修订 UT-PB-30/31；新增 UT-PB-32

## [code] 代码实现

- [x] `editor_render.rs`：`REL_LINE_HIT_PX_LOW=14`；`resolve_ref_hit_cluster`；`hit_test_*` 接受 `prev_selected`
- [x] pointerdown / 近线优先传入当前 `selected_ref_id`；hover 传 `None`
- [x] UT-PB-30/31 口径更新；UT-PB-32；reporter 登记
