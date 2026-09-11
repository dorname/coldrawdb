# Delta — core-01a-table-and-field.md

> 模块：core | 提案：optimize-canvas-connect-and-drag

## MODIFIED — 1. 表（Table） > 1.2 表操作

### 1.2 表操作

| 操作 | 触发 | 数据变化 |
|---|---|---|
| 创建 | 侧栏"+"或快捷键 `T` | 分配 UUID + 默认坐标 |
| 重命名 | 双击表名 | `name` 变更 |
| 移动 | 拖拽标题栏 | 拖动中 `{x, y}` 为未量化视觉坐标，关联关系线每帧跟随；`pointerup` 时对齐网格后写入 undo |
| 缩放 | 拖拽右下角 | `width` 变更（高度自适应） |
| 锁定 | 右键菜单 / 工具栏 | `locked = true` 后禁止移动/编辑 |
| 复制 | Ctrl/Cmd + D | 深拷贝 + 偏移 (20, 20) |
| 删除 | Delete / 右键 | 从 diagram 中移除；级联删除字段 |
| 改色 | ColorPicker | `color` 变更 |

**移动补充**：
- 主原型网格 12px；生产端 `GRID_SIZE`（当前 20px）。对齐只发生在 pointerup。
- 拖动中不得调用整页 `render()`（原型）或 `store.tables.set`（生产）；连线必须用当前视觉坐标重算。
- `locked === true` 时 pointerdown 不得开始拖动。
