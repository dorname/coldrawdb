# Delta — core-01-editor-canvas.md（画布性能 + 光标锚定缩放）

> 模块：core | 提案：fix-canvas-zoom-invite-comment-resize（问题 1）
> 目标主文档：`logos/resources/prd/2-product-design/1-feature-specs/core-01-editor-canvas.md`

## MODIFIED — 3. 交互手势

> merge 时整体替换主文档「## 3. 交互手势」章节。变更点：（1）滚轮手势行由「中心点 = 鼠标位置」升级为可验收的锚定公式；（2）新增 §3.3 锚定数学与 clamp 语义。

## 3. 交互手势

| 手势 | 效果 |
|---|---|
| 鼠标左键单击 | 选中对象（高亮） |
| 鼠标左键拖拽空白 | 框选多个对象 |
| 鼠标左键拖拽对象 | 移动对象；连线每帧跟随；网格仅 pointerup 对齐 |
| 关系工具下从字段拖出 | 橡皮筋连到目标字段（主要创建手势，见 `core-01b`） |
| 关系工具下点击两字段 | 辅助创建手势，与拖出线共用确认条 |
| 鼠标左键双击对象 | 进入编辑模式 |
| 鼠标右键 | 上下文菜单（编辑/删除/复制） |
| 滚轮 | 画布缩放（**光标锚定**：缩放前后光标下的世界点在屏幕上位置不变，见 §3.3） |
| 空格 + 拖拽 | 画布平移 |
| Delete / Backspace | 删除选中对象 |
| Ctrl/Cmd + Z | 撤销 |
| Ctrl/Cmd + Shift + Z | 重做 |
| Ctrl/Cmd + D | 复制选中 |

### 3.1 拖动跟线与网格对齐

- **指针捕获**：表头拖动 `setPointerCapture`；指针离开命中面不得丢跟。
- **rAF 合并**：`pointermove` 只更新临时坐标；同一 `requestAnimationFrame`（原型 `schedulePaint` / 生产 `schedule_paint` → `draw_canvas`）内重算表位置与关联关系路径。禁止每 move 整页 `render()` 或 `store.tables.set(整表)`。
- **松手网格**：`pointerup` 才将 `{x, y}` 量化。生产端 **`GRID_SIZE = 20`**；主原型演示网格为 12px（`GRID`），不得把原型步长写成生产合同。
- **拖动中禁止**在 pointermove 中量化（避免连线一格一格跳）。

### 3.2 协作画布叠加层（与主原型一致）

| 元素 | `data-testid` | 行为 |
|---|---|---|
| 远端光标 | `remote-cursor` | 显示协作者指针与标签；pointer-events: none |
| 连接 Banner | `reconnect-banner` | `connection ≠ connected` 时出现；重连中 / 同步中 / 失败（含仅本地）文案与操作 |
| 关系橡皮筋 | `rel-rubber-band` | 见 `core-01b`；拖动中仅更新 `path[d]` |
| 关系提示条 | `rel-tool-hint` | 关系工具激活时显示 |

演示控制台触发的远端事件**不得**写成生产必选 UI；生产以 WS presence / OT 事件驱动同等反馈。

### 3.3 滚轮缩放光标锚定（V1.1）

- **锚定不变量**：滚轮缩放前后，**光标下的世界坐标点映射到屏幕上的位置不变**。缩放不得使光标下的对象"漂走"。
- **计算公式**（纯函数，可单测）：

  ```
  world   = screen_to_diagram(mouse_client, canvas_rect, transform)   // 缩放前光标下的世界坐标
  anchor  = mouse_client − (rect.left, rect.top)                      // 光标在 canvas CSS 坐标系内的位置
  new_zoom = clamp(zoom × factor, ZOOM_MIN, ZOOM_MAX)
  pan'    = anchor − world × new_zoom
  zoom'   = new_zoom
  ```

  其中 `factor = 1.1`（滚上）/ `1/1.1`（滚下）；`ZOOM_MIN = 0.1`、`ZOOM_MAX = 5.0`（与现行实现 `on_wheel` / `zoom_in` / `zoom_out` 的 clamp 一致；主文档 §4 的 `0.25x ~ 4x` 为过期口径，本提案一并修订，见下方「MODIFIED — 4. 坐标系」）。
- **clamp 保持锚定**：当 `zoom × factor` 触及边界被 clamp 时，`pan'` 仍按上述公式以 clamp 后的 `new_zoom` 计算，锚定优先；禁止 clamp 时退化为画布中心缩放。
- **rect 基准**：`anchor` 必须减去 `get_bounding_client_rect()` 的 `left/top`（canvas 在视口内的偏移），否则缩放会累积偏移导致画布"漂"出视口（现行 `on_wheel` 已按此实现，固化为合同）。
- **工具栏按钮缩放**（`zoom_in` / `zoom_out` / `zoom_reset`）：以**视口中心**为锚点，公式同上，`anchor = (canvas_css_width / 2, canvas_css_height / 2)`。
- 缩放触发的重绘走 `schedule_paint`（rAF 合并，见 §5.6 R-PERF-05），禁止在 wheel 事件里同步全量重绘。

## MODIFIED — 4. 坐标系

> merge 时整体替换主文档「## 4. 坐标系」章节。变更点：缩放范围由过期的 `0.25x ~ 4x` 修订为与现行实现一致的 `0.1x ~ 5.0x`（用户确认保留代码口径）；缩放中心补充光标锚定合同引用。

## 4. 坐标系

- 坐标系：屏幕坐标系（x 向右、y 向下）
- 单位：像素
- 缩放范围：0.1x ~ 5.0x（V1.1 修订，对齐现行实现 clamp；含滚轮与工具栏 zoom_in / zoom_out）
- 缩放中心：滚轮缩放以**光标位置**为锚点（锚定不变量见 §3.3）；工具栏按钮缩放以视口中心为锚点
- 持久化：`{x, y}` 与对象状态一同存入 `core_diagram` JSON 字段

## ADDED — 5.6 渲染性能预算（V1.1）

> merge 时在 §5.5 之后新增小节。本小节把现行 `frontend-rs/src/editor_render.rs` 的渲染路径升级为可验收的性能合同；各项均为 V1.1 必交付。

### 5.6 渲染性能预算（V1.1）

**问题陈述**（现行实现缺口，源于 `frontend-rs/src/editor_render.rs` 现状核查）：

| 项 | 现行（V1 launched） | 期望（V1.1） |
|---|---|---|
| 视口裁剪 | `draw_canvas` 无条件遍历全部表/区域/便签/关系 | 仅绘制与视口 AABB 相交的对象 |
| 网格点阵 | `draw_grid` 逐点 `begin_path + arc + fill`（O(点数) 次绘制调用） | 合并为单个 path 一次 `fill`（或 CSS 背景替代），O(1) 次绘制调用 |
| 字体家族探测 | `resolve_canvas_font_family` 每次调用执行 `document.fonts().check()`，且 `draw_table` 等每对象重复调用 | 探测结果进程级缓存；同一帧每族至多 1 次 DOM 探测 |
| 拖动覆盖层 | `pointermove` 经 `apply_visual_table_position` 深克隆整张 `Vec<Table>` 存入 `LivePaint` | 拖动中仅记录被拖对象 `(id, x, y)`，绘制时合并 |
| pan 重绘 | wheel/pan 路径未见 rAF 合并入口 | pan 与表拖动共用 `schedule_paint`（`schedule_render_dedup`），一帧至多一次 `draw_canvas` |
| refs 端点查找 | `draw_canvas` 内对每个 reference 在 `tables` 上线性 `find`（O(refs × tables)） | `id → &Table` 的 `HashMap` 预建，O(refs + tables) |

**实现约束**：

| ID | 约束 |
|---|---|
| R-PERF-01 | `draw_canvas` 仅绘制与视口 AABB 相交的表 / 区域 / 便签 / 关系。裁剪判定抽为纯函数（如 `aabb_intersects(obj_aabb, viewport_aabb)`），视口 AABB 由 `transform` + canvas CSS 尺寸推导，可单测 |
| R-PERF-02 | 网格点阵合并为**单个 path 一次 `fill`**（所有点 `arc` 进同一 path），绘制调用数为 O(1)；允许以 CSS `radial-gradient` 背景替代 canvas 点阵，二者取一 |
| R-PERF-03 | `resolve_canvas_font_family` 的 `document.fonts().check()` 结果按 `(primary, mono)` 键进程级缓存（`OnceCell` / `Rc<RefCell<HashMap>>`）；`document.fonts` `loadingdone` 事件触发缓存失效重探。同一帧内同一字体家族至多 1 次 DOM 探测，禁止每对象每帧重复探测 |
| R-PERF-04 | 拖动覆盖层（`LivePaint.tables`）只存被拖对象的 `(id, x, y)`；`draw_canvas` 绘制该表时以覆盖坐标替换。`apply_visual_table_position`（整 Vec 克隆）仅限松手落账路径调用；`pointermove` 禁止整 Vec 深克隆 |
| R-PERF-05 | wheel/pan 与表拖动共用 `schedule_paint`（基于 `schedule_render_dedup` 的 rAF 合并），一帧至多一次 `draw_canvas`；渲染 effect 读取信号一律用 `.with()` / `.get_untracked()`，禁止 `.get()` 深克隆整表 `Vec` 进闭包 |
| R-PERF-06 | `draw_canvas` 内先以 `tables` 构建 `HashMap<&str, &Table>`，references 端点查找 O(1)；总体绘制前准备为 O(refs + tables) |

**验收口径**：

- 100 张表 @ 1080p（Chrome，dpr=1）：拖表与平移保持交互可用——单帧耗时稳定在 16ms 量级以内（主观流畅；抽查手段为 `performance.measure` 包住 `draw_canvas` 或 debug 计数器，不强制引入自动化帧率断言）。
- Inspector 字段输入（如 `inspector-field-tag` 连续键入）**不得**触发全画布重绘：重绘仅由画布自身信号（transform / 拖动覆盖层 / 选中态）驱动；可通过 `window.__cdb_paint_count` 之类的 debug 计数在 e2e 中断言（见 `core-CR-canvas-test-cases.md` ST-CR-INSP-01）。
- HiDPI（§11 R-DPR）与性能预算不冲突：dpr 只放大 backing store 像素，不改变裁剪 / 合批 / 缓存策略。
