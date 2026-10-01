# Delta — core-01b-relationship.md（修改）

> merge 时按 MODIFIED 标记合并到 `logos/resources/prd/2-product-design/1-feature-specs/core-01b-relationship.md`

> 模块：core | 提案：fix-issue-51-relation-select-hit | issue：#51

## MODIFIED — §4.7 关系线命中合同（#51）

**背景**：#51 人工复现——默认选择工具下表 AABB 抢选连线；主线 8px 热区在缩放/密集汇聚下对准困难。「选中关系」工具复用同一命中函数，同样受窄热区影响。

合同条款修订：

- **R-HIT-02 最近距离优先（#51 修订）**：命中带宽为 **12 屏幕像素**（实现常量 `REL_LINE_HIT_PX`；世界距离 `d` 满足 `d * zoom <= REL_LINE_HIT_PX`）。带宽内多条关系仍返回点到线距离最小者；禁止数组序先命中即返回。
- **R-HIT-05 表与关系同时命中时的优先级（#51 修订）**：
  - 默认选择工具：若指针同时命中表 AABB 与关系线，当关系最近距离（屏幕像素）≤ **`REL_OVER_TABLE_PREFER_PX`（= `REL_LINE_HIT_PX` = 12.0）** 时**优先选中关系**。即：只要关系进入命中带宽且落在表 AABB 内，选关系；深点表体（距线超出命中带宽、关系未命中）仍选表。
  - 只读分享（`read_only`）路径与默认可编辑路径使用同一近线优先口径（避免表命中后提前 return 吞掉关系选中）。
  - 「选中关系」工具：仍仅走关系命中（不命中表）。
  - 便签 / 区域 / 字段端口优先级不变。
- **R-HIT-06**：端点记号热区仍为 `REL_ENDPOINT_SIZE`（10px）；与主线 12px 带宽取并集——任一阈值命中即计入该关系，最近距离比较取 `min(主线距离, 端点距离)`。
- **R-HIT-07**：AABB 外扩距离改为 `max(REL_LINE_HIT_PX, REL_ENDPOINT_SIZE) / zoom`（即同时覆盖 12px 主线与 10px 端点）。
- **R-HIT-01 / R-HIT-03 / R-HIT-04**：保持不变。

验收：UT-PB-18/19/22（带宽口径随 R-HIT-02 更新）/ UT-PB-24 / **UT-PB-28** / **UT-PB-29**。
