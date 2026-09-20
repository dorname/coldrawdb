# Delta — core-CR-canvas-test-cases.md（修改）

> 模块：core | 提案：fix-remote-github-issues-7-18
> 关联 issue：#10（注释展示）、#12（表颜色）、#17（圆角高亮） | 规格：`core-01-editor-canvas.md` §5.7/§5.8、`core-01a-table-and-field.md` §1.5/§1.6

## ADDED — UT-CR-COMMENT-01 — 注释渲染口径（三态 + 空值）

- **位置**：`frontend-rs/src/editor_render.rs`（表头/字段行文本布局纯函数）
- **断言**：
  - 模式 `name`：只返回技术名；`name+comment`（默认）：comment 非空时返回 (名称, 注释) 双文本；`comment`：仅注释（无 comment 回退技术名）
  - comment 为空：任何模式下返回结构与现状一致（无占位、行高不变）
  - 注释文本参与 R-PERF-07 精灵指纹（改 comment → 指纹变化）

## ADDED — UT-CR-COMMENT-02 — 显示开关持久化

- **位置**：`editor_panels.rs`（`canvas-comment-display` 开关）
- **断言**：切换写 `localStorage["cdb.comment-display"]` ∈ `name | name+comment | comment`；缺省/非法值回退 `name+comment`；切换触发 `schedule_paint`；不写 diagram 数据

## ADDED — UT-CR-COLOR-01 — 表边框与关系线用色回退

- **位置**：`frontend-rs/src/editor_render.rs`（取色纯函数）
- **断言**：`table.color` 非空 → 表边框用色派生自该值，为空 → `palette.table_border`；`ref.color` 同口径（见 UT-PB-11）

## ADDED — UT-CR-GHOST-02 — 幽灵层圆角高亮（R-HL-02）

- **位置**：`frontend-rs/src/editor_render.rs::create_table_ghost`
- **断言**：幽灵层样式不含 `outline:` 高亮（改 `border`/`border-radius:16px` 或 canvas `round_rect` 描边）；`include_str!` 锚点 + 样式字符串断言

## ADDED — ST-CR-COMMENT-01 — e2e：中文注释画布可见

- **GIVEN**：导入含中文 COMMENT 的 SQL（复用 ST-PC-09 数据）
- **WHEN**：导入完成 → 画布查看表头与字段行 → 切显示开关三态
- **THEN**：默认模式可见中文注释；`name` 模式注释消失；空 comment 表无占位；刷新（localStorage 恢复）后模式保持

## ADDED — ST-CR-COLOR-01 — e2e：表颜色配置闭环

- **WHEN**：Inspector 选表 → 设置颜色 → 保存 → 刷新；导出 JSON 再导入
- **THEN**：表头与表边框即时变色；刷新后保留；JSON round-trip 后颜色仍在；设为「默认」后回退主题色

> 附录 A（用例 ID 清单）追加登记：UT-CR-COMMENT-01 / UT-CR-COMMENT-02 / UT-CR-COLOR-01 / UT-CR-GHOST-02 / ST-CR-COMMENT-01 / ST-CR-COLOR-01。
> 全部用例结果写入 `logos/resources/verify/test-results.jsonl`（`module: "core"`，`scenario: "S01"`）。
