# Delta — core-01b-relationship.md（修改）

> merge 时按 MODIFIED 标记合并到 `logos/resources/prd/2-product-design/1-feature-specs/core-01b-relationship.md`

> 模块：core | 提案：fix-relation-mouse-hit-precision

## MODIFIED — §4.7 关系线命中合同：线型同源 + 最近距离优先（fix-issues-42-44-mcp-and-relation-hit / #44）

**背景**：#44 命中错乱双根因——①`hit_test_reference_tier` 按 refs 数组序先命中即返回（非最近优先）；②命中几何恒为贝塞尔，忽略 `line_type`（orthogonal 折线绘制与命中不同源）。fix-relation-mouse-hit-precision 进一步发现：绘制端点记号（crow's foot / single bar，外接尺寸 `REL_ENDPOINT_SIZE=10px`）独立于主线，命中检测未覆盖端点符号热区，导致鼠标落在端点符号外延（距端点中心 ≤10px 但距主线 >8px）时无法命中。

合同条款：

- **R-HIT-01 线型同源**：命中检测几何必须与绘制几何同口径——`line_type=bezier` 用 `calc_path_tier` 贝塞尔（24 段折线近似）；`line_type=orthogonal` 用 `calc_orthogonal_path_tier` 折线顶点序列；`line_type=straight` 用 `calc_straight_path_tier` 端点连线；tier（详情/拓扑）锚定策略一致（沿用 #35 R-LOD-08）。
- **R-HIT-02 最近距离优先（fix-issue-47 修订）**：命中带宽为 **8 屏幕像素**（实现上在世界坐标计算距离 `d`，按当前 `zoom` 换算：`d * zoom <= 8.0`）。带宽内若有多条关系命中，必须返回**点到线距离最小**的那条；禁止「数组序先命中即返回」。低 zoom 时不得“隔空命中”。
- **R-HIT-03 阈值外不命中**：所有关系距离均超阈值时必须返回 None——点击空白不得误选远处关系（与 §5.15 空白点击清选语义衔接）。
- **R-HIT-04 悬停/点击同口径**：点击选中与悬浮检测（§4.8）共用同一命中函数，保证「所见即所选」。
- **R-HIT-05 命中优先级不变**：关系线命中仍排在表命中之后（连线被表遮住时点击选中表）；便签/区域/字段端口优先级不变。
- **R-HIT-06 端点记号命中热区（fix-relation-mouse-hit-precision）**：关系线两端 crow's foot / single bar 端点记号必须纳入命中检测。`dist_to_reference` 对三种 `line_type`，在主线距离基础上，额外取指针到两端端点中心 `(x1,y1)` / `(x2,y2)` 的距离，热区半径取 **`REL_ENDPOINT_SIZE`（10px，与绘制端点记号外接尺寸同源）**；单条关系最终距离 = `min(主线距离, 端点距离)`，再参与 8 屏幕像素阈值判定。保证用户点击可见端点符号即可选中关系；多线端点重叠处仍按 R-HIT-02 返回距离最小者。

验收：UT-PB-18 / UT-PB-19 / UT-PB-22 / **UT-PB-24**；e2e ST-PB-11 / **ST-PB-14**。