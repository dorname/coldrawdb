# Delta: core-01b-relationship.md §4.7 — 低缩放自适应命中带宽

## MODIFIED — §4.7 R-HIT-02 / R-HIT-05 / R-HIT-06 / R-HIT-07

- **R-HIT-02 最近距离优先（fix-dense-lowzoom-relation-hit 修订）**：命中带宽为 **随 zoom 自适应的屏幕像素**，由 `rel_line_hit_px(zoom)` 给出：
  - `zoom >= 1.0` → **12**（常量 `REL_LINE_HIT_PX`，与 #51 高缩放口径一致）
  - `zoom <= 0.25` → **20**（常量 `REL_LINE_HIT_PX_LOW`，全景点选放宽）
  - 中间区间线性插值
  - 世界距离 `d` 满足 `d * zoom <= rel_line_hit_px(zoom)`
  - 带宽内多条命中仍返回**点到线距离最小**者；禁止数组序先命中即返回。
- **R-HIT-05 表与关系同时命中时的优先级**：近线优先阈值与当前 zoom 下主线带宽对齐——`should_prefer_reference_over_table` 使用 `rel_line_hit_px(zoom)`（不再固定 12）。只读分享路径同步。「选中关系」工具仍仅走关系命中。
- **R-HIT-06 端点记号命中热区**：主线距离阈值改为 `rel_line_hit_px(zoom)`；端点热区仍为 `REL_ENDPOINT_SIZE`（10px）屏幕像素。
- **R-HIT-07 AABB 预过滤**：外扩改为 `max(rel_line_hit_px(zoom), REL_ENDPOINT_SIZE) / zoom` 世界像素。

验收追加：**UT-PB-30** / **UT-PB-31**；既有 UT-PB-18/22/28/29 在 zoom=1 仍按 12px。

## ADDED — 合并注记

## 合并自 fix-dense-lowzoom-relation-hit（2026-10-01）

> DSPM 全景低缩放：自适应放宽主线命中带宽（12→最高 20），近线优先与 AABB 同步。
