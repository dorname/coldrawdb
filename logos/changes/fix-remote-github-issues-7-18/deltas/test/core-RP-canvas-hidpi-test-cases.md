# Delta — core-RP-canvas-hidpi-test-cases.md（修改）

> 模块：core | 提案：fix-remote-github-issues-7-18
> 关联 issue：#11（尺寸变化失真） | 规格：`core-01-editor-canvas.md` §11.5（R-DPR-07～10）

## ADDED — UT-RP-06 — 容器尺寸变化经 ResizeObserver 同步 backing store

- **GIVEN**：canvas 父容器初始 1200×800 @ dpr=1
- **WHEN**：容器宽度变为 900（模拟分隔条拖动），触发 ResizeObserver 回调
- **THEN**：同帧（rAF 内）`canvas.width == 900`、`canvas.height == 800`；`set_transform` 按最新 dpr×zoom 复位；无需任何画布交互事件介入
- **位置**：`frontend-rs/src/editor_render.rs`（`setup_canvas_resize_observer` 或等价实现）

## ADDED — UT-RP-07 — 分隔条拖动触发重绘且不绕过统一 dpr 来源

- **GIVEN**：Inspector 分隔条拖动中（`splitter.rs` pointermove）
- **WHEN**：连续 5 次宽度变更
- **THEN**：每次都经 ResizeObserver（或 splitter 显式 `schedule_paint` 兜底）触发重绘；重绘使用的有效 dpr 来自 `effective_device_pixel_ratio()`（与 R-PERF-10 同源）；卸载后 observer 已 disconnect

## ADDED — ST-RP-04 — e2e：拖分隔条全程文字清晰

- **GIVEN**：编辑器含多表，缩放 64%（非整数），dpr=1 与 dpr=2 两组
- **WHEN**：拖动 Inspector 分隔条往返 ≥10 次 → 松手；随后窗口尺寸变更
- **THEN**：过程中与结束后画布无位图拉伸模糊（抽查：画布区域截图与稳定态基线像素相似度 ≥ 95%，或 `canvas.width == CSS宽×dpr` 断言在每步成立）；全程未点击画布

> §3 验收条件追加：AC-RP-06 — ResizeObserver 同步路径落地且 UT-RP-06/07、ST-RP-04 PASS。
> 结果写入 `logos/resources/verify/test-results.jsonl`（`module: "core"`，`scenario: "S01"`）。
