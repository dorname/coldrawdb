## ADDED — §4.7 R-HIT-07 关系线命中 AABB 预过滤（perf-canvas-relation-hit-index）

- **R-HIT-07 关系线命中 AABB 预过滤（perf-canvas-relation-hit-index）**：为降低大图谱下的命中检测开销，`hit_test_reference_tier` 在逐条调用 `dist_to_reference` 前，必须先对每条关系计算轴对齐包围盒（AABB），并按当前 `zoom` 将 AABB 外扩 `max(8.0, REL_ENDPOINT_SIZE) / zoom` 世界像素（即同时覆盖 8px 主线命中带宽与 10px 端点热区）。若指针世界坐标落在外扩 AABB 之外，则跳过该关系的精确距离计算。任何可能命中的点必然落在外扩 AABB 内，因此预过滤不得改变 R-HIT-02 最近距离优先结果，也不得漏掉 R-HIT-06 端点热区内的命中。
