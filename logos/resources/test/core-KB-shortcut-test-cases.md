# 全局键盘快捷键测试用例规格

> 模块：core | 提案：add-frontend-completeness
> 路径：`logos/resources/test/core-KB-shortcut-test-cases.md`
> 对齐参考源：`core-05-top-menu-modals.md` §2.2 Edit 菜单 + `editor_core.rs::CommandStack`

## 1. 范围

快捷键与主原型一致处：⌘K/Ctrl+K、Esc、T/R（建表/关系）等。

状态：后端已实现；生产前端部分接入。本提案 `implement-unified-prototype-spec-parity`（D 批）将 ST-KB-* 落实为自动化，结果写入 `logos/resources/verify/test-results.jsonl`。不得将「规格已写」标为「生产已完成」。

## ADDED / MODIFIED

| ID | 前置 | 操作 | 预期 | 状态 |
|---|---|---|---|---|
| ST-KB-CMD-01 | room-editor | ⌘K / Ctrl+K | 打开 `command-palette`（主原型入口 `tool-search`）；再 Esc 关闭无残留 | 本提案 D 批实现 |
| ST-KB-ESC-01 | 任意浮层 | Esc | 按层级关闭最上层；不误关编辑器页 | 本提案 D 批实现 |
| ST-KB-T-01 | 可写 | 按 `T`（无输入焦点） | 触发建表工具/新建表（与主原型 tool tip 一致） | 本提案 D 批实现 |
| ST-KB-R-01 | 可写 | 按 `R` | 进入关系工具 | 本提案 D 批实现 |
| UT-KB-01 / UT-MM-15/16 / ST-UI-05 | 既有 | 撤销重做 | 保留；输入框焦点时快捷键不抢焦点 | 既有；D 批回归 |
| ST-KB-VIEWER | Viewer | T/R | 不创建；只读 | 本提案 D 批实现（ADDED 用例） |

## 2. UT 用例

### UT-MM-15 — CommandStack::undo 弹出最近命令

- **位置**：`frontend-rs/src/editor_core.rs::CommandStack::undo`
- **步骤**：
  1. 创建 `CommandStack::new()`
  2. `apply(AddTable(table))` 一次
  3. `undo()` 返回 `(Some(cmd), _)`，其中 `cmd == AddTable(table)`
- **断言**：
  - `stack.undo.is_empty()` 后
  - `stack.redo` 长度为 1
  - 多次 undo 直到空 → 返回 `(None, _)`

### UT-MM-16 — CommandStack::redo 弹出最近 undo

- **位置**：`frontend-rs/src/editor_core.rs::CommandStack::redo`
- **步骤**：
  1. `apply(AddTable)` 一次
  2. `undo()` 一次（push 到 redo）
  3. `redo()` 返回 `(Some(cmd), _)`
- **断言**：
  - redo 后 `stack.undo` 长度为 1，`stack.redo` 为空
  - redo 空 stack → `(None, _)`

### UT-KB-01 — 键盘事件 Ctrl+Z 触发 undo（纯函数路径）

- **位置**：`frontend-rs/src/editor_panels.rs::modals::is_undo_shortcut`
- **步骤**：传入 keydown event `{ key: "z", ctrlKey: true, shiftKey: false, metaKey: false }`
- **断言**：
  - `is_undo_shortcut(&ev) == true`
  - `{ ctrlKey: true, shiftKey: true }` → `is_undo_shortcut` 返回 false（属于 redo）
  - `is_redo_shortcut(&ev) == true`
  - 不带 ctrl/meta → 都返回 false

## 3. ST 用例

### ST-UI-05 — Ctrl+Z / Ctrl+Shift+Z 键盘快捷键 e2e

- **位置**：`frontend-rs/tests/wasm/kb.rs`（B5 接入）
- **类型**：wasm-pack test --headless --chrome
- **步骤**：
  1. 创建 table "t1"
  2. 按 Ctrl+Z → table 消失（undo）
  3. 按 Ctrl+Shift+Z → table 恢复（redo）
- **B5 标记 skip**：完整 e2e 跑在 B5 wasm-pack test 接入后

## 边界

未在主原型出现的自定义快捷键不纳入本提案合同；Space 平移等 V1 占位保持边界。

## 4. V1 边界

- ❌ Space + 拖拽画布平移（V1 仅占位，B5 不实现）
- ❌ 自定义快捷键（V1 硬编码 drawdb 默认）
- ❌ Delete/Backspace 删除 table 完整反向命令（V1 undo/redo 仅 CommandStack 底座）

## 5. 对齐参考源

- `core-05-top-menu-modals.md` §2.2 Edit 菜单
- `frontend-rs/src/editor_panels.rs::KeyboardShortcuts`（新增）
- `frontend-rs/src/editor_core.rs::CommandStack`（扩展 undo/redo）

## 附录 A：用例 ID 清单（OpenLogos verify 解析用）

| ID | 标题 | 对齐实现 |
|---|---|---|
| UT-MM-15 | CommandStack::undo 弹出最近命令 | `editor_core.rs::CommandStack::undo` |
| UT-MM-16 | CommandStack::redo 弹出最近 undo | `editor_core.rs::CommandStack::redo` |
| UT-KB-01 | 键盘事件 Ctrl+Z 触发 undo | `editor_panels.rs::modals::is_undo_shortcut` |
| ST-UI-05 | Ctrl+Z / Ctrl+Shift+Z 键盘快捷键 e2e | `frontend-rs/tests/wasm/kb.rs`（B5） |
