# 变更提案：optimize-ui-prototypes

> module: core | created: 2026-06-20

## 变更原因

S01–S05 HTML 原型在 `refresh-editor-prototype` 归档后已可用，但对照 **ui-ux-pro-max** 交付清单与 **core-07/08/0a** 规格仍存在差距：

1. **图标**：Tool Rail / AppBar 仍用 unicode emoji（↶、⊕、🔗 等），违反 E2「禁止 emoji 占位」
2. **Token 重复**：5 个原型各自内联 CSS，dark mode / z-index / shadow 不一致
3. **缺失 E4 交互**：编辑器原型未演示 Command Palette（`Ctrl+K`）与 Code View 全屏模态
4. **无障碍**：部分原型缺少 `:focus-visible` 环、`prefers-reduced-motion`、icon-only 按钮 `aria-label`

## 变更类型

设计级变更（Phase 2 产品设计层 — 原型与交互规格微调）

## 变更范围

- 影响的需求文档：无
- 影响的功能规格：`core-S01-edit-and-save-design.md`（补充 E4 原型锚点）
- 影响的业务场景：S01（主）、S03/S04/S05（视觉统一）
- 影响的 API：无
- 影响的 DB 表：无
- 影响的编排测试：无

## 部署影响

- 是否需要部署：**否**（仅规格/原型文档）
- 部署原因：文档变更，不涉及运行时
- 影响环境：无
- 是否涉及数据迁移：否
- 是否需要回滚预案：否
- 是否需要 smoke：否

## 变更概述

1. **新增** `core-00-prototype-shared.css`：统一 token 子集、按钮/图标/焦点/动效降级基类，供 5 个 HTML 原型 `@import`。
2. **升级** `core-01-editor-prototype.html`：SVG 图标、Command Palette（`Ctrl+K`）、Code View 模态、键盘 Esc 关闭。
3. **统一** S03/S04/S05 原型：引用共享 CSS、替换 emoji、补齐 focus-visible 与 reduced-motion。
4. **更新** S01 交互设计文档：补充 `[data-testid="command-palette"]` / `[data-testid="code-view-modal"]` 锚点说明。
