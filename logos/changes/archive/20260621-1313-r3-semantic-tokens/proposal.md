# 变更提案：r3-semantic-tokens

> module: core | created: 2026-06-21

## 变更原因

R1+R2 完成后，界面美学诊断 R3 项仍待落地：`styles.css` 组件层存在 **11 处硬编码色**（`#fef2f2`、`rgba(15,23,42,0.48)` 等），暗色模式下 Issue 徽章 / 模态遮罩 / 画布网格不会随 Token 切换；`prefers-color-scheme: dark` 仅覆写 3 个 Token，系统跟随暗色时出现「半暗半亮」界面。

## 变更类型

设计级 + 代码级（CSS Token 贯通，无 API/DB 变更）

## 变更范围

- 影响的功能规格：
  - `core-07-design-tokens.md` — 新增 surface / focus 语义 Token
  - `core-0b-dark-mode.md` — 补全系统偏好 media query 完整映射
- 影响的业务场景：S01（编辑器 UI 暗色一致性）
- 影响的 API / DB / 编排测试：无

## 部署影响

- 是否需要部署：否
- 是否需要 smoke：否
- 是否涉及数据迁移：否

## 变更概述

**R3 语义 Token 贯通**：新增 `--cdb-color-canvas-grid`、`--cdb-color-inspector-edge`、`--cdb-color-focus-error`、`--cdb-color-error-hover-bg` 四组 surface Token；组件层 Issue/Badge/Overlay/Canvas/Form 全部改用 `var(--cdb-color-*)`；`white` 字面量改为 `--cdb-color-primary-on`。

**暗色模式一致性**：`@media (prefers-color-scheme: dark)` 与 `[data-mode="dark"]` 同步完整 Token 映射（含新增 surface Token 暗色值）。
