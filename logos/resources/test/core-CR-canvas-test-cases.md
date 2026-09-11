# 画布渲染测试用例规格

> 模块：core | 提案：add-frontend-completeness
> 路径：`logos/resources/test/core-CR-canvas-test-cases.md`
> 对齐参考源：`core-01-editor-canvas.md` §5.3 + `core-04-side-panel-tabs.md` §8

## 1. 范围

画布：表拖动、pointer capture、关系线跟手。生产松手网格 **`GRID_SIZE=20`**；主原型演示 `GRID=12`，不得把 12 写成生产合同。

状态：后端已实现；生产前端部分接入；逐项对齐待第二阶段。实现阶段须将用例结果写入 `logos/resources/verify/test-results.jsonl`（OpenLogos reporter）；本提案仅规格收口，不执行自动化。

## 2. UT 用例

### UT-CR-01 — Areas 渲染（store.areas → draw_area）

- **位置**：`frontend-rs/src/editor_core.rs`（store 状态切换）+ `frontend-rs/src/editor_render.rs::draw_canvas`（接收 `&[Area]`）
- **前置**：store 初始 areas=Vec::new()，table 列表 1 项
- **步骤**：
  1. 创建 EditorStore
  2. 通过 `load(diagram_with_2_areas)` 注入 2 个 area
  3. 验证 `store.areas.get().len() == 2`
  4. 调用 `snapshot` 验证 Diagram 序列化含 areas
- **断言**：
  - `store.areas.get().len() == 2`
  - `snapshot.areas` 含 2 项（id/name 一致）
  - 初始 `EditorStore::new()` 时 `store.areas.get().is_empty()`

### UT-CR-02 — Notes 渲染（store.notes → draw_note）

- **位置**：`frontend-rs/src/editor_core.rs`（store.notes）
- **前置**：EditorStore::new() 时 notes 为空
- **步骤**：
  1. 创建 EditorStore
  2. `load(diagram_with_3_notes)` 注入 3 个 note
  3. 验证 `store.notes.get().len() == 3`
- **断言**：
  - `store.notes.get().len() == 3`
  - `snapshot.notes` 含 3 项

### UT-CR-03 — 端点 drag 改 start_field_id

- **位置**：`frontend-rs/src/editor_render.rs`（新增纯函数 `update_reference_endpoint`）
- **前置**：store 含 1 张表 (id="t1", fields=[f1, f2]) + 1 条 reference (start_field_id="f1", end_field_id="f2")
- **步骤**：
  1. 调用 `update_reference_endpoint(refs, "r1", EndpointEnd::Start, "f2")`
  2. 验证返回值
- **断言**：
  - 返回 Vec<Reference> 长度 == 1
  - `result[0].start_field_id == "f2"`（已更新）
  - `result[0].end_field_id == "f2"`（未变）
  - 原始 Vec 未被修改（pure function）

### UT-CR-04 — 端点 drag 改 end_field_id

- **位置**：同 UT-CR-03
- **步骤**：调用 `update_reference_endpoint(refs, "r1", EndpointEnd::End, "f3")`
- **断言**：
  - `result[0].end_field_id == "f3"`
  - `result[0].start_field_id == "f1"`（未变）

### UT-CR-05 — 端点 drag 不存在的 reference_id

- **位置**：同 UT-CR-03
- **步骤**：调用 `update_reference_endpoint(refs, "nonexistent", EndpointEnd::Start, "f2")`
- **断言**：
  - 返回原 Vec（未修改）
  - 调用方应检查返回 Vec 与原 Vec 一致

### UT-CR-06 — 网格对齐仅在松手

| ID | 变更 | 合同 |
|---|---|---|
| UT-CR-06 | MODIFIED | 生产 `snap_to_grid(..., 20.0)`；拖动中不量化 |
| UT-CR-07 / ST-CR-02 | MODIFIED | pointermove 期间关系 path 使用当前视觉坐标；跟手非松手跳变 |
| UT-CR-PC-01（ADDED） | ADDED | 表头拖动 `setPointerCapture`；指针移出命中面不丢拖 |
| UT-CR-PC-02（ADDED） | ADDED | rAF 合并重绘；禁止每 move 整页重建 `#app` |
| ST-CR-GRID-20（ADDED） | ADDED | 生产 e2e：松手后 `x/y` 为 20 的倍数 |
| ST-CR-GRID-PROTO（ADDED） | ADDED | 主原型 PU：松手后为 12 的倍数（仅原型回归） |

### UT-CR-07 — 连线使用表的当前视觉坐标

- **位置**：`frontend-rs/src/editor_render.rs::calc_path` / `draw_canvas`
- **前置**：表 A (100,100) 与表 B (400,100) 之间已有 reference
- **步骤**：将表 A 的绘制坐标改为 (160, 140)（模拟拖动中临时坐标），调用路径计算
- **断言**：起点 x/y 随 160/140 变化，不得仍使用 100/100

## 3. ST 用例

### ST-CR-01 — references 贝塞尔连线在画布可见（e2e，wasm-pack test）

- **位置**：`frontend-rs/tests/wasm/cr.rs`
- **类型**：wasm-pack test --headless --chrome
- **步骤**：
  1. 启动后端 + 前端
  2. 通过 UI 创建 2 张表 + 1 条 reference
  3. 等待 Canvas 渲染（100ms）
  4. 截图后采样参考线区域
- **断言**：
  - Canvas 像素采样：起点/终点圆点 + 中段贝塞尔曲线存在
  - 注：完整像素断言需 image-diff 工具，B3 用 DOM 检查（`data-testid="editor-canvas"` 存在）+ 画布尺寸 + 端点坐标计算正确
- **B3 标记 skip**：完整 e2e 跑在 B5 wasm-pack test 接入后；B3 实现保证 draw_canvas 函数逻辑正确

### ST-CR-02 — 拖表过程中连线路径更新（e2e）

- **位置**：`frontend-rs/tests/e2e/11_canvas_interaction.spec.ts`（或新 `18_canvas_drag_follow.spec.ts`）
- **步骤**：
  1. 画布上已有两表一条关系
  2. 标题栏 pointerdown 后 pointermove 至少 40px，**在 pointerup 之前**采样连线几何（Canvas 像素或暴露的 debug 路径）
  3. pointerup
- **断言**：
  - 移动过程中路径已偏离 pointerdown 时的几何（跟手，非松手才跳变）
  - 松手后表坐标为 `GRID_SIZE` 的倍数

## 与主原型对齐说明

- 拖表过程中已有关系 SVG/`path[d]` 必须连续更新（对齐 ST-PU-05/21）。
- 生产验收以 `GRID_SIZE=20` + pointer capture + 跟线为准；视觉像素级对齐待第二阶段。

## 4. V1 边界

- ❌ Areas/Notes 的拖拽创建（B3 仅 render + store 接入，创建按钮放 B4 模态）
- ❌ 端点 drag 的实时 visual feedback（B3 完成后端点位置更新，前端视觉反馈放 B5）
- ❌ Issues Tab 跳转的画布闪烁（pan to target）— B3 实现 selected_id 切换，闪烁效果放 B5
- ❌ Areas/Notes 端点的右键菜单 — B3 范围外

## 5. 对齐参考源

- `core-01-editor-canvas.md` §5.3（Areas/Notes/References 渲染）
- `core-04-side-panel-tabs.md` §8（Issues Tab 跳转需求）
- `frontend-rs/src/editor_render.rs::draw_canvas`
- `frontend-rs/src/editor_core.rs::EditorStore`
- `frontend-rs/src/editor_panels.rs::IssuesTab`

## 附录 A：用例 ID 清单（OpenLogos verify 解析用）

| ID | 标题 | 对齐实现 |
|---|---|---|
| UT-CR-01 | Areas 渲染（store 状态切换） | `editor_core.rs::EditorStore` |
| UT-CR-02 | Notes 渲染（store 状态切换） | `editor_core.rs::EditorStore` |
| UT-CR-03 | 端点 drag 改 start_field_id | `editor_render.rs::update_reference_endpoint` |
| UT-CR-04 | 端点 drag 改 end_field_id | `editor_render.rs::update_reference_endpoint` |
| UT-CR-05 | 端点 drag 不存在的 reference_id | `editor_render.rs::update_reference_endpoint` |
| UT-CR-06 | 网格对齐仅在松手 | `editor_render.rs::snap_to_grid` |
| UT-CR-07 | 连线使用当前视觉坐标 | `editor_render.rs::calc_path` |
| ST-CR-01 | references 贝塞尔连线在画布可见 | `frontend-rs/tests/wasm/cr.rs` |
| ST-CR-02 | 拖表过程中连线路径更新 | e2e canvas drag follow

## 6. p0-fix 定点 2：区域 / 便签创建交互（2026-09-04）

> 「V1 边界」中「Areas/Notes 拖拽创建放 B4 模态」由 p0-fix 定点 2 提前落地（工具按钮直接创建，无模态）。

### UT-AREA-01 — Area 拖拽创建（拖框 < 10px 不创建）

- **位置**：`frontend-rs/src/editor_render.rs`（`area_rect_from_drag` / `build_area` / `hit_test_area`）
- **步骤**：
  1. `area_rect_from_drag(100,100,260,220)` → `Some((100,100,160,120))`
  2. 反向拖框 `(260,220)→(100,100)` → 归一化为同一矩形
  3. 拖框 5×8 / 0×0 / 150×5 → `None`（<10px 防误触）
  4. `build_area` 默认 `name="未命名区域"`、`color="#3b82f6"`
  5. `hit_test_area` 矩形内命中、外部不命中、重叠时后创建优先

### UT-NOTE-01 — Note 点击放置（固定 180×100 命中）

- **位置**：`frontend-rs/src/editor_render.rs`（`build_note` / `hit_test_note`，`NOTE_WIDTH=180` / `NOTE_HEIGHT=100`）
- **步骤**：
  1. `build_note` 默认 `content=""`、`color="#f59e0b"`
  2. `hit_test_note` 渲染矩形中心命中；右/下边界外 1px 不命中

### ST-AN-01 — 拖框创建区域 → Inspector 编辑 → Delete 键删除（e2e）

- **位置**：`frontend-rs/scripts/test-spec-parity-d.mjs`
- **步骤**：`tool-new-area` → 画布拖框 300×200 → `inspector-area-form` 可见 → PUT `areas.len()==1`（默认名/宽度正确）→ `inspector-area-name` 改名落账 → Delete 键 → PUT `areas.len()==0`

### ST-AN-02 — 点击放置便签 → Inspector 编辑内容 → 按钮删除（e2e）

- **位置**：`frontend-rs/scripts/test-spec-parity-d.mjs`
- **步骤**：`tool-new-note` → 点击画布 → `inspector-note-form` 可见 → PUT `notes.len()==1` → `inspector-note-content` 编辑落账 → `btn-delete-note` → PUT `notes.len()==0`

### 附录 A 追加

| ID | 标题 | 对齐实现 |
|---|---|---|
| UT-AREA-01 | Area 拖框创建（<10px 不创建） | `editor_render.rs::area_rect_from_drag` |
| UT-NOTE-01 | Note 点击放置 | `editor_render.rs::build_note` |
| ST-AN-01 | 区域创建/编辑/Delete 删除 | e2e test-spec-parity-d |
| ST-AN-02 | 便签创建/编辑/按钮删除 | e2e test-spec-parity-d |

## 7. redesign-listview-type-length-canvas-fix：便签/区域拖动 + tag 受控输入（2026-09-07）

> 修复三个交互 bug：note/area 拖拽状态机缺分支（命中只选中）、tag 输入静态快照 + blur 重建丢字、高频 signal set × 同步全量重绘卡死。

### ST-CR-NOTE-01 — 便签拖动落账 + 编辑不卡死（e2e）

- **位置**：`frontend-rs/scripts/test-spec-parity-d.mjs`
- **步骤**：`tool-new-note` → 点击画布放置 → mousedown 便签中心 → 拖动 ≥60px → mouseup → Inspector 编辑内容连续键入一段文本
- **预期**：拖动中便签跟随（视觉层）；mouseup 后 PUT 落账 `notes[0].x/y` 更新为拖后坐标；连续键入不丢字（`inspector-note-content` 最终值与键入一致）；全程无长时间帧阻塞（拖动 mousemove 不触发 PUT）

### ST-CR-AREA-01 — 区域拖动落账（e2e）

- **位置**：`frontend-rs/scripts/test-spec-parity-d.mjs`
- **步骤**：`tool-new-area` → 拖框创建区域 → mousedown 区域内部 → 拖动 ≥60px → mouseup
- **预期**：mouseup 后 PUT 落账 `areas[0].x/y` 更新为拖后坐标（`on_update_area` x/y 写回通道）；宽高不变；Inspector 区域表单仍选中

### ST-CR-TAG-01 — Inspector 字段 tag 受控输入连续键入不丢字（e2e）

- **位置**：`frontend-rs/scripts/test-spec-parity-d.mjs`
- **步骤**：选中表字段 → Inspector `inspector-field-tag` 连续键入 `finance-core`（逐字 input 事件）→ blur
- **预期**：输入过程中值不被重建打断（每击键后输入框值单调增长）；blur 落账 PUT 快照 `fields[i].tag=="finance-core"`；再次聚焦时值保持

### 附录 A 追加

| ID | 标题 | 对齐实现 |
|---|---|---|
| ST-CR-NOTE-01 | 便签拖动落账 + 编辑不丢字 | e2e test-spec-parity-d |
| ST-CR-AREA-01 | 区域拖动落账（on_update_area x/y 通道） | e2e test-spec-parity-d |
| ST-CR-TAG-01 | Inspector tag 受控输入不丢字 | e2e test-spec-parity-d |

## 8. fix-canvas-zoom-perf：光标锚定缩放 + 渲染性能预算（2026-09-08）

> 覆盖 `core-01-editor-canvas.md` §3.3 锚定公式与 §5.6 性能预算（R-PERF-01~06）。纯数学 / 纯函数用例走 wasm-pack unit test（`editor_render.rs` 内 `#[cfg(test)]` 或 `frontend-rs/tests/wasm/`），e2e 走 `frontend-rs/scripts/`。

### UT-CR-ZOOM-01 — 滚轮缩放光标锚定（纯坐标数学）

- **位置**：`frontend-rs/src/editor_render.rs`（新增纯函数，如 `zoom_transform_at_anchor(transform, anchor_css, factor, rect_origin) -> Transform`）
- **前置**：`transform = { pan_x: 100.0, pan_y: 60.0, zoom: 1.0 }`；canvas rect 原点 `(rect.left, rect.top) = (40, 32)`；光标 client 坐标 `(440, 332)`（即 canvas CSS 坐标 `anchor = (400, 300)`）
- **步骤**：
  1. 计算缩放前光标下世界点 `world = screen_to_diagram(...)`
  2. 以 `factor = 1.1` 调用锚定缩放纯函数得 `t1`
  3. 断言锚定不变量：`world × t1.zoom + t1.pan == anchor`（光标下世界点屏幕位置不变）
  4. 再滚一次（`factor = 1.1`），重复断言不变量
- **断言**：
  - 两次缩放后不变量均成立（误差 < 1e-9）
  - `t1.zoom == 1.1`，`t1.pan_x == anchor.x − world.x × 1.1`
- **clamp 子用例**：`zoom = 5.0, factor = 1.1` → `new_zoom == 5.0`（ZOOM_MAX = 5.0，对齐现行实现与修订后 §4），且 `pan'` 按 clamp 后 zoom 计算、不变量仍成立（锚定优先，不退化为画布中心）

### UT-CR-CULL-01 — 视口 AABB 裁剪（视口外对象零绘制调用）

- **位置**：`frontend-rs/src/editor_render.rs`（`aabb_intersects` / `visible_in_viewport` 纯函数；`draw_canvas` 裁剪分支）
- **前置**：视口 AABB `(0,0,1920,1080)`（世界坐标）；表 A `(100,100)` 在视口内，表 B `(-5000,-5000)` 完全在视口外
- **步骤**：
  1. `aabb_intersects`：相交 / 包含 / 相离 / 边界贴合一组 4 case
  2. 以计数包装（或抽出 `collect_visible` 纯函数）验证：仅视口内对象进入绘制列表
- **断言**：
  - 相离返回 false；相交/包含/贴合返回 true
  - 视口外表 / 区域 / 便签 / 关系不发起 `draw_table` / `draw_area` / `draw_note` / `draw_bezier_fields` 调用（spy 计数为 0）

### UT-CR-BATCH-01 — 网格点阵合批（绘制调用 O(1)）

- **位置**：`frontend-rs/src/editor_render.rs::draw_grid`
- **前置**：任意 transform 与 canvas 尺寸（点数 > 1000）
- **步骤**：以 mock / spy context 记录 `begin_path` 与 `fill` 调用次数
- **断言**：
  - `begin_path` 次数 ≤ 1（单 path 合并全部点）
  - `fill` 次数 = 1，且与点数无关（改变 GRID 密度点数翻倍，调用数不变）
  - 若采用 CSS 背景替代方案：canvas 层无点阵绘制调用（`draw_grid` 移除或为空实现），本用例断言等价物

### UT-CR-FONTCACHE-01 — 字体家族探测缓存（check 调用有界）

- **位置**：`frontend-rs/src/editor_render.rs::resolve_canvas_font_family`
- **前置**：mock `document.fonts().check` 计数器
- **步骤**：
  1. 连续 2 次调用 `resolve_canvas_font_family(CANVAS_FONT, CANVAS_FONT_MONO)`
  2. 模拟同帧内多次 `draw_table`（每个表触发一次字体解析）
- **断言**：
  - 第 2 次解析不再调用 `fonts().check`（缓存命中，计数 = 1）
  - 模拟 20 张表一帧渲染后，check 总次数 ≤ 1（每族每进程至多 1 次直至 `loadingdone` 失效）

### UT-CR-DRAG-01 — 拖动覆盖层不克隆整表 Vec

- **位置**：`frontend-rs/src/editor_render.rs`（`LivePaint` 拖动路径 + `apply_visual_table_position` 调用点）
- **前置**：store 含 100 张表；Clone 计数包装 `Table` 或架构计数 spy
- **步骤**：
  1. 模拟表头 `pointermove` × 20 次（拖动中）
  2. 记录 `Table::clone`（或 `Vec<Table>` 重建）次数
  3. `pointerup` 落账
- **断言**：
  - pointermove 阶段 `Table` clone 次数为 0（`LivePaint` 仅 `(id, x, y)` 覆盖记录）
  - `apply_visual_table_position`（整 Vec 克隆）仅在 `pointerup` 落账路径被调用 1 次
  - 拖动中 `draw_canvas` 使用覆盖坐标绘制被拖表（坐标断言：绘制输入含 `(id, new_x, new_y)`）

### ST-CR-PAN-01 — pan 走 rAF 合并（e2e，可选 debug 计数）

- **位置**：`frontend-rs/scripts/`（复用 test-spec-parity 风格脚本）+ `window.__cdb_paint_count` debug 计数
- **步骤**：滚轮缩放 / 空格拖拽平移连续触发 ≥ 10 次事件，统计 `draw_canvas` 实际执行次数
- **断言**：
  - 同一帧内多次事件至多 1 次重绘（paint_count 增长 ≤ 事件帧数）
  - **perf-canvas-drag-smoothness 口径修订**：平移断言放宽为 `≤ 事件数 + 1`——R-PERF-10 拖拽降采样在松手帧恢复全分辨率 backing store，产生一次有界的收尾重绘；rAF 合并不变量（每帧至多 1 次重绘、与事件数线性有界）不变
  - 缩放后光标下对象仍在光标附近（锚定 e2e 佐证，容差 ±2 CSS px）

### ST-CR-INSP-01 — Inspector 输入不触发画布全量重绘（e2e）

- **位置**：`frontend-rs/scripts/test-spec-parity-d.mjs`（或新增脚本）+ `window.__cdb_paint_count`
- **步骤**：选中表字段 → Inspector `inspector-field-tag` 连续键入 10 字符 → blur
- **断言**：
  - 键入期间 `paint_count` 不增长（Inspector 输入不驱动 `draw_canvas`）
  - blur 落账后下一次画布信号（如有）正常重绘，画布对象与保存快照一致（与 ST-CR-TAG-01 不丢字结论共存不冲突）

### UT-CR-SPRITE-01 — 表精灵指纹与 zoom 分档（R-PERF-07）

- **位置**：`frontend-rs/src/editor_render.rs`（`sprite_zoom_bucket` / `table_sprite_fingerprint` 纯函数）
- **步骤**：构造同一 `Table` 的多份变体，分别计算指纹
- **断言**：
  - 分档边界：zoom 0.5 / 1.0 → 档 1；1.01 / 2.0 → 档 2
  - 坐标稳定性：仅改 `x` / `y` 指纹不变（拖动不触发重光栅）
  - 失效面：改名 / 改字段类型 / 改主键 / 改色 / 改宽 / 主题 / dpr×100 / 分档 任一变化指纹必变

### UT-CR-GUARD-01 — DOM 写守护（R-PERF-08）

- **位置**：`frontend-rs/src/editor_render.rs`（`dom_write_guard` 纯函数）
- **断言**：空→空 不写；值变化写并更新 last；同值连帧不写；新值再写；清空也算变更

### UT-CR-DPR-01 — 拖拽降采样 dpr（R-PERF-10）

- **位置**：`frontend-rs/src/editor_render.rs`（`capped_drag_dpr` 纯函数）
- **断言**：
  - 拖拽活跃：dpr 2.0 / 3.0 / 1.5 均压到 `DRAG_RENDER_DPR_CAP = 1.0`
  - 低 dpr 设备（0.8）不再降（min 语义，不抬升）
  - 非拖拽：原样透传（2.0 → 2.0，1.0 → 1.0）

### UT-CR-GHOST-01 — 幽灵层准入判定与 CSS 定位（R-PERF-11）

- **位置**：`frontend-rs/src/editor_render.rs`（`table_has_references` / `ghost_css_position` 纯函数）
- **断言**：
  - refs 起点表 / 终点表判定为有关联（禁走幽灵层）；无关联表与空 refs 可走幽灵层
  - 定位换算 `(x - margin) × zoom + pan_x` / `(y - margin) × zoom + pan_y`，原点零值用例

### ST-CR-GHOST-01 — 无关系线表拖拽期间主画布零重绘（e2e）

- **位置**：`frontend-rs/scripts/test-canvas-perf.mjs` + `window.__cdb_paint_count` debug 计数
- **步骤**：建 6 张无关系线表 → 拖动首表（≥20 次 pointermove）→ 松手
- **断言**：
  - 拖动中出现 `data-testid="drag-ghost"` 幽灵层节点，松手后被移除
  - 拖动 move 阶段 `__cdb_paint_count` 增长 ≤ 2（仅幽灵层创建当帧的「抠出」重绘；零重绘路径不再逐帧 paint）
  - 松手落账后画布恢复正常重绘（paint_count 恢复增长）且表位置与拖动终点一致

### 附录 A 追加

| ID | 标题 | 对齐实现 |
|---|---|---|
| UT-CR-ZOOM-01 | 滚轮缩放光标锚定（含 clamp 锚定） | `editor_render.rs` 锚定缩放纯函数（新增） |
| UT-CR-CULL-01 | 视口 AABB 裁剪（视口外零绘制） | `editor_render.rs::aabb_intersects` / `draw_canvas` |
| UT-CR-BATCH-01 | 网格点阵合批 O(1) | `editor_render.rs::draw_grid` |
| UT-CR-FONTCACHE-01 | 字体家族 check 缓存有界 | `editor_render.rs::resolve_canvas_font_family` |
| UT-CR-DRAG-01 | 拖动 mousemove 不克隆整表 Vec | `editor_render.rs::LivePaint` / `apply_visual_table_position` |
| ST-CR-PAN-01 | pan/缩放 rAF 合并重绘 | e2e + `__cdb_paint_count` |
| ST-CR-INSP-01 | Inspector 输入不触发画布重绘 | e2e test-spec-parity 脚本 |
| UT-CR-SPRITE-01 | 表精灵指纹与 zoom 分档 | `editor_render.rs::sprite_zoom_bucket` / `table_sprite_fingerprint` |
| UT-CR-GUARD-01 | DOM 写守护 | `editor_render.rs::dom_write_guard` |
| UT-CR-DPR-01 | 拖拽降采样 dpr | `editor_render.rs::capped_drag_dpr` |
| UT-CR-GHOST-01 | 幽灵层准入判定与 CSS 定位 | `editor_render.rs::table_has_references` / `ghost_css_position` |
| ST-CR-GHOST-01 | 无关系线表拖拽主画布零重绘 | e2e test-canvas-perf + `__cdb_paint_count` + `drag-ghost` |
