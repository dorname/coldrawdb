# Delta — core-01-editor-canvas.md（#2 聚焦 / #5 多表拖动）

> 变更：fix-remote-github-issues / GitHub #2 #5

## ADDED — §交互：表聚焦（focus_table）

### 聚焦能力（CAP-CANVAS-FOCUS）

当用户需要在大画布中定位某张表时，编辑器必须把视口平移（必要时微调 zoom）使目标表进入可视区域并短暂高亮。

| 触发场景 | 行为 |
|---|---|
| 新建表成功 | 自动 `focus_table(new_id)` |
| 画布搜索 / Command Palette 选中表 | `focus_table(id)` + 选中该表 |
| 左侧列表 / ListView 点击表（跳回画布） | 切到画布视图（若当前在列表）+ `focus_table(id)` + 选中 |

**纯函数合同**（可单测）：

```text
focus_transform(table_aabb, viewport_css, current_transform) -> Transform
```

- 若表 AABB 已完全在视口内：可只高亮、不改 pan；或轻微居中（实现任选，须单测稳定）
- 若部分/完全在外：调整 `pan` 使表中心落入视口中心（允许边距 padding，建议 ≥ 40 CSS px）
- **不**强制改 zoom（除非表尺寸大于视口，此时允许缩小至刚好放入）

`data-testid`：聚焦动画/高亮可用 `table-focus-flash`（可选）；验收以 transform/选中态为准。

## ADDED — §交互：多表选中后整体拖动

### 多选拖动（CAP-CANVAS-MULTI-DRAG）

| 规则 | 规格 |
|---|---|
| 选中集合 | 框选（既有橡胶筋）或后续扩展的多选手势产生「表 id 集合」`selected_table_ids: Vec<String>`（≥2） |
| 拖动 | 在任一已选表的表头/卡体 pointerdown 并拖动 → **集合内所有表**同步加同一 `Δx/Δy`（世界坐标） |
| 松手 | 各表位置一次性落账；网格对齐规则与单表拖动一致（`GRID_SIZE=20`） |
| 关系线 | 拖动中跟手重算（与单表拖动相同 rAF 路径） |
| Undo | V1：允许多表位移记为一次命令单元（推荐 `MoveTables { before, after }`）；若本提案仅交付位移手势，至少保证松手后 store 与 dirty/保存通路正确 |
| 非目标 | 多选移动时自动吸附到网格的**额外**吸附算法仍按既有 Non-Goals（单表松手网格除外） |

## MODIFIED — Non-Goals 脚注

- 「❌ 多选移动时自动吸附到网格」仍成立；**不**禁止多选移动本身。
