# 变更提案：r4-appbar-hierarchy

> module: core | created: 2026-06-21

## 变更原因

R1–R3 完成后，界面美学诊断 **R4** 仍待落地：AppBar 48px 单行同时承载 Logo、撤销/重做、标题、保存状态、rev、导入/导出 pill、分享、代码视图、主题、Inspector 切换等 **10+ 控件**，色彩与视线锚点分散；保存状态与标题 dirty 点重复表达「未保存」；`revision-display` 在 AppBar 与 StatusBar **双份渲染**。

## 变更类型

设计级 + 代码级（AppBar 布局与 CSS，无 API/DB 变更）

## 变更范围

- 影响的功能规格：
  - `core-05-top-menu-modals.md` — §1 AppBar 三区布局 + 溢出菜单
  - `core-00-information-architecture.md` — §1 AppBar 信息架构描述
  - `core-07-design-tokens.md` — §15.2 AppBar 分区间距 Token
- 影响的页面原型：`core-01-editor-prototype.html` — AppBar 结构对齐
- 影响的业务场景：S01（保存状态 / revision 锚点不变）
- 影响的 API / DB / 编排测试：无（保留全部 `data-testid`）

## 部署影响

- 是否需要部署：否
- 部署原因：前端 WASM/CSS 变更随常规 `trunk build` 验证，无独立部署步骤
- 影响环境：无（开发构建即可验证）
- 是否涉及数据迁移：否
- 是否需要 smoke：否

## 变更概述

**R4 AppBar 信息分层**：将 AppBar 划分为 **品牌区**（Logo + Undo/Redo + 标题）、**状态区**（单一 `.cdb-status-chip`：圆点 + 文案 + rev）、**操作区**（分享 Primary + 代码视图 + `⋯` 溢出菜单）。导入/导出/主题切换迁入溢出菜单；移除 AppBar 上重复的 Inspector 切换（保留 StatusBar `btn-inspector-toggle`）；标题旁 dirty 点移除（由状态 Chip 统一表达）。

**视觉**：操作区间距对齐 4px 网格 Token；状态 Chip 使用 `bg-subtle + border` 单一容器，降低 48px 行内色彩竞争。
