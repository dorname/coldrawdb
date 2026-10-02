# Delta: core-01b-relationship.md — 精准叠线消歧

## MODIFIED — R-HIT-02 低缩放带宽

- `REL_LINE_HIT_PX_LOW`：**20 → 14** 屏幕像素（`zoom <= 0.25`）；高缩放仍为 12。

## ADDED — R-HIT-08 叠线簇点击轮选

- 带宽内候选按**屏幕距离**升序。
- 与最近线相差 ≤ `REL_HIT_CLUSTER_PX`（**3** 屏幕像素）的构成叠线簇。
- 若当前已选中簇内某关系，再次在同簇点击 → **轮到下一根**；否则取最近。
- 悬停检测不轮选（`prev_selected=None`）。
- 默认选择 / 「选中关系」/ 近线优先共用。

验收：UT-PB-30/31（14px 口径）/ **UT-PB-32**。

## ADDED — 合并注记

## 合并自 fix-relation-precise-pick（2026-10-02）

> 收紧低缩放带宽；叠线簇轮选实现精准选中。
