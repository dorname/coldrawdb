# Delta — core-CR-canvas-test-cases.md（fix-enter-diagram-fit-content）

> 模块：core | 画布交互 | 关联：进入已有图视口不定位到内容（空画布反馈）

## ADDED — UT-CR-FOCUS-03 — fit_content_transform 内容适应纯函数

### UT-CR-FOCUS-03 — `fit_content_transform` / `content_bounds`

- **位置**：`frontend-rs/src/editor_render.rs` 纯函数 + 内嵌 tests
- **步骤**：
  1. 远处内容（表位于 (4000, 3000) 附近）→ 返回 Some(transform)：内容包围盒整体落入视口并居中（留 padding），zoom clamp [ZOOM_MIN, 1.0]
  2. 小图内容在原点附近且视口足够 → zoom 不放大超过 1.0（100% 上限），内容居中
  3. 空内容（无表/便签/区域）→ 返回 None（调用方保持现有 transform 不动）
- **断言**：zoom clamp 边界（≤1.0、≥ZOOM_MIN）；bounds 计算含表渲染尺寸（`compute_table_render_size`）、便签与区域；空 bounds 不产生 NaN/除零

## ADDED — UT-CR-FOCUS-04 — 加载挂点自动 fit 与适应画布按钮锚点

### UT-CR-FOCUS-04 — 三个加载挂点 + btn-fit-canvas

- **位置**：`frontend-rs/src/editor_panels.rs`（`include_str!` 锚点口径）
- **断言**：
  1. `on_enter_room` / `on_open_personal_diagram` / share 冷启动三处 `store.load` 成功分支均触发自动 fit（调用 fit 辅助或等价符号）
  2. FloatingControls 存在 `data-testid="btn-fit-canvas"`（文案「适应画布」，对齐主原型 fit-canvas）
  3. fit 辅助函数经双 rAF 延迟读取容器尺寸后调用 `resolve_canvas_viewport` 与 `request_canvas_repaint`

## MODIFIED — 附录用例 ID 清单（追加）

| ID | 标题 |
|---|---|
| UT-CR-FOCUS-03 | fit_content_transform 内容适应（远处/近处/空三态 + clamp） |
| UT-CR-FOCUS-04 | 三加载挂点自动 fit + btn-fit-canvas 锚点 |
