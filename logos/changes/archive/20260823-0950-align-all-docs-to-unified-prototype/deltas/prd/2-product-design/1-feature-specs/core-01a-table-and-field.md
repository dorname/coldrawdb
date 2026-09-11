# Delta — core-01a-table-and-field.md（修改）

> module: core | proposal: align-all-docs-to-unified-prototype

## ADDED — 0. 现行基线与实现状态

唯一现行主原型：`core-01-editor-prototype.html`（`room-editor` 内表卡片 + `inspector`）。

| 项 | 约定 |
|---|---|
| 页面流 | 表/字段编辑发生在 `auth → rooms → room-editor` 之后的协作编辑器壳内 |
| 演示 ≠ 生产 | 主原型字段编辑为本地 state + 模拟保存；生产以 diagram REST / OT 为准 |
| 实现状态 | **后端已实现**；**生产前端部分接入**；逐项对齐待 `implement-unified-prototype-spec-parity` |
| Inspector 锚点 | **`data-testid="inspector"`**（不是 `inspector-panel`） |

## MODIFIED — 1.2 表操作

### 1.2 表操作

| 操作 | 触发 | 数据变化 |
|---|---|---|
| 移动 | 拖拽表头（`data-drag-table`） | 拖动中未量化视觉坐标；关联关系线同帧跟随；`pointerup` 对齐网格后写入 undo |

**移动补充（对齐主原型 + optimize-canvas）**：

- **捕获**：`setPointerCapture`，避免跟丢。
- **rAF**：拖动中只更新表 `left/top` 与关系 `path[d]`；不得整页重渲染或整表 store set。
- **网格**：松手对齐；生产 **`GRID_SIZE = 20`**；主原型演示网格 12px，不作生产合同。
- **Viewer / 只读**：`canEdit() === false` 时 pointerdown 不得开始拖动；Inspector 输入 disabled。
- **锁定**：`locked === true` 时同 Viewer，不得开始拖动。

## MODIFIED — 2.3 字段操作

### 2.3 字段操作

字段增删改、类型与约束的主编辑面为右侧 **Inspector**（选中表后展开），与主原型 `renderInspector` 一致：

- 表名 / 强调色
- 字段列表卡片：名称、类型、PK / NOT NULL / UNIQUE、删除
- 「添加」字段、删除数据表（确认模态）

历史左栏 Tables Tab 浏览能力已迁至 Command Palette / 画布选中；不以 V1 左栏 7 Tab 作为默认编辑路径。

## ADDED — §6.x 边界与对齐补充

- ❌ 要求生产端使用主原型 12px 网格步长
- ❌ Viewer 可拖表或改字段
- ✅ 表拖动 pointer 捕获 + rAF + 松手 `GRID_SIZE=20`
- ✅ Inspector `data-testid="inspector"` 为 e2e / 规格锚点
