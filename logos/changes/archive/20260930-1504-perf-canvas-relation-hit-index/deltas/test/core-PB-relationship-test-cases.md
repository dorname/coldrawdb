## ADDED — perf-canvas-relation-hit-index 关系线命中 AABB 预过滤

### UT-PB-25 — 大图谱远距离点击 AABB 预过滤生效

- **位置**：`frontend-rs/src/editor_render.rs`（`hit_test_reference_tier` + `reference_aabb`）
- **前置**：构造 200 条关系，源表与目标表均匀分布在远离指针的坐标上；`zoom = 1.0`
- **步骤**：
  1. 在远离所有关系 AABB 的位置调用 `hit_test_reference_tier(...)`
  2. 通过测试探针记录 `dist_to_reference` 被调用次数
- **断言**：
  - 返回 `None`
  - `dist_to_reference` 调用次数为 0（全部被 AABB 预过滤剔除）

### UT-PB-26 — 大图谱近距离点击仍命中正确关系

- **位置**：`frontend-rs/src/editor_render.rs`（`hit_test_reference_tier` + `reference_aabb`）
- **前置**：构造 200 条关系，其中仅 `r_target` 的端点附近 9px 处被点击
- **步骤**：
  1. 调用 `hit_test_reference_tier(...)` 于 `r_target` 端点外侧 9px 处
- **断言**：
  - 返回 `Some("r_target".to_string())`
  - 所有候选关系均经过 AABB 预过滤，且结果与暴力遍历一致

### 附录 A 追加

| ID | 标题 | 对齐实现 |
|---|---|---|
| UT-PB-25 | 大图谱远距离点击 AABB 预过滤生效 | `editor_render.rs::hit_test_reference_tier` |
| UT-PB-26 | 大图谱近距离点击仍命中正确关系 | `editor_render.rs::hit_test_reference_tier` |
