# Delta — core-01-editor-canvas.md（修改）

> module: core | proposal: align-all-docs-to-unified-prototype

## ADDED — 0. 现行基线与实现状态

唯一现行主原型：`core-01-editor-prototype.html`。编辑器画布规格以该文件中 `room-editor` 视图为准。

| 项 | 约定 |
|---|---|
| 页面流 | 默认经 `auth → rooms → room-editor` 进入画布；历史 Landing / 空白 `/editor` 不再作为默认主路径 |
| 演示 ≠ 生产 | 协作演示控制台、模拟断线/远端光标/OT 合并仅表达体验；生产语义以 REST/WS 为准 |
| S03～S05 | **后端已实现**；**生产前端部分接入**；相对主原型的结构/视觉/交互逐项对齐，由下一变更 `implement-unified-prototype-spec-parity` 实现与验收 |
| 历史原型 | `core-03/04/05-*-prototype.html` 仅参考，不作为验收入口 |

## MODIFIED — 3.1 拖动跟线与网格对齐

### 3.1 拖动跟线与网格对齐

- **指针捕获**：表头拖动 `setPointerCapture`；指针离开命中面不得丢跟。
- **rAF 合并**：`pointermove` 只更新临时坐标；同一 `requestAnimationFrame`（原型 `schedulePaint` / 生产 `draw_canvas`）内重算表位置与关联关系路径。禁止每 move 整页 `render()` 或 `store.tables.set(整表)`。
- **松手网格**：`pointerup` 才将 `{x, y}` 量化。生产端 **`GRID_SIZE = 20`**；主原型演示网格为 12px（`GRID`），不得把原型步长写成生产合同。
- **拖动中禁止**在 pointermove 中量化（避免连线一格一格跳）。

## ADDED — §3.2 协作画布叠加层（与主原型一致）

| 元素 | `data-testid` | 行为 |
|---|---|---|
| 远端光标 | `remote-cursor` | 显示协作者指针与标签；pointer-events: none |
| 连接 Banner | `reconnect-banner` | `connection ≠ connected` 时出现；重连中 / 同步中 / 失败（含仅本地）文案与操作 |
| 关系橡皮筋 | `rel-rubber-band` | 见 `core-01b`；拖动中仅更新 `path[d]` |
| 关系提示条 | `rel-tool-hint` | 关系工具激活时显示 |

演示控制台触发的远端事件**不得**写成生产必选 UI；生产以 WS presence / OT 事件驱动同等反馈。

## MODIFIED — 5.1 渲染主体

### 5.1 渲染主体

- 画布容器：`data-testid="editor-canvas"`（主原型挂在 `canvas-shell`）。
- 右侧属性面板：**`data-testid="inspector"`**（禁止继续使用 `inspector-panel` 作为验收锚点）。
- StatusBar：`status-bar` / `ws-status` / `ot-rev`；Inspector 折叠由 `btn-inspector-toggle` 触发。

## ADDED — §5.x 响应式与 Viewer 只读

- 响应式三档与主原型 `@media` 一致（宽屏 / ≤1179 Inspector 叠层 / ≤760 ToolRail 底栏等）；窄屏可隐藏部分 AppBar 次要控件，画布与写工具仍可达。
- **Viewer**：写工具、拖表、改 Inspector 字段 disabled；仍可查看远端光标、连接 Banner、只读画布。
- 协作离线且未选「仅本地」时，写操作暂停（与主原型 `canEdit` 语义对齐）。

## ADDED — 统一原型对齐补充：6 与侧栏 / 顶部菜单的联动

- 选中表 → 打开并填充 **`inspector`**（非历史左栏 Tables Tab 作为主编辑面）。
- 创建入口：Tool Rail（`tool-add-table` / `tool-relationship` 等）；浏览/搜索：Command Palette（⌘K）。
- IO、分享、主题：经 AppBar **更多菜单**，见 `core-01d` / `core-05`。

## ADDED — §8.x V2 边界补充（对齐统一原型）

- ❌ 将主原型演示器当作生产交付物
- ❌ 以独立 S03/S04/S05 HTML 作为画布验收
- ✅ 表拖动 pointer 捕获 + rAF + 松手 `GRID_SIZE=20`
- ✅ 远端光标与连接 Banner 作为协作反馈合同（实现批次见 `implement-unified-prototype-spec-parity`）
