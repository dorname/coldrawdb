# 画布渲染测试用例规格

> 模块：core | 提案：add-frontend-completeness
> 路径：`logos/resources/test/core-CR-canvas-test-cases.md`
> 对齐参考源：`core-01-editor-canvas.md` §5.3 + `core-04-side-panel-tabs.md` §8

## 1. 范围

画布渲染补全（B3 范围）：
- `EditorStore` 新增 `areas: RwSignal<Vec<Area>>` + `notes: RwSignal<Vec<Note>>`
- Canvas 从 `store.areas.get() / store.notes.get()` 渲染（替换当前占位 `Vec::new()`）
- 端点 drag：拖动 reference 端点 → 改 `start_field_id` / `end_field_id`
- Issues Tab「跳转」按钮：点击 issue → 选中对应 table + 闪烁

**对应实现**：
- `frontend-rs/src/editor_render.rs`（Canvas 组件 + draw_canvas 纯函数）
- `frontend-rs/src/editor_core.rs`（EditorStore 新增 areas/notes + load/snapshot 同步）
- `frontend-rs/src/editor_panels.rs`（IssuesTab 跳转按钮 + AreasTab/NotesTab 升级用 store 而非 Stub）

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

- **位置**：`frontend-rs/src/editor_render.rs`（纯函数 `snap_to_grid(x, y, grid)`）
- **前置**：`grid = 20.0`（生产）；原型对照 `grid = 12`
- **步骤**：
  1. 拖动中视觉坐标保持 `(133.4, 87.1)` 不调用 snap
  2. pointerup 调用 `snap_to_grid(133.4, 87.1, 20.0)`
- **断言**：
  - 拖动中函数未被用于量化（由集成约定：move 路径不调用）
  - 松手结果为 `(140.0, 80.0)`（`round(n/grid)*grid`）

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
