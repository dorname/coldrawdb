# 变更提案：fix-floating-controls-statusbar-overlap

> module: core | created: 2026-11-01
> 关联 issue：#69（矮视口下画布底部悬浮控件被状态栏遮挡，整排按钮不可点）

## 变更原因

G 批 e2e（cr-click-width-rel-style / cr-comment-color 等 5 个用例）持续报 `<footer data-testid="status-bar"> intercepts pointer events`，正式验收中靠声明式兜底 PASS，掩盖了真实产品缺陷。真实栈仪器化探针（2026-11-01，issue #69）实锤：

- **1280×720**（Playwright 默认视口，亦是常见 720p 级窗口）：`.cdb-canvas-container` 实测 h=646、bottom=728，**溢出 `.cdb-main` 行框（h=586、bottom=668）60px**，伸入状态栏带（y=676..710）；底部悬浮条 `.cdb-floating-controls`（锚定容器 bottom-24px）随之部分没入状态栏，`canvas-comment-display` 中心点 elementFromPoint 命中 FOOTER——**按钮不可点**
- 1440×900 / 1600×900 正常（内容高度恰与行框一致，未触发溢出）

根因（Why）：`.cdb-main` 只声明了 `grid-template-columns`，**行是隐式 auto 行**——行高由内容（画布渲染缓冲的 CSS 高度反馈）决定而非 `minmax(0,1fr)` 约束；矮视口下行被内容撑到超过 `.cdb-app` 分配的 1fr 份额，容器溢出行框叠到状态栏下。叠加因素：悬浮条 `z-index:10` < 状态栏 `z-index:20`，重叠区点击一律被状态栏截获。

## 变更类型

代码级（纯 CSS 布局修复，不改交互语义、不改 API/DB、无 delta）

## 变更范围

- 影响的需求文档：无
- 影响的功能规格：无（悬浮控件可点击本就是既有规格意图，此为实现缺陷修复）
- 影响的业务场景：S01 编辑器画布（几何布局层）
- 影响的 API：无
- 影响的 DB 表：无
- 影响的编排测试：无
- 影响代码：
  - `frontend-rs/src/styles.css` `.cdb-main`（补 `grid-template-rows: minmax(0, 1fr)` 行约束）
  - `frontend-rs/src/styles.css` `.cdb-floating-controls`（z-index 10 → 30，防御性层级修正）

## 部署影响

- 是否需要部署：否
- 部署原因：前端样式修复，随下次常规发布携带；本提案仅负责本地验收
- 影响环境：无（本地验证经 trunk :8080）
- 是否涉及数据迁移：否
- 是否需要回滚预案：否
- 是否需要 smoke：否

## 变更概述

1. `.cdb-main` 补显式行约束 `grid-template-rows: minmax(0, 1fr)`：中间列（画布容器）行高锁定为 `.cdb-app` 分配的 1fr 份额，矮视口下不再被画布内容反馈撑高溢出到状态栏带；画布内容溢出部分由容器既有 `overflow: hidden` 裁剪（画布滚动/缩放语义不变）。
2. `.cdb-floating-controls` z-index 10 → 30：防御性保证悬浮条恒在状态栏之上可点（与 `.cdb-inspector` 同层 30 的既有分层口径一致）。

验收口径：1280×720 探针复测 `canvas-comment-display` 中心命中按钮本身；G 批 5 个用例真实通过（无声明式兜底依赖）；全量验收 Gate 3.6 PASS 且 ST-PU-27/28/29/30、ST-SP-LIST-* 等几何用例无回归。
