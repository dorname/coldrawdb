# Delta — core-PB-relationship-test-cases.md（fix-open-issues-26-33 / #28）

> 模块：core | 提案：fix-open-issues-26-33
> 目标：批量修改全部关系线条类型用例，对齐 §4.6。

## ADDED — UT-PB-17 — 批量 line_type 命令纯函数与单次 Undo

- **位置**：`frontend-rs/src/editor_core.rs` / `editor_render.rs`（批量应用函数，如 `apply_line_type_to_all(references, line_type)` + CommandStack 事务）
- **断言**：
  1. 含 3 条不同 `line_type` 的 references → `apply_line_type_to_all(..., "straight")` 后全部 `line_type=="straight"`，其余字段（含 `stroke_style` / `color`）不变
  2. 空 references → 无命令产生（入口禁用口径的纯函数侧保证）
  3. 批量应用入 CommandStack 为**单条命令** → 一次 undo 后 3 条关系各自恢复批量前值（不串味、不丢条）
  4. 已是目标值的关系不产生多余 diff（批量后 dirty 仅含实际变化；全部已是目标值时无命令）

## ADDED — ST-PB-10 — 「应用到全部关系」+ 命令面板 e2e（#28）

- **位置**：`frontend-rs` e2e（test-spec-parity 或独立 spec）
- **GIVEN**：画布含 ≥3 条关系（混合 bezier / orthogonal）
- **WHEN**：选中一条关系 → Inspector 线条类型选「直线」→ 点 `inspector-rel-line-type-apply-all`；随后刷新页面；再执行 Undo；最后经 Command Palette 执行 `palette-action-line-type-orthogonal`
- **THEN**：
  1. apply-all 后全部关系渲染为直线（连线路径无贝塞尔弯），PUT 快照全部 `line_type=="straight"`
  2. 刷新后线型保持
  3. 一次 Undo 后全部恢复批量前各自值
  4. palette 命令后全部关系 `line_type=="orthogonal"`
  5. 只读分享页中 apply-all 按钮禁用 / palette 命令不生效
- **reporter**：`ST-PB-10` 写入 `logos/resources/verify/test-results.jsonl`

### 用例登记追加

| ID | 描述 |
|---|---|
| UT-PB-17 | 批量 line_type 命令纯函数与单次 Undo |
| ST-PB-10 | 应用到全部关系 + 命令面板 e2e |

## MODIFIED — UT-PB-15 — 选中态密度降噪（#21；fix-open-issues-26-33 / #31 强化数值口径）

- **THEN**：非相关线 alpha ≤ 0.25；非相关表 alpha ≤ 0.5；相关线 alpha = 1 且线宽 ≥ 默认 1.5×；邻接表（相关表）alpha = 1 不被误伤；无选中时全部恢复正常
