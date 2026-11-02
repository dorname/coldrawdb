# Delta: core-PB-relationship-test-cases.md — #74 同源几何与大图拖表回归

## MODIFIED — UT-PB-28 — 表与关系同时命中时近线优先关系（#51）

### UT-PB-28 — 只读模式表与关系同时命中时近线优先关系（#51；#74 收窄适用范围）

- **位置**：`frontend-rs/src/editor_render.rs`（`should_prefer_reference_over_table` / 只读分享路径）。
- **覆盖**：R-HIT-05 只读分支。
- **步骤**：
  1. `REL_OVER_TABLE_PREFER_PX == REL_LINE_HIT_PX`（12）；屏幕距离 12 优先关系、13 不优先。
  2. 几何回归：只读路径中表内近端口点在对齐 12px 后优先关系。
  3. 本用例不再代表可编辑默认/框选工具；后者的表优先行为由 ST-PB-21 真实指针链路验收。
- **reporter**：`UT-PB-28`。

## MODIFIED — UT-PB-31 — 低缩放近线优先与自适应带宽对齐

### UT-PB-31 — 只读低缩放近线优先与自适应带宽对齐

- **位置**：`should_prefer_reference_over_table`。
- **覆盖**：R-HIT-05 只读分支的自适应带宽。
- **断言**：zoom=0.25 时世界 56 优先 / 60 不优先；zoom=1 与 UT-PB-28 一致。可编辑默认/框选工具不再调用该优先级覆盖表命中。
- **reporter**：`UT-PB-31`。

## ADDED — UT-PB-37 — 三种线型绘制与命中的有效几何同源（#74）

- **位置**：`frontend-rs/src/editor_render.rs` 的完整参数关系命中、距离计算与 AABB 预过滤路径。
- **覆盖**：R-HIT-01/03/06/07。
- **GIVEN**：构造长表名、多字段的两张表及一条关系，使 `zoom=0.25、label_font_scale=1.5` 下的实际路径与旧 `1.0 / 1.0` 路径明显分离；分别覆盖 `bezier / orthogonal / straight`，并覆盖 Detail 与 Topology 档位。
- **WHEN**：用当前有效几何计算实际路径中点及旧几何独有采样点，调用完整参数命中函数；默认参数包装函数继续做既有兼容性断言。
- **THEN**：
  1. 三种线型、两种 Tier 下，实际可见路径中点均命中目标 `r1`，AABB 预过滤不得提前裁掉该点。
  2. 选取与当前路径屏幕距离超过命中带宽的旧几何独有点时必须返回 `None`，不得产生不可见关系热区。
  3. 端点命中和 AABB 外扩消费同一有效几何；最近距离、阈值与轮选算法本身不变。
  4. 既有默认参数包装函数保持原签名，UT-PB-18/19/24/25/26/29/30/32/33 的默认几何口径不依赖画布事件入口。
- **reporter**：登记 `UT-PB-37` 到 `logos/resources/verify/test-results.jsonl`。

## ADDED — ST-PB-21 — 54 表 / 97 关系下拖表、显式选线与悬浮条穿透（#74）

- **位置**：`frontend-rs/scripts/test-spec-parity-d.mjs`。
- **GIVEN**：真实前端画布预置确定性的 **54 表 / 97 关系**夹具，至少包含：①一条有效关系热区穿过目标表 `drag_target` 表头；②低缩放与 `label_font_scale=1.5` 下旧几何和实际几何分离的关系；③一张表头位于悬浮条非按钮区域下方的表。不得以 `svg.cdb-rel-overlay` 的 path 数量推断正式关系渲染数量。
- **WHEN**：
  1. 在默认「选择 / 平移」工具下，用 `page.mouse` 真实事件点击并拖动 `drag_target` 表头；框选工具重复同类拖表断言。
  2. 切换 `tool-relationship-select`，点击当前有效几何上的可见关系，再点击不命中关系的表体。
  3. 在 1440×900 与 1280×720 下采样悬浮条 padding/gap/缩放文本和按钮中心；从非按钮区域下方表头执行一次真实拖拽，并点击一个实际按钮。
- **THEN**：
  1. 默认与框选工具均选中目标表而非关系，拖动后 `data-table-pos` 与文档坐标发生预期变化；松手吸附和既有保存/op 通道不变。
  2. 低缩放、放大字号后，旧几何独有点不选中不可见关系；显式关系工具仍能选中有效可见关系，未命中关系时回落表选择，#51/#63 能力不回退。
  3. 悬浮条非按钮采样点的 `elementFromPoint` 命中链到达 `#editor-canvas`，其下方表可真实拖动；所有按钮中心命中按钮或可点击子元素，实际点击命令生效。
  4. 全程无 pageerror / console.error；除明确拖表与按钮视图偏好操作外无额外文档写入。
  5. 既有 ST-PB-16/17/20 与 ST-CR-02、ST-CR-MULTI-01、ST-CR-PAN-02 继续作为点线、轮选、统一路由、默认拖表、框选和平移回归锚点。
- **reporter**：登记 `ST-PB-21` 到 `logos/resources/verify/test-results.jsonl`。

### 用例登记追加

| TC ID | 描述 | 实现位置 |
|---|---|---|
| UT-PB-37 | 三种线型在低缩放/放大字号下绘制与命中有效几何同源（#74） | `frontend-rs/src/editor_render.rs` |
| ST-PB-21 | 54 表 / 97 关系下拖表、显式选线与悬浮条穿透（#74） | `frontend-rs/scripts/test-spec-parity-d.mjs` |
