# Delta: core-PC-import-export-test-cases.md

> 提案：fix-select-bg-diagram-delete-and-log-format

## ADDED — 汇总表行（ST-PC-06 行后）

| UT-PC-23 | — | `include_str!` 锚点 | `styles.css` 中命中 select 的复合选择器（`.cdb-create-room-modal .cdb-input` / `.cdb-auth-card .cdb-input`）不含 `background:` 简写（仅 `background-color`）；暗色 chevron 规则含 `background-repeat: no-repeat` |

## ADDED — 用例详情

### UT-PC-23 — 样式锚点：select 背景简写修复

- **位置**：`editor_panels::tests::test_select_background_shorthand_anchor_ut_pc_23`
- **断言**：
  1. `.cdb-create-room-modal .cdb-input` 与 `.cdb-auth-card .cdb-input` 规则块不含 `background:` 简写（防止重置 chevron image/repeat）
  2. `[data-mode="dark"] .cdb-form-select, [data-mode="dark"] .cdb-select` 规则块含 `background-repeat: no-repeat` 与 `background-position`（双保险）

## ADDED — 变更记录行

| UT-PC-23 | ADDED（fix-select-bg-diagram-delete-and-log-format） | select 背景简写平铺回归锚点 |
