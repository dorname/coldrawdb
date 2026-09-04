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
