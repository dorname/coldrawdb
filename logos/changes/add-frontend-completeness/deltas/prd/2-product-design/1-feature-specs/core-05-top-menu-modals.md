## ADDED — §9.2 B5 测试 ID 索引（提案：add-frontend-completeness）

> 模块：core | 提案：add-frontend-completeness
> 路径：deltas/prd/2-product-design/1-feature-specs/core-05-top-menu-modals.md
> 对齐参考源：`core-05-top-menu-modals.md` §5.3/5.4/5.5/5.6/5.9 + §2.2 Edit 菜单

# B5 模态 + 快捷键 — 测试 ID 索引

## 1. 范围

B5 在 §3 的 9 个模态清单中，补齐最后 **5 个模态** + **键盘快捷键**：
- Import（§5.3）
- ImportSource（§5.4）
- Language（§5.5）
- SetTableWidth（§5.6）
- ConfigureCustomTypes（§5.9）
- 全局键盘：Ctrl+Z / Ctrl+Shift+Z（§2.2 Edit 菜单）

## 2. 测试 ID 索引

| TC ID | 描述 | 对齐实现 | B5 状态 |
|---|---|---|---|
| UT-MM-10 | Import 模态 SQL 解析（parse_sql_statements） | `editor_panels.rs::modals::parse_sql_statements` | ✅ B5 实现 |
| UT-MM-11 | SetTableWidth 模态宽度解析（parse_table_width） | `editor_panels.rs::modals::parse_table_width` | ✅ B5 实现 |
| UT-MM-12 | Language 模态验证（validate_language） | `editor_panels.rs::modals::validate_language` | ✅ B5 实现 |
| UT-MM-13 | ConfigureCustomTypes 增删（add/remove_custom_type） | `editor_panels.rs::modals::{add,remove}_custom_type` | ✅ B5 实现 |
| UT-MM-14 | ImportSource 模态选择解析（resolve_import_source） | `editor_panels.rs::modals::resolve_import_source` | ✅ B5 实现 |
| UT-MM-15 | CommandStack::undo 弹出最近命令 | `editor_core.rs::CommandStack::undo` | ✅ B5 实现 |
| UT-MM-16 | CommandStack::redo 弹出最近 undo | `editor_core.rs::CommandStack::redo` | ✅ B5 实现 |
| UT-KB-01 | 键盘事件 Ctrl+Z 触发 undo（is_undo_shortcut） | `editor_panels.rs::modals::is_undo_shortcut` | ✅ B5 实现 |
| ST-MM-02 | 端到端 Import 模态 SQL 解析 | `frontend-rs/tests/wasm/ui.rs` | ⏭️ B5 e2e |
| ST-MM-03 | ConfigureCustomTypes 关闭后跨刷新保留 | `frontend-rs/tests/wasm/ui.rs` | ⏭️ B5 e2e（V1 限制） |
| ST-UI-05 | Ctrl+Z / Ctrl+Shift+Z 键盘快捷键 e2e | `frontend-rs/tests/wasm/kb.rs` | ⏭️ B5 e2e |

## 3. B5 spec 修正

- 原 §7 编号 UT-MM-02（Edit → Undo → 撤销栈 -1）+ UT-MM-03（View → Zoom In → 画布放大 0.25x）也属于本 B5 范围。UT-MM-02 由 `CommandStack::undo` + 键盘 Ctrl+Z 覆盖；UT-MM-03（Zoom In）属画布交互，本 B5 不实现（V1 边界）。
- `ST-S02-04`（Import 模态 SQL 解析）+ `ST-S02-05`（Language 模态切换 i18n）是 backend `core-S02-test-cases.md` 中的端到端用例，**不在前端 B5 范围**。前端 B5 用 UT-MM-10~14 覆盖解析/校验纯函数。
- `ST-UI-05` 键盘快捷键 e2e 留 B5 wasm-pack test 接入后跑（UT-KB-01 已用纯函数覆盖 `is_undo_shortcut` 逻辑）。

## 4. B5 实施分解（避免单批过大）

按 rollback 条件，B5 可拆为 B5a + B5b 顺序执行：
- **B5a**：5 个剩余模态（UT-MM-10~14）+ 模态组件（Import/ImportSource/Language/SetTableWidth/ConfigureCustomTypes）
- **B5b**：键盘快捷键（UT-MM-15/16 + UT-KB-01）+ 修正 `core-implementation-checklist.md`

实际可按单批闭环（每批 5~6 UT）执行，避免无谓拆分。

## 5. 对齐参考源

- `core-05-top-menu-modals.md` §5.3 / §5.4 / §5.5 / §5.6 / §5.9 / §2.2
- `core-UI-modals-2-test-cases.md`（5 模态 UT 详细步骤）
- `core-KB-shortcut-test-cases.md`（键盘快捷键 UT 详细步骤）
- `frontend-rs/src/editor_panels.rs::modals`（B4 子模块扩展）
- `frontend-rs/src/editor_core.rs::CommandStack`（扩展 undo/redo）
