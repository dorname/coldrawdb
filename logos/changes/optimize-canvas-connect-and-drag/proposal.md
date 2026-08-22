# 变更提案：optimize-canvas-connect-and-drag

> module: core | created: 2026-08-22

## 变更原因

现行画布关系创建与拖表跟线和规格不一致，实际体验卡顿：

1. **只能点字段、无法直接连线**：`core-01b-relationship.md` §3 写明创建手势是「拖拽表 A 的字段 → 表 B 的字段」，但 §3.1 状态机与主原型、生产前端都只实现了「点击源字段 → 点击目标字段」，没有字段按下后的橡皮筋出线。
2. **拖表时连线掉帧**：规格要求「起点 / 终点移动时实时重算」。主原型 `pointermove` 只改表的 `left/top`，SVG 连线等到 `pointerup` 才 `render()`，过程中还按 12px 网格量化，线会一格一格跳。生产端 `editor_render.rs` 注释写了 `requestAnimationFrame`，实际每次 `mousemove` 都 `store.tables.set(整表数组)` 触发全量重绘，且未 `setPointerCapture`，光标离开 canvas 会丢事件。

本提案收口这两项：拖字段出线为主、点击两点保留；拖表时连线每帧跟随，网格只在松手时对齐。需求语义、API、DB 不变。

## 变更类型

设计级变更（交互手势 + 原型 + 测试用例 + 生产前端实现）。

## 变更范围

- 影响的需求文档：无（不改 FR/NFR 语义；NFR 60fps 画布预算已在架构中，本次落实）
- 影响的功能规格：
  - `logos/resources/prd/2-product-design/1-feature-specs/core-01b-relationship.md`（§3 / §3.1 状态机：拖拽出线为主、点击两点为辅；橡皮筋预览；松手进入确认条）
  - `logos/resources/prd/2-product-design/1-feature-specs/core-01-editor-canvas.md`（§3 手势：字段拖出连线；拖对象时连线 rAF 跟随；网格仅 pointerup 对齐）
  - `logos/resources/prd/2-product-design/1-feature-specs/core-01a-table-and-field.md`（§1.2 移动：拖动中实时坐标 + 连线跟随，松手再网格对齐）
  - `logos/resources/prd/2-product-design/2-page-design/core-01-editor-prototype.html`（唯一现行主原型：橡皮筋、拖表跟线、松手对齐）
- 影响的业务场景：S01 编辑并保存图表（画布建模手势）；S04/S05 仅回归 Viewer 只读下关系工具仍 disabled
- 影响的部署方案：无
- 影响的 API：无
- 影响的 DB 表：无
- 影响的编排测试：无（不改 HTTP/WS 契约）
- 影响的测试用例：
  - `logos/resources/test/core-PB-relationship-test-cases.md`（新增拖字段出线 UT/ST；保留 ST-PB-01 点击两点）
  - `logos/resources/test/core-CR-canvas-test-cases.md`（新增拖表过程中连线路径每帧更新、松手网格对齐）
  - `logos/resources/test/core-PU-unified-prototype-test-cases.md`（ST-PU-06 保留点击；新增拖出线与拖表跟线用例）
- 影响的 smoke 测试：无
- 影响的代码：
  - `frontend-rs/src/editor_render.rs`（pointer 捕获、橡皮筋、rAF 合并绘制、拖动中临时坐标、松手写入 store + 网格对齐）
  - `frontend-rs/src/editor_panels.rs`（`RelToolState` 增加 Dragging；点击/拖拽阈值分流；确认条不变）
  - `frontend-rs/tests/phase_b_relationship.rs`、`frontend-rs/tests/e2e/16_relationship_tool.spec.ts`、`frontend-rs/scripts/test-unified-prototype.mjs`

S03/S04/S05 历史独立原型（`core-03`/`core-04`/`core-05-*-prototype.html`）不改。

## 部署影响

- 是否需要部署：否
- 部署原因：仅编辑器画布手势与渲染节流，不改 API、DB、路由或发布拓扑；本地 `trunk` 热更新即可验收。模块级 `deployment_required` 不覆盖本提案明确决策。
- 影响环境：无
- 是否涉及数据迁移：否
- 是否需要回滚预案：否
- 是否需要 smoke：否

## UI/UX 变更声明

```yaml
ui_impact: true
design_system_mode: generated
design_system_fallback_reason: ""
pages:
  - id: editor
    prototype: core-01-editor-prototype.html
    description: 关系工具支持从字段拖出橡皮筋连到目标字段（点击两点保留）；拖表时关系线每帧跟随，网格仅松手对齐
```

## 变更概述

关系工具激活后，**主要手势**是在字段上 pointerdown，拖出一条跟随指针的贝塞尔预览线（`data-testid="rel-rubber-band"`），在目标字段 pointerup 后进入现有确认条（cardinality 默认 `one_to_many`，点「创建」才写入）。位移小于阈值（建议 4px）的 pointerdown/up 视为点击，仍走「源字段 → 目标字段 → 确认条」。Esc / 切回选择工具 / 松在空白处取消当前拖线，回到 `PickSource`。

拖动表（及后续若拖 Note/Area）时：移动过程使用未量化的视觉坐标，**每一动画帧**重算并重绘相关连线；`GRID_SIZE`（原型 12px，生产端与现有网格一致，当前为 20px）只在 pointerup 时对齐并提交 undo。生产端用 `requestAnimationFrame` 合并绘制，拖动中不把整表数组 `set` 进 store；`setPointerCapture` 保证指针不丢失。
