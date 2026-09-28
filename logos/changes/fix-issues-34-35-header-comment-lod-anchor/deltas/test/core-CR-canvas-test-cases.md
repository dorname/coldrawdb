# Delta — core-CR-canvas-test-cases.md（fix-issues-34-35 / #35）

> 模块：core | 提案：fix-issues-34-35-header-comment-lod-anchor
> 目标：UT-CR-LOD-01 / ST-CR-LOD-01 随 §5.12 R-LOD-08（表级锚定）与 R-LOD-02 注释补齐修订断言。不新增用例 ID。

## MODIFIED — UT-CR-LOD-01 — LOD 档位判定与参数纯函数（fix-issues-34-35 / #35 扩展）

- **位置**：`frontend-rs/src/editor_render.rs`（`lod_tier` / `lod_table_font_size` / `lod_line_width` / 表级锚点纯函数）
- **步骤与预期**：
  1. `lod_tier(0.5)` / `lod_tier(0.35)` → 拓扑档；`lod_tier(0.56)` / `lod_tier(1.0)` → 详情档；边界滞回 ±0.03 内保持原档不抖动
  2. `lod_table_font_size(0.5)` → 世界字号使屏幕字号 ≥ 11px（22px@0.5）；`lod_table_font_size(0.35)` 触发夹紧上限；详情档返回默认字号
  3. `lod_line_width(0.5, base)` → ≥ `1.5 / 0.5 = 3px` 世界宽；补偿倍率夹紧 ≤ 4×
  4. LOD 档位进入 R-PERF-07 精灵指纹：同表跨档指纹不同、同档内指纹相同（拖动不触发重光栅）
  5. **表级锚点（R-LOD-08，#35）**：拓扑档下关系锚点纵坐标 = `table.y + TABLE_HEADER_HEIGHT / 2`（表头中线），横坐标 = 卡体左/右缘（选侧沿用 `pick_port_sides`）；详情档同输入保持字段锚点（`field_anchor_for_side`）不变；bezier / orthogonal / straight 三路径函数同口径
  6. **拓扑档注释（R-LOD-02 补齐，#35）**：NameComment 模式且表注释非空 → 拓扑卡宽度估算覆盖「表名 + 注释」（`resolve_table_width` 注释感知）；空注释/非 NameComment 模式不增宽、不渲染

## MODIFIED — ST-CR-LOD-01 — 小缩放拓扑可读性 e2e（fix-issues-34-35 / #35 扩展）

- **位置**：`frontend-rs` e2e（`scripts/test-spec-parity-d.mjs`）
- **GIVEN**：导入 fixture 图（≥20 表、≥20 关系，含 ≥5 对已知 FK 表对；≥2 张表含中文注释）；注释显示模式 = 英文名+注释
- **WHEN**：缩放至 50% 与 35% 各观察一次；随后缩放回 100%
- **THEN**：
  1. 50% / 35% 下表卡进入拓扑档（字段行隐藏、仅表头），表名文字可见且屏幕字号 ≥ 11px（canvas 探针或截图锚点）
  2. 关系线可见且明显粗于默认（探针断言线宽 ≥ 补偿下限）
  3. 选中一张表后其关系连线按 §4.4 × R-LOD-04 叠乘加粗，拓扑可追踪
  4. **拓扑档中文注释可见**（R-LOD-02 / #35：`__cdb_lod_probe` 暴露拓扑档注释渲染计数 > 0，或截图锚点）
  5. **关系线表级锚定**（R-LOD-08 / #35：探针 `anchor_mode == "table"`；同一表多条关系线端点纵坐标收敛表头中线，不再按字段高度扇形散开）
  6. 回到 100%：字段行恢复、字号线宽回默认、关系线恢复字段维度锚定（探针 `anchor_mode == "field"`），与现状渲染一致（无回归）
- **reporter**：`ST-CR-LOD-01` 写入 `logos/resources/verify/test-results.jsonl`

## MODIFIED — 附录 A（更新行）

| ID | 标题 | 对齐实现 |
|---|---|---|
| UT-CR-LOD-01 | LOD 档位判定与参数纯函数（含表级锚点 / 拓扑注释宽度） | `editor_render.rs::lod_tier` 等 |
| ST-CR-LOD-01 | 小缩放拓扑可读性 e2e（含注释可见 + 表级锚定探针） | e2e canvas LOD |
