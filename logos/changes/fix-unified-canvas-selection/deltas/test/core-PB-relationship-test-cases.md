## ADDED — UT-PB-36 — 选择类工具字段拖连路由策略（#63）

- **位置**：`frontend-rs/src/editor_render.rs` 纯路由函数。
- **GIVEN**：只读标志与选中关系工具激活标志的不同组合。
- **WHEN**：判断是否允许从 Idle 字段连接点发起建关系。
- **THEN**：
  1. 默认可写选择态允许既有字段拖连。
  2. 选中关系工具下禁止字段连接点建关系。
  3. 只读下无论工具均禁止建关系。
  4. 此策略只约束创建手势，不得用于阻断表选择/拖动或空白平移；统一回落由 ST-PB-20 真实事件链验收。
- **reporter**：登记 `UT-PB-36`。

## ADDED — ST-PB-20 — 选中关系工具下统一选择、拖动与平移（#63）

- **位置**：`frontend-rs/scripts/test-spec-parity-d.mjs`。
- **GIVEN**：真实画布预置两张表与一条关系，激活 `tool-relationship-select`，无需切换默认工具。
- **WHEN**：依次点表、拖表、点关系、尝试从字段端口拖线、空白拖动平移。
- **THEN**：
  1. `tool-pan` 的可访问名称为「选择 / 平移」；选中关系工具激活时点表立即显示表 Inspector 与选中高亮。
  2. 拖动表后 `data-table-pos` 与协作物化文档坐标一致，松手吸附既有 20 网格；拖动中产生且仅产生既有文档操作。
  3. 点击关系仍选中正确 reference；字段端口手势不新增 reference，不进入关系创建状态。
  4. 空白拖动只改变视口且保留当前选中；全程无需切回默认工具。
  5. 既有 ST-PB-16、ST-CR-02、ST-CR-MULTI-01、ST-CR-PAN-02 分别回归点线、默认拖表、框选和空白平移。
- **reporter**：登记 `ST-PB-20`。

### 用例登记追加

| TC ID | 描述 | 实现位置 |
|---|---|---|
| UT-PB-36 | 选中关系/只读下字段拖连路由策略（#63） | `frontend-rs/src/editor_render.rs` |
| ST-PB-20 | 选中关系工具下点表、拖表、点线、禁止误建与空白平移（#63） | `frontend-rs/scripts/test-spec-parity-d.mjs` |
