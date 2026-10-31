## MODIFIED — ST-PB-12 — 悬浮摘要同步完整关系定义

- **GIVEN**：具名关系包含可解析的两端字段、cardinality 与 `on_delete`。
- **WHEN**：pointermove 悬停关系线，随后移出命中区域。
- **THEN**：固定角落摘要除原有完整端点字段对外，还必须包含关系名、显示化基数、正确的 `ON DELETE` 和叠线轮选提示；悬停前后选中态不变，移出后摘要消失。不得再用旧的单行端点文案做全串相等断言，应逐项断言必备内容。
- **reporter**：继续登记 `ST-PB-12`，新增完整链路验收由 ST-PB-19 承接。

## ADDED — UT-PB-34 — 表检查器相关关系明细视图模型

- **位置**：`frontend-rs/src/editor_panels.rs` 关系明细纯函数。
- **GIVEN**：当前表 `orders` 同时具有一条出边、一条入边和一条自环；关系包含有名/空名、不同字段对、cardinality 与 `on_delete`。
- **WHEN**：以当前表 id、tables 和 references 构造检查器关系明细。
- **THEN**：
  1. 返回 3 条且顺序与 references 一致；入边、出边、自环方向正确，自环不重复。
  2. 每项保留 reference id，并生成 `源表.源字段 → 目标表.目标字段`。
  3. cardinality 转为显示标签，`on_delete` 原样透传；空关系名回退为 `未命名关系（<reference_id>）`。
  4. 同对端的两条 reference 不合并；无关表返回空集合。
- **reporter**：登记 `UT-PB-34`。

## ADDED — UT-PB-35 — 关系标签内容、偏好、路径中点与碰撞规则

- **位置**：`frontend-rs/src/editor_core.rs` 与 `frontend-rs/src/editor_render.rs` 纯函数。
- **断言**：
  1. `cdb.relation-labels` 缺省/非法值回落关闭，`on`/`off` 往返解析一致；视图偏好不进入 diagram 数据。
  2. 标签内容包含关系名、显示化基数和完整字段对，空名稳定回退；显隐满足“全局开 → 全部；全局关 → 仅选中/相关；无选中 → 无”。
  3. 贝塞尔中点与 `bezier_point(t=0.5)` 一致；直线取端点均值；正交不等长折线按总长度而非顶点序号取中点。
  4. AABB 相交检测覆盖重叠、包含和分离；候选布局中选中关系优先于相关关系、相关关系优先于普通关系，冲突普通标签可省略且不得叠字。
- **reporter**：登记 `UT-PB-35`。

## ADDED — ST-PB-18 — 大图表检查器关系识别与点击定位

- **位置**：`frontend-rs/scripts/test-spec-parity-d.mjs`。
- **GIVEN**：真实画布加载 16 表 / 19 关系电商模型；`orders` 具有来自 `users`、`addresses` 的 2 条入边，以及指向 `order_items`、`payments`、`shipments`、`refunds` 的 4 条出边；关系含不同字段对、基数与级联策略。
- **WHEN**：选中 `orders` 表，读取 Inspector“关系”区，再点击 `users.id → orders.user_id` 对应明细。
- **THEN**：
  1. 列表恰好显示 6 条 `orders` 关系，方向、关系名、字段对、基数与 `ON DELETE` 可辨认；其他表关系不混入，同对端关系不合并。
  2. 点击后 `window.__cdb_hl_probe.sel_ref_id` 等于目标 reference id，既有 `inspector-reference-form` 可见。
  3. 查看与定位前后 PUT 次数和协作 op 数不增加。
- **reporter**：登记 `ST-PB-18`。

## ADDED — ST-PB-19 — 完整悬停、上下文/全局标签与偏好持久化

- **位置**：`frontend-rs/scripts/test-spec-parity-d.mjs`。
- **GIVEN**：与 ST-PB-18 相同的 16 表 / 19 关系模型，默认未开启全局关系标签。
- **WHEN**：依次悬停关系、选中 `orders`、打开 `canvas-relation-labels`、刷新页面，并构造标签密集区域。
- **THEN**：
  1. 悬停摘要包含关系名、完整字段对、显示化基数、正确 `ON DELETE` 和轮选提示，且悬停不改选中态。
  2. 全局关闭且无选中时 `visible_ids` 为空；选中 `orders` 后实际绘制的上下文标签覆盖其可布局的相关关系，选中关系的优先级最高。
  3. 全局打开后 `show_all == true`，出现非相关普通关系标签；`visible_ids` 与 `skipped_ids` 不重叠，任何省略项均来自碰撞避让而非数据丢失。
  4. 刷新后开关保持开启，标签仍绘制；关闭后立即恢复仅上下文模式。
  5. 全流程不新增 PUT 或协作 op；既有 ST-PB-12 的端点/选中态断言同步为完整摘要口径。
- **reporter**：登记 `ST-PB-19`。

### 用例登记追加

| TC ID | 描述 | 实现位置 |
|---|---|---|
| UT-PB-34 | 入边/出边/自环关系明细视图模型（#66） | `frontend-rs/src/editor_panels.rs` |
| UT-PB-35 | 关系标签内容、偏好、路径中点与碰撞规则（#65/#66） | `frontend-rs/src/editor_core.rs` / `editor_render.rs` |
| ST-PB-18 | 16 表 / 19 关系中识别并定位当前表关系（#66） | `frontend-rs/scripts/test-spec-parity-d.mjs` |
| ST-PB-19 | 完整悬停、上下文/全局标签及持久化（#65/#66） | `frontend-rs/scripts/test-spec-parity-d.mjs` |
