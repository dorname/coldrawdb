# 实现任务

## [delta] 规格变更

- [x] 产出 delta 文件到 `deltas/prd/2-product-design/1-feature-specs/core-01b-relationship.md` — §4.7 修订 R-HIT-02/05/07（自适应命中带宽）
- [x] 产出 delta 文件到 `deltas/test/core-PB-relationship-test-cases.md` — 新增 UT-PB-30/31

## [code] 代码实现

- [x] `frontend-rs/src/editor_render.rs`：`rel_line_hit_px(zoom)`；命中 / AABB / 近线优先改用自适应阈值
- [x] `frontend-rs/src/editor_render.rs`：UT-PB-30 / UT-PB-31；zoom=1 既有 UT 保持 12px 口径
- [x] `frontend-rs/tests/openlogos_reporter.rs`：登记 UT-PB-30 / UT-PB-31
