# 实现任务 — redesign-phase-a-layout

> Phase A：布局重构规格 delta。代码实现待 merge 后按 `[code]` section 执行。

## [delta] 规格变更 — Phase A

- [x] 产出 delta → `deltas/prd/2-product-design/1-feature-specs/core-00-information-architecture.md` — 顶层布局 V2
- [x] 产出 delta → `deltas/prd/2-product-design/1-feature-specs/core-01-editor-canvas.md` — 布局栅格、空白引导、选中态
- [x] 产出 delta → `deltas/prd/2-product-design/1-feature-specs/core-04-side-panel-tabs.md` — 左栏 → Tool Rail 迁移
- [x] 产出 delta → `deltas/prd/2-product-design/1-feature-specs/core-05-top-menu-modals.md` — AppBar 单行布局
- [x] 产出 delta → `deltas/prd/2-product-design/1-feature-specs/core-06-inspector-panel.md` — Inspector 抽屉（新文件）
- [x] 产出 delta → `deltas/prd/2-product-design/2-page-design/core-01-editor-prototype.html` — 静态原型 V2
- [x] 产出 delta → `deltas/test/core-PA-layout-test-cases.md` — Phase A 布局 UT/ST 用例

## [code] 代码实现 — merge 后执行（非 Phase A 范围）

- [x] `frontend-rs/src/editor_panels.rs` — AppRoot 布局重构（AppBar / ToolRail / Inspector）
- [x] `frontend-rs/src/styles.css` — 布局栅格、Tool Rail、Inspector 抽屉样式
- [x] `frontend-rs/src/editor_render.rs` — 画布空白双击/取消选中回调
- [x] `frontend-rs/src/editor_panels.rs` — 空白引导卡片 EmptyGuide
- [x] 更新 `data-testid` 锚点并对齐 `core-PA-layout-test-cases.md`
- [x] 更新 Playwright smoke 脚本（HP-01~HP-05 适配 Phase A）
