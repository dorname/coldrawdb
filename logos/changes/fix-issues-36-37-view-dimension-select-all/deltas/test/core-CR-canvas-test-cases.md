## MODIFIED — UT-CR-LOD-01 — 维度模式与可读性补偿纯函数（fix-issues-36-37 / #36 显式维度化改写）

- **位置**：`frontend-rs/src/editor_core.rs`（`ViewDimension` / 存储键 / `from_stored` / `next`）+ `frontend-rs/src/editor_render.rs`（维度→档位映射 / `lod_table_font_size` / `lod_line_width` / 表级锚点 / 注释感知宽度）
- **步骤与预期**：
  1. **存储合同（R-VIEW-DIM-04）**：`from_stored(None)` / `from_stored(非法值)` → Field；`"table"` → Table、`"field"` → Field；`as_str` 往返一致；`next` 双态互换；缺省维度 = Field
  2. **维度→档位映射（R-LOD-01 修订）**：Table → 拓扑渲染档、Field → 详情渲染档，与 zoom 无关（同一维度在 zoom 0.35 / 1.0 / 2.0 下档位相同）；断言 `lod_tier(zoom, …)` 阈值判定不再被渲染主循环消费（源码锚点）
  3. **维度内补偿（R-LOD-03/04）**：表维度 `lod_table_font_size(0.5)` → 屏幕字号 ≥ 11px（22px@0.5）、`lod_line_width(0.5, base)` ≥ 3px 世界宽且夹紧 ≤ 4×；字段维度任意 zoom 返回默认字号/线宽
  4. **精灵指纹（R-LOD-05 / R-VIEW-DIM-05）**：同表跨维度指纹不同、同维度内指纹相同（拖动不触发重光栅）
  5. **表级锚点（R-LOD-08，#35 沿用）**：表维度锚点纵坐标 = `table.y + TABLE_HEADER_HEIGHT / 2`，横坐标 = 卡体左/右缘；字段维度同输入保持字段锚点（`field_anchor_for_side`）不变；bezier / orthogonal / straight 三路径同口径
  6. **表维度注释（R-LOD-02，#35 沿用）**：NameComment 模式且表注释非空 → 表维度卡宽度估算覆盖「表名 + 注释」；空注释/非 NameComment 模式不增宽、不渲染

## MODIFIED — ST-CR-LOD-01 — 显式维度切换 e2e（fix-issues-36-37 / #36 改写）

- **位置**：`frontend-rs` e2e（`scripts/test-spec-parity-d.mjs`）
- **GIVEN**：导入 fixture 图（≥20 表、≥20 关系，含 ≥5 对已知 FK 表对；≥2 张表含中文注释）；注释显示模式 = 英文名+注释；默认字段维度
- **WHEN**：字段维度下缩放至 35% 观察；快捷键 `V` 切表维度，35% / 100% / 200% 各观察一次；ToolRail 维度按钮切回字段维度；画布右键菜单再切表维度；刷新页面
- **THEN**：
  1. **维度不随缩放串档（R-VIEW-DIM-01）**：字段维度 35% 下字段行仍展开（探针 `anchor_mode == "field"`）；表维度 35% / 100% / 200% 下始终仅表头（`anchor_mode == "table"`、字段行不渲染）
  2. **表维度补偿（R-LOD-03/04）**：表维度 35% 下表名屏幕字号 ≥ 11px、关系线宽 ≥ 补偿下限（探针断言）；`topo_comments > 0`（中文注释可见，#35 沿用）
  3. **三入口一致（R-VIEW-DIM-02/03）**：快捷键 / ToolRail / 右键菜单每次切换后 FloatingControls 指示文案（「维度：表」/「维度：字段」）与探针档位一致；ToolRail 按钮 `cdb-is-active` 态与维度一致
  4. **持久化（R-VIEW-DIM-04）**：刷新后仍为表维度（localStorage `cdb.view-dimension`）
  5. **缩放不触发切换**：任一维度内滚轮缩放跨越 50% 旧阈值，维度不变
- **reporter**：`ST-CR-LOD-01` 写入 `logos/resources/verify/test-results.jsonl`

## MODIFIED — 附录 A（更新行）

| ID | 标题 | 对齐实现 |
|---|---|---|
| UT-CR-LOD-01 | 维度模式与可读性补偿纯函数 | `editor_core.rs::ViewDimension` + `editor_render.rs` 维度映射 |
| ST-CR-LOD-01 | 显式维度切换 e2e（三入口 + 不随缩放串档） | e2e canvas view-dimension |
