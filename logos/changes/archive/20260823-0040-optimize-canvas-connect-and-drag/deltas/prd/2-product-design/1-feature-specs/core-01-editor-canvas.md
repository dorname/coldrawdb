# Delta — core-01-editor-canvas.md

> 模块：core | 提案：optimize-canvas-connect-and-drag

## MODIFIED — 3. 交互手势

| 手势 | 效果 |
|---|---|
| 鼠标左键单击 | 选中对象（高亮） |
| 鼠标左键拖拽空白 | 框选多个对象 |
| 鼠标左键拖拽对象 | 移动对象；连线每帧跟随；网格仅 pointerup 对齐 |
| 关系工具下从字段拖出 | 橡皮筋连到目标字段（主要创建手势，见 `core-01b`） |
| 关系工具下点击两字段 | 辅助创建手势，与拖出线共用确认条 |
| 鼠标左键双击对象 | 进入编辑模式 |
| 鼠标右键 | 上下文菜单（编辑/删除/复制） |
| 滚轮 | 画布缩放（中心点 = 鼠标位置） |
| 空格 + 拖拽 | 画布平移 |
| Delete / Backspace | 删除选中对象 |
| Ctrl/Cmd + Z | 撤销 |
| Ctrl/Cmd + Shift + Z | 重做 |
| Ctrl/Cmd + D | 复制选中 |

### 3.1 拖动跟线与网格对齐

- **拖动中**：对象使用未量化的视觉坐标；所有连接到该对象的关系路径在同一 `requestAnimationFrame` 回调内重算。禁止只改对象位置而让连线停留在 pointerdown 时的几何。
- **松手对齐**：`pointerup` 时将 `{x, y}` 对齐到网格并写入 undo。主原型网格步长 **12px**；生产端网格步长与现有画布网格一致（`GRID_SIZE`，当前 **20px**）。不得在 pointermove 中量化坐标（避免连线一格一格跳）。
- **DOM/Canvas**：拖动过程禁止重建整页或整块 Canvas 容器；原型只更新表 `left/top` 与 SVG `path[d]`；生产端在 rAF 中调用 `draw_canvas`，拖动中不 `set` 整表数组。
- **捕获**：`setPointerCapture`，避免指针离开命中面后跟丢。

## MODIFIED — 5. 渲染策略（coldrawdb V1） > 5.1 渲染主体

### 5.1 渲染主体

- **render 层**：`frontend-rs/editor_render` 使用 `<canvas>`（HTML5）+ 贝塞尔连线（自渲染，无 vDOM diff）
- **响应式**：基于 Leptos signals 细粒度更新（仅重绘变更部分）
- **拖动绘制**：pointermove 只更新临时变换；`requestAnimationFrame` 合并为每帧至多一次 `draw_canvas`。文件头声明的 rAF 节流必须落地，不得每 mousemove 触发 store 全量 set。
- **性能预算**：100 张表 / 200 条关系 / 60fps（来源：Phase 4 W4 perf）；拖表跟线计入该预算
- **画布容器 testid**：`<div class="cdb-canvas-container">` 必须带 `data-testid="editor-canvas"`，用于 e2e 定位画布区域（HP-01 / HP-05 锚点）
