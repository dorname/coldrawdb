# 变更提案：r6-button-motion

> module: core | created: 2026-06-21

## 变更原因

R5 完成后，界面美学诊断 **R6** 仍待落地：按钮缺少 `:focus-visible` 焦点环与 `primary-active` 按压反馈；Inspector / IO Drawer 打开时无 spring 风格入场；`styles.css` 存在 **两处 `@keyframes cdb-pulse` 定义**（保存圆点 opacity 版 ~L524 与 E6 Issues 徽章 scale 版 ~L2004），后者覆盖前者导致保存中圆点动画行为不确定。

## 变更类型

设计级 + 代码级（CSS 动效与微交互，无 API/DB 变更）

## 变更范围

- 影响的功能规格：
  - `core-0c-motion.md` — R6 按钮 focus/active + 面板 spring 入场 + pulse 命名拆分
  - `core-07-design-tokens.md` — `--cdb-easing-spring` / `--cdb-shadow-focus` Token
- 影响的测试用例：`core-PE-design-system-test-cases.md` — UT-R6 断言
- 影响的业务场景：无行为变更（纯视觉/可访问性）
- 影响的 API / DB / 编排测试：无

## 部署影响

- 是否需要部署：否
- 是否需要 smoke：否
- 是否涉及数据迁移：否

## 变更概述

**R6 按钮微交互 + 面板 spring 入场**：

1. **Focus Ring**：`.cdb-btn` / `.cdb-tool-btn` / `.cdb-tab--icon` 增加 `:focus-visible` + `var(--cdb-shadow-focus)`。
2. **Primary 按压态**：`.cdb-btn--primary:active` 使用 `--cdb-color-primary-active` + `translateY(1px)`。
3. **Spring 入场**：`.cdb-inspector`、`.cdb-main.cdb-has-io-drawer .cdb-io-drawer`、`.cdb-app-bar__overflow-menu` 使用 `--cdb-easing-spring` 的 slide/fade 动画。
4. **修复 pulse 冲突**：保存圆点改用 `@keyframes cdb-pulse-opacity`；E6 Issues 徽章保留 `@keyframes cdb-pulse`（scale）。
