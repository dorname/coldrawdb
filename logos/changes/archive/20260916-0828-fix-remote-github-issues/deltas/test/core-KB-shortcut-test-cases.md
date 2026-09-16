# Delta — core-KB-shortcut-test-cases.md（#4 关系进 undo + Ctrl+Y）

> 变更：fix-remote-github-issues / GitHub #4

## ADDED — UT-KB-02 — Ctrl+Y 判定为 redo

### UT-KB-02 — `is_redo_shortcut` 支持 Ctrl/Cmd+Y

- **位置**：`frontend-rs/src/editor_panels.rs::modals::is_redo_shortcut`
- **步骤 / 断言**：
  1. `is_redo_shortcut("y", true, false) == true`（Ctrl+Y）
  2. `is_redo_shortcut("Y", true, false) == true`
  3. `is_redo_shortcut("z", true, true) == true`（既有 Ctrl+Shift+Z 仍成立）
  4. `is_redo_shortcut("y", false, false) == false`
  5. `is_redo_shortcut("y", true, true) == false`（带 Shift 不抢 Ctrl+Shift+Y）

## ADDED — UT-KB-03 — AddReference 进 undo 栈；undo 只撤关系不删表

### UT-KB-03 — 创建关系记录 `Command::AddReference`；Ctrl+Z 对称删除关系

- **位置**：`frontend-rs/src/editor_core.rs::CommandStack` + 创建关系通路（`on_create_reference` / 等价）
- **前置**：store 含 2 张表；undo 栈可为空或已有 `AddTable`
- **步骤**：
  1. 经正式创建通路写入一条 `Reference`（不得直接绕过 CommandStack）
  2. 断言 undo 栈顶为 `AddReference` 且 `references.len()==1`、`tables.len()` 不变
  3. `undo()` + `revert` → `references.len()==0`，**表数量不变**
  4. `redo()` + `execute` → 关系恢复
- **断言**：禁止「撤关系却删表」；创建关系必须 `record`/`apply(AddReference)`

## ADDED — UT-KB-04 — undo revert 失败时栈状态回滚

### UT-KB-04 — `revert`/`execute` 失败不得留下「已 pop 未生效」的空弹

- **位置**：Undo/Redo 按钮与 `KeyboardShortcuts` 调用点
- **步骤**：模拟对不支持 revert 的命令（或强制 Err）执行 undo
- **断言**：若副作用失败，undo/redo 栈恢复到调用前；或调用方先尝试副作用再改栈（二选一，但不得静默丢命令）

## MODIFIED — ST-UI-05 — 增补 Ctrl+Y

### ST-UI-05 — Ctrl+Z / Ctrl+Shift+Z / Ctrl+Y 键盘快捷键

- 原合同保留 Ctrl+Z / Ctrl+Shift+Z
- **ADDED**：建关系后 Ctrl+Z 撤关系不删表；再 Ctrl+Y 恢复关系

## MODIFIED — 附录 A：用例 ID 清单

| ID | 标题 | 对齐实现 |
|---|---|---|
| UT-MM-15 | CommandStack::undo 弹出最近命令 | `editor_core.rs::CommandStack::undo` |
| UT-MM-16 | CommandStack::redo 弹出最近 undo | `editor_core.rs::CommandStack::redo` |
| UT-KB-01 | 键盘事件 Ctrl+Z 触发 undo | `editor_panels.rs::modals::is_undo_shortcut` |
| UT-KB-02 | Ctrl+Y / Ctrl+Shift+Z 触发 redo | `editor_panels.rs::modals::is_redo_shortcut` |
| UT-KB-03 | AddReference 进栈；undo 只撤关系 | `editor_core.rs` + `on_create_reference` |
| UT-KB-04 | revert 失败时栈回滚 | UndoRedoButtons / KeyboardShortcuts |
| ST-UI-05 | Ctrl+Z / Ctrl+Shift+Z / Ctrl+Y e2e | `frontend-rs/tests/` |
