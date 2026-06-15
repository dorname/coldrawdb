# 变更提案：redesign-phase-d-command-code

> module: core | created: 2026-06-14
> 前置：`redesign-phase-c-import-export` 已归档（Phase C IO 抽屉已落地）

## 变更原因

Phase A 移除 280px 左栏 7 Tab 后，用户失去**全局浏览与跳转**能力（仅 Issues Tab 保留在 Tool Rail 徽章内）。Phase A/C 规格明确将下列能力推迟至 Phase D：

1. **Command Palette（`Ctrl+K`）** — 替代左栏 Tables / Areas / Enums / Notes / Relationships / Types 列表浏览与全局搜索（搁置用例 UT-SP-09、UT-SP-10）。
2. **SQL/DBML 全屏代码视图** — AppBar `btn-code-view` 占位未实现；用户无法从画布一键查看当前 schema 的 SQL/DBML 全文。
3. Phase C 已在客户端实现 `export_diagram_sql` / `export_diagram_dbml`，但未提供全屏阅读场景；导入仍走 IO 抽屉，代码视图 V1 为**只读预览 + 复制**。

Phase D 完成 V2 重规划最后一环：恢复浏览能力（Palette）+ 补齐代码视图，不恢复左栏 UI。

## 变更类型

设计级 + 代码实现（Command Palette + Code View UI）

## 变更范围

- **新增** `core-01e-command-palette.md` — Command Palette 完整规格
- **新增** `core-01f-code-view.md` — SQL/DBML 全屏代码视图规格
- `core-00-information-architecture.md` — 视图模式、Palette z-index、Phase D 边界
- `core-04-side-panel-tabs.md` — 浏览能力迁移至 Palette 说明
- `core-05-top-menu-modals.md` — `btn-code-view` 启用；View 菜单接线
- `core-01-editor-canvas.md` — Canvas ↔ Code 视图切换
- `core-01-editor-prototype.html` — Palette + Code View 视觉原型
- **新增** `core-PD-command-code-test-cases.md`
- 代码：`editor_panels.rs`、`styles.css`；复用 Phase C 导出纯函数

## 影响的业务场景

- 编辑器内全局搜索跳转（替代左栏列表）
- AppBar / View 菜单 → 代码视图切换
- 键盘 `Ctrl+K` 快捷入口

## 影响的 API

- 无新增端点（代码视图只读预览；Palette 跳转纯前端）

## 影响的 DB 表

- 无 schema 变更

## 部署影响

- 是否需要部署：否
- 部署原因：纯前端 WASM 交互增强
- 影响环境：无
- 是否涉及数据迁移：否
- 是否需要回滚预案：否
- 是否需要 smoke：否（本地 UT + 可选 e2e ST-PD-01）

## 变更概述

### Phase D 交付物

1. **CommandPalette**（`data-testid="command-palette"`）：`Ctrl+K` / `Cmd+K` 打开居中浮层；模糊搜索表/区域/枚举/便签/关系/类型；Enter 跳转并选中；Esc 关闭。
2. **CodeView**（`data-testid="code-view"`）：AppBar `btn-code-view` 切换全屏；SQL/DBML Tab 只读预览（复用 `export_diagram_*`）；复制按钮；返回画布。
3. **视图互斥**：Code 视图隐藏 Tool Rail、Inspector、IO 抽屉；Palette 与模态互斥（打开 Palette 时关闭其他浮层）。
4. **恢复搁置用例**：UT-SP-09 / UT-SP-10 迁移为 Palette 等价断言（UT-PD-07 / UT-PD-08）。

### V1 边界（不在 Phase D）

- 代码视图**双向编辑**（粘贴 SQL 应用回画布）— 后续迭代
- Mermaid / PNG 导出
- Palette 高级命令（改主题、批量删除等）
- Monaco / CodeMirror 集成（V1 用 `<textarea readonly>`）
