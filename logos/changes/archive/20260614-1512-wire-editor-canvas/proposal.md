# 变更提案：wire-editor-canvas

> module: core | created: 2026-06-14

## 变更原因

1. `AppRoot` 曾使用 B3 占位符，未接入 `editor_render::Canvas`；用户只见开发占位文字。
2. 界面 CSS 类缺失、grid 行数错位导致 Tab 重叠、按钮漂浮，基本可用性极差。
3. 与 drawdb 参考界面差距大：无浮动工具栏、空状态、动态 SaveState、画布点阵背景。

## 变更类型

代码级修复 + UI 打磨

## 变更范围

- 影响的功能规格：`core-01-editor-canvas.md` §5.3；`core-05-top-menu-modals.md` §1–2
- 影响的 smoke 测试：HP-01 / HP-05
- 影响的源文件：
  - `frontend-rs/src/editor_panels.rs` — Canvas 接入、FloatingControls、顶栏/侧栏/右栏优化
  - `frontend-rs/src/editor_render.rs` — 动态网格、Zoom API
  - `frontend-rs/src/styles.css` — 补全缺失类、点阵背景、浮动条、空状态
  - `scripts/start-local.sh` — trunk 启动前清除 `NO_COLOR`

## 部署影响

- 是否需要部署：否
- 是否需要 smoke：是

## 变更概述

分三批交付：（1）修复 CSS/布局/画布网格基线；（2）drawdb 视觉对齐（浮动工具栏、空状态、顶栏 SaveState、侧栏建表）；（3）标题编辑、View Zoom、Share 统一、Tables 列表增强。
