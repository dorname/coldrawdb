# Delta — core-CR-canvas-test-cases.md（光标锚定缩放 + 渲染性能预算）

> 模块：core | 提案：fix-canvas-zoom-invite-comment-resize（问题 1）
> 目标主文档：`logos/resources/test/core-CR-canvas-test-cases.md`
> 对齐规格：`core-01-editor-canvas.md` §3.3（锚定缩放）+ §5.6（渲染性能预算 R-PERF-01~06）

## ADDED — 8. fix-canvas-zoom-perf：光标锚定缩放 + 渲染性能预算（2026-09-08）

> merge 时在 §7 之后新增章节。本批用例覆盖 §3.3 锚定公式与 §5.6 性能预算；纯数学 / 纯函数用例走 wasm-pack unit test（`editor_render.rs` 内 `#[cfg(test)]` 或 `frontend-rs/tests/wasm/`），e2e 走 `frontend-rs/scripts/`。

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
  - 缩放后光标下对象仍在光标附近（锚定 e2e 佐证，容差 ±2 CSS px）

### ST-CR-INSP-01 — Inspector 输入不触发画布全量重绘（e2e）

- **位置**：`frontend-rs/scripts/test-spec-parity-d.mjs`（或新增脚本）+ `window.__cdb_paint_count`
- **步骤**：选中表字段 → Inspector `inspector-field-tag` 连续键入 10 字符 → blur
- **断言**：
  - 键入期间 `paint_count` 不增长（Inspector 输入不驱动 `draw_canvas`）
  - blur 落账后下一次画布信号（如有）正常重绘，画布对象与保存快照一致（与 ST-CR-TAG-01 不丢字结论共存不冲突）

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
