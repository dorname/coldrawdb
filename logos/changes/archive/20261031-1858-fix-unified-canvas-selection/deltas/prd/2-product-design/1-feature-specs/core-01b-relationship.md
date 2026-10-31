## MODIFIED — §3.1 关系工具模式（Tool Rail 拆分）

**激活**：
- `tool-relationship-create`（创建关系）或快捷键 `R`；Viewer 下按钮 disabled。
- `tool-relationship-select`（选中关系）或快捷键 `Shift+R`；Viewer 下按钮 disabled。

**关键规则（合同）**：

| 工具 | 按钮 testid | 激活后行为 |
|---|---|---|
| 创建关系 | `tool-relationship-create` | 与既有 `ActiveTool::Relationship` 一致：从字段拖出连线 ≥ 4px 进入 `Dragging`，或依次点击源字段、目标字段创建关系。 |
| 选中关系 | `tool-relationship-select` | 命中关系线时按 `hit_test_reference_tier` 选中（`SelectionKind::Reference`），不触发创建关系手势；未命中关系时回落到统一选择路由，仍可点选/拖动表及其他图元，并支持空白清选/平移。`RelToolState` 保持 `Idle`。 |

**通用规则**：

- 拖线阈值 `DRAG_THRESHOLD = 4`px 仅作用于**创建关系**工具及默认选择态已有的字段拖连能力；选中关系工具不得进入 `Dragging`，其字段连接点也不得发起新关系。
- `Esc` / 取消工具 / 切换到其它工具：回 `Idle` 并隐藏橡皮筋。
- 选中关系工具下点击空白处：按既有 `Select` 工具空白点击语义清选；空白拖动则平移并保留选中。
- 选中关系工具下点表不再无响应：表选择与拖动行为与默认「选择 / 平移」一致；命中关系时仍优先按关系规则处理。
- 创建关系工具下点击已有关系线：不选中关系，仅当命中字段时按创建规则处理（避免与选中混淆）。
