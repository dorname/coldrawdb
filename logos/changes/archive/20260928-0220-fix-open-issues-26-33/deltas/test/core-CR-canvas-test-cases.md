# Delta — core-CR-canvas-test-cases.md（fix-open-issues-26-33 / #27 + #30）

> 模块：core | 提案：fix-open-issues-26-33
> 目标：
> - #27：区域 resize（画布手柄 + Inspector 宽高）用例，对齐 §5.11（R-ARESZ-01～06）。
> - #30：小缩放 LOD 分档用例，对齐 §5.12（R-LOD-01～07）。

## ADDED — §6.x 区域 resize（fix-open-issues-26-33 / #27）

### UT-AREA-02 — 区域 resize 几何纯函数（命中 / 夹紧 / 锚定）

- **位置**：`frontend-rs/src/editor_render.rs`（`area_resize_handles` / `hit_test_area_resize` / `area_rect_from_resize`）
- **步骤与预期**：
  1. `area_resize_handles(area)` 返回 8 个命中面（4 角 + 4 边中点），坐标与区域矩形一致
  2. `hit_test_area_resize`：手柄热区内命中返回对应方位（`E` / `SE` / `NW` …），区域内部（非手柄）不命中 resize（回退整框拖动），区域外不命中
  3. `area_rect_from_resize`：拖 `E` 边 → 仅 `width` 变化、`x` 锚定；拖 `NW` 角 → `x`/`y`/`width`/`height` 联动且 `SE` 角锚定
  4. 拖至小于 `AREA_MIN_WIDTH` / `AREA_MIN_HEIGHT` → 夹紧到下限，无负尺寸/翻转
  5. 只读模式调用命中判定 → 不命中（R-ARESZ-05）

### UT-AREA-03 — 区域 resize 落账与撤销

- **位置**：`frontend-rs/src/editor_render.rs` / `editor_core.rs`（`on_update_area` 写回通道 + CommandStack）
- **步骤与预期**：
  1. 模拟手柄拖拽 `pointermove` 期间不落账（无写回调用），`pointerup` 一次写回 `width`/`height`（必要时含 `x`/`y`）
  2. 写回进入 CommandStack 单条命令 → 一次 Undo 恢复 resize 前矩形
  3. Inspector 宽高输入 blur 落账同样产生单条命令；非法输入（`<` 下限 / 非数值）按 R-ARESZ-03/04 夹紧或忽略

### ST-CR-AREA-02 — 区域 resize e2e（手柄 + Inspector 双向）

- **位置**：`frontend-rs/scripts/test-spec-parity-d.mjs`
- **步骤**：`tool-new-area` → 拖框创建区域 → 选中 → 拖 `SE` 角手柄放大 ≥40px → mouseup → Inspector `inspector-area-width` / `inspector-area-height` 显示新值 → 改宽度数值 blur → 画布矩形同步 → 刷新页面
- **预期**：mouseup 后 PUT 落账 `areas[0].width/height` 更新；Inspector 与画布双向一致；刷新后尺寸保持；Undo 一次恢复原尺寸
- **reporter**：`ST-CR-AREA-02` 写入 `logos/resources/verify/test-results.jsonl`

### 附录 A 追加

| ID | 标题 | 对齐实现 |
|---|---|---|
| UT-AREA-02 | 区域 resize 几何纯函数 | `editor_render.rs::area_rect_from_resize` |
| UT-AREA-03 | 区域 resize 落账与撤销 | `editor_render.rs` / `editor_core.rs` CommandStack |
| ST-CR-AREA-02 | 区域 resize e2e（手柄 + Inspector） | e2e test-spec-parity-d |

## ADDED — §6.y 小缩放 LOD 分档（fix-open-issues-26-33 / #30）

### UT-CR-LOD-01 — LOD 档位判定与参数纯函数

- **位置**：`frontend-rs/src/editor_render.rs`（`lod_tier` / `lod_table_font_size` / `lod_line_width`）
- **步骤与预期**：
  1. `lod_tier(0.5)` / `lod_tier(0.35)` → 拓扑档；`lod_tier(0.56)` / `lod_tier(1.0)` → 详情档；边界滞回 ±0.03 内保持原档不抖动
  2. `lod_table_font_size(0.5)` → 世界字号使屏幕字号 ≥ 11px（22px@0.5）；`lod_table_font_size(0.35)` 触发夹紧上限；详情档返回默认字号
  3. `lod_line_width(0.5, base)` → ≥ `1.5 / 0.5 = 3px` 世界宽；补偿倍率夹紧 ≤ 4×
  4. LOD 档位进入 R-PERF-07 精灵指纹：同表跨档指纹不同、同档内指纹相同（拖动不触发重光栅）

### ST-CR-LOD-01 — 小缩放拓扑可读性 e2e

- **位置**：`frontend-rs` e2e（独立 spec 或并入 test-spec-parity）
- **GIVEN**：导入 fixture 图（≥20 表、≥20 关系，含 ≥5 对已知 FK 表对）
- **WHEN**：缩放至 50% 与 35% 各观察一次；随后缩放回 100%
- **THEN**：
  1. 50% / 35% 下表卡进入拓扑档（字段行隐藏、仅表头），表名文字可见且屏幕字号 ≥ 11px（canvas 探针或截图锚点）
  2. 关系线可见且明显粗于默认（探针断言线宽 ≥ 补偿下限）
  3. 选中一张表后其关系连线按 §4.4 × R-LOD-04 叠乘加粗，拓扑可追踪
  4. 回到 100%：字段行恢复、字号线宽回默认，与现状渲染一致（无回归）
- **reporter**：`ST-CR-LOD-01` 写入 `logos/resources/verify/test-results.jsonl`

### 附录 A 追加

| ID | 标题 | 对齐实现 |
|---|---|---|
| UT-CR-LOD-01 | LOD 档位判定与参数纯函数 | `editor_render.rs::lod_tier` |
| ST-CR-LOD-01 | 小缩放拓扑可读性 e2e | e2e canvas LOD |
