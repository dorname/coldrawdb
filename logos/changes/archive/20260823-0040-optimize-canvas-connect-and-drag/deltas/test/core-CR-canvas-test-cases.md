# Delta — core-CR-canvas-test-cases.md

> 模块：core | 提案：optimize-canvas-connect-and-drag

## ADDED — 2. UT 用例 > UT-CR-06 — 网格对齐仅在松手

- **位置**：`frontend-rs/src/editor_render.rs`（纯函数 `snap_to_grid(x, y, grid)`）
- **前置**：`grid = 20.0`（生产）；原型对照 `grid = 12`
- **步骤**：
  1. 拖动中视觉坐标保持 `(133.4, 87.1)` 不调用 snap
  2. pointerup 调用 `snap_to_grid(133.4, 87.1, 20.0)`
- **断言**：
  - 拖动中函数未被用于量化（由集成约定：move 路径不调用）
  - 松手结果为 `(140.0, 80.0)`（`round(n/grid)*grid`）

## ADDED — 2. UT 用例 > UT-CR-07 — 连线使用表的当前视觉坐标

- **位置**：`frontend-rs/src/editor_render.rs::calc_path` / `draw_canvas`
- **前置**：表 A (100,100) 与表 B (400,100) 之间已有 reference
- **步骤**：将表 A 的绘制坐标改为 (160, 140)（模拟拖动中临时坐标），调用路径计算
- **断言**：起点 x/y 随 160/140 变化，不得仍使用 100/100

## ADDED — 3. ST 用例 > ST-CR-02 — 拖表过程中连线路径更新（e2e）

- **位置**：`frontend-rs/tests/e2e/11_canvas_interaction.spec.ts`（或新 `18_canvas_drag_follow.spec.ts`）
- **步骤**：
  1. 画布上已有两表一条关系
  2. 标题栏 pointerdown 后 pointermove 至少 40px，**在 pointerup 之前**采样连线几何（Canvas 像素或暴露的 debug 路径）
  3. pointerup
- **断言**：
  - 移动过程中路径已偏离 pointerdown 时的几何（跟手，非松手才跳变）
  - 松手后表坐标为 `GRID_SIZE` 的倍数

## MODIFIED — 附录 A：用例 ID 清单（OpenLogos verify 解析用）

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
| ST-CR-02 | 拖表过程中连线路径更新 | e2e canvas drag follow |
