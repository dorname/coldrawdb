# Delta: core-01b-relationship.md — 大图有效几何同源与表拖拽优先级（#74）

## MODIFIED — §4.7 关系线命中合同：线型同源 + 最近距离优先（fix-issues-42-44-mcp-and-relation-hit / #44）

**背景**：#44 命中错乱双根因——①`hit_test_reference_tier` 按 refs 数组序先命中即返回（非最近优先）；②命中几何恒为贝塞尔，忽略 `line_type`（orthogonal 折线绘制与命中不同源）。fix-relation-mouse-hit-precision 进一步发现：绘制端点记号（crow's foot / single bar，外接尺寸 `REL_ENDPOINT_SIZE=10px`）独立于主线，命中检测未覆盖端点符号热区，导致鼠标落在端点符号外延（距端点中心 ≤10px 但距主线 >8px 时）无法命中。#74 补充发现：命中路径与 AABB 预过滤硬编码 `zoom=1.0、label_scale=1.0`，与正式 Canvas 绘制的有效几何不一致；可编辑默认工具的无条件近线优先还会让穿过表头/表体的关系热区阻断拖表。

合同条款：

- **R-HIT-01 线型与有效几何同源（#74 修订）**：命中检测几何必须与绘制几何同口径——`line_type=bezier` 用 `calc_path_tier_with` 贝塞尔（24 段折线近似）；`line_type=orthogonal` 用 `calc_orthogonal_path_tier_with` 折线顶点序列；`line_type=straight` 用 `calc_straight_path_tier_with` 端点连线。绘制、主线/端点命中和 AABB 预过滤必须消费同一组当前 `tier`、`comment_mode`、`zoom` 与 `label_font_scale`，任何一项都不得不同源，运行时命中路径尤其不得把 `zoom / label_font_scale` 回退为 `1.0 / 1.0`。保留默认参数包装函数仅用于兼容既有纯函数调用，不得作为画布事件入口。
- **R-HIT-02 最近距离优先（fix-relation-precise-pick 修订）**：命中带宽为随 zoom 自适应的屏幕像素，由 `rel_line_hit_px(zoom)` 给出：`zoom >= 1.0` → **12**（`REL_LINE_HIT_PX`）；`zoom <= 0.25` → **14**（`REL_LINE_HIT_PX_LOW`）；中间线性插值。世界距离 `d` 满足 `d * zoom <= rel_line_hit_px(zoom)`。带宽内若有多条关系命中，先取最近；叠线消歧见 R-HIT-08。
- **R-HIT-03 阈值外不命中**：所有关系距离均超阈值时必须返回 None——点击空白不得误选远处关系（与 §5.15 空白点击清选语义衔接）。旧 `zoom / label_font_scale` 几何上的独有热区同样不得命中。
- **R-HIT-04 悬停/点击同口径**：点击选中与悬浮检测（§4.8）共用同一有效几何与命中函数，保证所见路径与命中路径一致。悬停不轮选；悬停只报告关系候选，不越过 R-HIT-05 改变最终点击对象优先级。
- **R-HIT-05 表与关系同时命中时的优先级（#74 修订）**：
  - **可编辑默认选择 / 框选工具**：指针命中表头或表体 AABB 时一律优先表，立即进入既有选择与拖表手势；关系不再凭近线距离抢占表。只有指针未命中任何表时，才允许在空白处按 R-HIT-02/08 选择关系。
  - **选中关系工具**：仍优先按关系规则处理，保留选择压在表上的可见关系及叠线轮选能力；只有有效几何未命中关系时才回落表/空白统一路由。不得为修复默认拖表而削弱该显式工具。
  - **只读分享 / Viewer**：没有拖表写入风险，保留 #51 既有近线优先口径，避免只读模式重新出现压在表上的关系无法查看；其命中几何同样必须满足 R-HIT-01。
  - 便签 / 区域 / 字段端口优先级不变。
- **R-HIT-06 端点记号命中热区**：`hit_test_reference_tier` 对主线距离使用 `rel_line_hit_px(zoom)`，对端点距离独立使用 `REL_ENDPOINT_SIZE`（10px）；任一阈值命中即视为该关系命中。端点坐标同样使用 R-HIT-01 的当前有效几何。
- **R-HIT-07 关系线命中 AABB 预过滤**：AABB 外扩 `max(rel_line_hit_px(zoom), REL_ENDPOINT_SIZE) / zoom` 世界像素。AABB 必须由当前有效几何计算，预过滤不得裁掉实际可见路径，也不得让旧几何独有区域进入精算。
- **R-HIT-08 叠线簇点击轮选（fix-relation-precise-pick）**：带宽内候选按屏幕距离升序；与最近线相差 ≤ **`REL_HIT_CLUSTER_PX`（3px）** 的构成叠线簇。若当前已选中簇内某关系，再次在同簇区域点击则轮到下一根；否则取最近。默认选择、「选中关系」、只读近线优先共用；悬停传 `prev=None` 不轮选。完全重合路径必须能通过在同一位置重复点击逐条选中，不得要求用户先选中不可见的几何分支。

验收：UT-PB-18/19/22 / UT-PB-24 / UT-PB-28 / UT-PB-29 / UT-PB-30 / UT-PB-31 / UT-PB-32 / UT-PB-33 / UT-PB-25 / UT-PB-26 / **UT-PB-37**；e2e ST-PB-11 / ST-PB-14 / ST-PB-17 / ST-PB-20 / **ST-PB-21**。

## ADDED — 合并自 fix-big-diagram-table-drag-blocked（2026-11-02）

> #74：可编辑默认/框选工具表优先；显式关系工具与只读能力不回退；绘制、命中和 AABB 统一当前 zoom 与有效字号。
