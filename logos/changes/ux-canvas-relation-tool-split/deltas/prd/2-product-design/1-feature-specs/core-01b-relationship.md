## MODIFIED — §3.1 关系工具模式（Tool Rail 拆分）

**激活**：
- `tool-relationship-create`（创建关系）或快捷键 `R`；Viewer 下按钮 disabled。
- `tool-relationship-select`（选中关系）或快捷键 `Shift+R`；Viewer 下按钮 disabled。

**关键规则（合同）**：

| 工具 | 按钮 testid | 激活后行为 |
|---|---|---|
| 创建关系 | `tool-relationship-create` | 与既有 `ActiveTool::Relationship` 一致：从字段拖出连线 ≥ 4px 进入 `Dragging`，或依次点击源字段、目标字段创建关系。 |
| 选中关系 | `tool-relationship-select` | 鼠标悬停/点击画布上的关系线即可按 `hit_test_reference_tier` 命中并选中（`SelectionKind::Reference`），不触发创建关系手势；`RelToolState` 保持 `Idle`。 |

**通用规则**：

- 拖线阈值 `DRAG_THRESHOLD = 4`px 仅作用于**创建关系**工具；选中关系工具不进入 `Dragging`。
- `Esc` / 取消工具 / 切换到其它工具：回 `Idle` 并隐藏橡皮筋。
- 选中关系工具下点击空白处：按既有 `Select` 工具空白点击语义清选。
- 创建关系工具下点击已有关系线：不选中关系，仅当命中字段时按创建规则处理（避免与选中混淆）。
