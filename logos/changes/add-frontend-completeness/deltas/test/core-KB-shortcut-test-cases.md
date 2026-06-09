# 全局键盘快捷键测试用例规格

> 模块：core | 提案：add-frontend-completeness
> 路径：`logos/resources/test/core-KB-shortcut-test-cases.md`
> 对齐参考源：`core-05-top-menu-modals.md` §2.2 Edit 菜单 + `editor_core.rs::CommandStack`

## 1. 范围

B5 全局键盘事件监听：
- `Ctrl/Cmd + Z` → 撤销栈弹一步
- `Ctrl/Cmd + Shift + Z` → 重做栈弹一步
- `Delete / Backspace` → 删除选中对象（spec core-01 §3 手势；B5 接 store hook）
- `Ctrl/Cmd + D` → 复制选中（V1 stub：toast 提示）
- `Space + 拖拽` → 画布平移（V1 不实现拖拽；B5 仅占位）
- `Ctrl/Cmd + S` → 强制保存（绕过 debounce）

**对应实现**：
- `frontend-rs/src/editor_panels.rs::KeyboardShortcuts`（新增组件）
- `frontend-rs/src/editor_core.rs::CommandStack::{undo, redo}`（新增方法）

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
