## ADDED — UT-KB-05 / UT-KB-06 / ST-KB-SEL-01（fix-issues-36-37 / #37 画布全选与可撤销删除）

### UT-KB-05 — Ctrl/Cmd+A 全选判定与门控纯函数

- **位置**：`frontend-rs/src/editor_panels.rs`（`is_select_all_shortcut` 纯函数 + `setup_editor_tool_shortcuts` 锚点）
- **步骤与预期**：
  1. `is_select_all_shortcut("a", ctrl=true, meta=false, …)` / `("A", …)` / `(meta=true)` → true；无修饰键 / 仅 Alt / 仅 Shift → false（R-KBSEL-01）
  2. 处理器锚点：select-all 分支位于文本目标门控（`shortcut_event_is_text_target`）与浮层门控**之后**；分支内调用 `prevent_default`（R-KBSEL-01 / R-KBSEL-04）
  3. 全选落账锚点：分支将 `store.tables` / `notes` / `areas` 全量 id 写入 `selected_table_ids` / `selected_note_ids` / `selected_area_ids`（关系经两端表选中自动高亮，不单独入集）

### UT-KB-06 — DeleteTables 命令级联快照与撤销/重做

- **位置**：`frontend-rs/src/editor_core.rs`（`Command::DeleteTables { tables, references }` 的 `apply` / `revert`）
- **步骤与预期**：
  1. 构造 3 表 + 2 关系（其一级联、其一独立）：`apply(DeleteTables { tables: [t1], references: [r1] })` → t1 与 r1 移除、r2 保留（R-KBSEL-03 级联快照由调用方组装，命令按快照执行）
  2. `revert` → t1 与 r1 完整恢复（id / 字段 / 坐标 / 关系端点逐一相等）；再 `apply`（redo）→ 再次移除，对称
  3. 多表批量：`tables` 长度 3 + 级联 `references` 长度 2 → 单条命令 revert 一次恢复全部（一次按键 = 一个撤销单元，R-KBSEL-02）
  4. 处理器锚点：Delete/Backspace 分支覆盖 `SelectionKind::Table` 与多选集非空路径；`on_delete_table` 改为经 CommandStack 入栈（锚点：`Command::DeleteTables`）

### ST-KB-SEL-01 — 全选 + 删除 + 撤销 e2e

- **位置**：`frontend-rs/scripts/test-spec-parity-d.mjs`
- **GIVEN**：导入 fixture（≥3 表、≥2 关系、≥1 便签、≥1 区域）；画布点击获得焦点
- **WHEN → THEN**：
  1. `Ctrl+A` → 页面无文本高亮（`window.getSelection().isCollapsed` 或选区为空），全部表/便签/区域入选中集（选中计数探针或多选样式锚点）（R-KBSEL-01）
  2. `Delete` → 画布图元清空（表/关系/便签/区域计数 = 0）（R-KBSEL-02）
  3. `Ctrl+Z` → 全部恢复（各类计数回到原值）；`Ctrl+Y` 再删除（对称 redo）（R-KBSEL-03）
  4. 单选一张表 → `Delete` → 该表与其级联关系消失，`Ctrl+Z` 恢复（R-KBSEL-03）
  5. 聚焦 Inspector 文本输入框 → `Ctrl+A` 不触发画布全选、`Delete` 仅删文本（画布选中集与图元不变）（R-KBSEL-04）
- **reporter**：`ST-KB-SEL-01` 写入 `logos/resources/verify/test-results.jsonl`

## MODIFIED — 附录 A：用例 ID 清单（追加行）

| ID | 标题 | 对齐实现 |
|---|---|---|
| UT-KB-05 | Ctrl/Cmd+A 全选判定与门控 | `editor_panels.rs::is_select_all_shortcut` |
| UT-KB-06 | DeleteTables 级联快照撤销/重做 | `editor_core.rs::Command::DeleteTables` |
| ST-KB-SEL-01 | 全选 + 删除 + 撤销 e2e | e2e test-spec-parity-d |
