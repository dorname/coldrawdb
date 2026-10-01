# Delta: core-PB-relationship-test-cases.md — UT-PB-30/31

## ADDED — fix-dense-lowzoom-relation-hit 低缩放自适应命中

### UT-PB-30 — 自适应主线命中带宽（zoom 分段）

- **位置**：`frontend-rs/src/editor_render.rs`（`rel_line_hit_px` / `hit_test_reference_tier`）
- **覆盖**：R-HIT-02（fix-dense-lowzoom-relation-hit）
- **断言**：
  1. `rel_line_hit_px(1.0) == 12`；`rel_line_hit_px(0.25) == 20`；`rel_line_hit_px(0.1) == 20`
  2. zoom=1.0：中点垂向 12px 命中、13px 不命中（与 UT-PB-29 一致）
  3. zoom=0.25：世界距离 80px（屏幕 20）命中、84px（屏幕 21）不命中
- **reporter**：`UT-PB-30`

### UT-PB-31 — 低缩放近线优先与自适应带宽对齐

- **位置**：`should_prefer_reference_over_table`
- **覆盖**：R-HIT-05（自适应）
- **断言**：
  1. zoom=0.25：世界距离 80 → 屏幕 20 ≤ 20 优先关系；世界 84 → 屏幕 21 不优先
  2. zoom=1.0：仍与 UT-PB-28 一致（12 / 13）
- **reporter**：`UT-PB-31`

### 附录 A 追加

| ID | 标题 | 对齐实现 |
|---|---|---|
| UT-PB-30 | 自适应主线命中带宽（低缩放放宽至 20px） | `rel_line_hit_px` / `hit_test_reference_tier` |
| UT-PB-31 | 低缩放近线优先与自适应带宽对齐 | `should_prefer_reference_over_table` |
