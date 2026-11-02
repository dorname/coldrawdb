# 合并指令

## 变更提案
- 提案名称：fix-big-diagram-table-drag-blocked
- 提案目录：logos/changes/fix-big-diagram-table-drag-blocked/

## 提案内容

# 变更提案：修复大图密集关系下表拖拽被阻断

> module: core | created: 2026-11-02
> 关联 issue：#74（54 表 / 97 关系下关系抢占表点击、悬浮控件遮挡画布）

## 变更原因

Issue #74 在 54 表 / 97 关系的真实图表中观察到：默认选择、框选等工具下点击表头被关系选中，表拖动 0/8 成功；悬浮控件条的非按钮区域也会截获画布 pointerdown。代码核查定位到三项实现问题：

1. 可编辑默认路径在表 AABB 与关系同时命中时，只要关系进入自适应命中带宽就无条件优先关系，未区分用户是否点在表头/表体；选中关系工具则在关系命中后提前返回。
2. 正式关系虽然由 Canvas 按当前 `zoom` 与 `label_font_scale` 绘制，但 `dist_to_reference` 与 `reference_aabb` 的路径计算硬编码为 `zoom=1.0、label_scale=1.0`。低缩放或放大画布字号后，命中几何与可见几何可能漂移，不可见的关系热区仍会参与点击与悬停。
3. `.cdb-floating-controls` 未收窄指针事件，容器背景、padding、分隔和文本区域均按 `auto` 截获事件，落在其下的表无法接收 pointerdown。

同时纠正 issue 中一项测量口径：`svg.cdb-rel-overlay` 仅包含 `rel-follow-path` 与 `rel-rubber-band` 两条建关系临时路径，正式关系由 `draw_canvas` 绘制到 Canvas。SVG 只有 1 条有效 `d` 不能证明正式关系只渲染 1/97，本次不引入 97 个 SVG 节点，也不据此合并关闭 #60。

## 变更类型

设计级变更（修订表与关系的命中优先级合同，并补齐同源几何与回归覆盖）。

## 变更范围

- 影响的需求文档：无（不改变 S01 产品目标）。
- 影响的功能规格：
  - `core-01b-relationship.md` §4.7 R-HIT-01/R-HIT-05：有效字号几何同源；可编辑默认/框选工具表优先，选中关系工具关系优先，只读保留既有近线优先。
  - `core-01-editor-canvas.md` §5.18 R-SEL-UNIFY-02/03/04/08 与新增 §5.19：补充密集关系下的工具优先级、直接拖表及悬浮控件指针边界口径。
  - 悬浮控件不新增视觉组件，也不改变 #69 已修复的布局与层级，仅补充既有“按钮可操作、画布手势不被非按钮区域阻挡”的交互规格。
- 影响的业务场景：S01 编辑并保存图表（画布选择、拖动、关系浏览）。
- 影响的测试：`core-PB-relationship-test-cases.md`，修订 UT-PB-28/31 的适用范围，新增 UT-PB-37 与 ST-PB-21。
- 影响的代码：
  - `frontend-rs/src/editor_render.rs`：可编辑路径表优先；关系绘制/命中使用相同 `zoom`、`label_font_scale`；保留现有默认参数包装函数以兼容既有纯函数调用。
  - `frontend-rs/src/styles.css`：悬浮条容器 `pointer-events:none`，实际按钮恢复 `pointer-events:auto`。
  - `frontend-rs/scripts/test-spec-parity-d.mjs`：54 表 / 97 关系真实指针回归。
  - OpenLogos reporter 及对应测试登记。
- 影响的主原型：无（不新增界面或视觉状态，现行主原型无需承载该 Canvas 内部命中细节）。
- 影响的场景时序：无外部参与方或 API 编排变化，不修改 S01 时序图。
- 影响的 API：无。
- 影响的 DB 表：无。
- 影响的编排测试：无。
- 明确不做：关系路由重排、候选弹窗、叠线轮选算法调整、正式关系 SVG 化、#60 的独立可达性结论与关闭。

## 兼容性影响

可编辑默认选择 / 框选工具的冲突优先级将由 #51 的“近线优先于表”调整为“表 AABB 内优先表”，这是为恢复直接拖表而作出的有意行为变更，不是对所有路径统一反转优先级。#51 所需的关系查看能力由两条既有路径保留：可编辑模式使用显式「选中关系」工具，只读分享 / Viewer 继续近线优先。#63 的统一回落、R-HIT-08 叠线轮选、命中带宽、数据格式与持久化协议均保持兼容。

## 部署影响

- 是否需要部署：否。
- 部署原因：本地 Leptos 前端交互与样式修复，随下次常规发布携带；本提案仅负责实现与验收。
- 影响环境：无。
- 是否涉及数据迁移：否。
- 是否需要回滚预案：否。
- 是否需要 smoke：否。

## 变更概述

可编辑默认选择与框选工具中，只要指针命中表头/表体，就优先进入既有表选择与拖动手势；只有未命中任何表时才在空白处选择近线关系。选中关系工具继续优先关系，未命中关系时回落表/空白路由，从而保留 #51 和 #63 已交付的关系选择能力。只读模式没有拖表写入风险，继续沿用既有近线优先，避免只读分享重新出现压在表上的关系无法查看的问题。

关系命中与 AABB 预过滤增加接收当前 `zoom` 和 `label_font_scale` 的完整参数路径，点击、悬停、只读与选中关系工具统一使用，与 Canvas 正式绘制消费同一组有效几何。悬浮条仅在按钮上接收指针，非按钮区域穿透到画布。

新增 UT-PB-37 验证三种线型在低缩放/放大字号下的绘制与命中同源；新增 ST-PB-21 用 54 表 / 97 关系验证默认工具直接拖表、显式关系选择不回退，以及悬浮条空白穿透和按钮可点。既有 UT-PB-28/31 保留只读近线优先含义，ST-PB-16/17/20 继续覆盖关系选择、叠线轮选与统一路由。


## 需要合并的 Delta 文件

### 1. deltas/prd/2-product-design/1-feature-specs/core-01-editor-canvas.md

- Delta 文件：`logos/changes/fix-big-diagram-table-drag-blocked/deltas/prd/2-product-design/1-feature-specs/core-01-editor-canvas.md`
- 目标目录：`logos/resources/prd/2-product-design/1-feature-specs/`
- 操作：读取 delta 中的 ADDED / MODIFIED / REMOVED 标记，合并到目标目录中对应的主文档

### 2. deltas/prd/2-product-design/1-feature-specs/core-01b-relationship.md

- Delta 文件：`logos/changes/fix-big-diagram-table-drag-blocked/deltas/prd/2-product-design/1-feature-specs/core-01b-relationship.md`
- 目标目录：`logos/resources/prd/2-product-design/1-feature-specs/`
- 操作：读取 delta 中的 ADDED / MODIFIED / REMOVED 标记，合并到目标目录中对应的主文档

### 3. deltas/test/core-PB-relationship-test-cases.md

- Delta 文件：`logos/changes/fix-big-diagram-table-drag-blocked/deltas/test/core-PB-relationship-test-cases.md`
- 目标目录：`logos/resources/test/`
- 操作：读取 delta 中的 ADDED / MODIFIED / REMOVED 标记，合并到目标目录中对应的主文档

## 执行要求

1. 逐个 Delta 文件处理，每处理完一个报告修改摘要
2. 对于 ADDED 标记：在主文档的指定位置插入新内容
3. 对于 MODIFIED 标记：替换主文档中同名章节的内容
4. 对于 REMOVED 标记：从主文档中删除对应章节
5. 保持主文档的原有格式和风格
6. 如果主文档有"最后更新"时间戳，同步更新
7. 所有变更完成后，列出修改清单
8. 所有变更合并完成后，自动执行 git commit（告知用户，无需确认）：
   git add -A && git commit -m "docs(fix-big-diagram-table-drag-blocked): merge spec deltas"
   然后提示用户：按更新后的规格实现代码，代码完成后运行 `openlogos verify` 验收，验收通过后明确授权执行 `openlogos archive fix-big-diagram-table-drag-blocked`。
