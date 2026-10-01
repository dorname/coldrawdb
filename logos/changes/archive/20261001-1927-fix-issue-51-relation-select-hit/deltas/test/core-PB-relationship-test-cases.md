# Delta — core-PB-relationship-test-cases.md（修改）

> merge 时按 ADDED / MODIFIED 标记合并到 `logos/resources/test/core-PB-relationship-test-cases.md`

> 模块：core | 提案：fix-issue-51-relation-select-hit | issue：#51

## MODIFIED — UT-PB-22 / UT-PB-18 带宽口径（随 R-HIT-02：8→12 屏幕像素）

- UT-PB-22：zoom=1.0 边界改为 **12 世界像素命中 / 13 不命中**；zoom=0.5 → 24/25；zoom=2.0 → 6/7。
- UT-PB-18：最近距离优先语义不变；阈值外用例仍用远距离点（如 −20px）验证 R-HIT-03。
- UT-PB-24：端点外侧「不应命中」点改为超出 `max(12, 10)=12` 的偏移（如 13px），避免被加宽后的主线带宽误判。

## ADDED — UT-PB-28 — 表与关系同时命中时近线优先关系

- **位置**：`frontend-rs/src/editor_render.rs`（`should_prefer_reference_over_table` / 默认 pointerdown 与 **只读分享** 路径优先级）
- **覆盖**：R-HIT-05（#51）
- **步骤**：
  1. `REL_OVER_TABLE_PREFER_PX == REL_LINE_HIT_PX`（12）；屏幕距离 12 优先关系、13 不优先
  2. 几何回归：表内近端口点（旧 6px 阈值会误选表）在对齐 12px 后 `should_prefer_reference_over_table` 为 true
- **实现备注**：`read_only`（匿名分享）路径原先表命中后 `return`，会绕过近线优先；#51 已与默认可编辑路径对齐。
- **reporter**：`UT-PB-28`

## ADDED — UT-PB-29 — 主线 12px 带宽边界

- **位置**：`hit_test_reference_tier`
- **覆盖**：R-HIT-02（#51）
- **步骤**：zoom=1.0，中点垂向偏移 12px 命中、13px 不命中
- **reporter**：`UT-PB-29`

## 索引更新

| ID | 说明 |
|----|------|
| UT-PB-28 | 表∩关系时近线优先关系（阈值=主线带宽 12px） |
| UT-PB-29 | 主线命中带宽 12 屏幕像素边界 |
