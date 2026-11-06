# 合并指令

## 变更提案
- 提案名称：fix-enter-diagram-fit-content
- 提案目录：logos/changes/fix-enter-diagram-fit-content/

## 提案内容

# 变更提案：fix-enter-diagram-fit-content（进入已有图自动适应内容 + 适应画布按钮）

## 变更原因

用户反馈（截图）：登录后进入房间 `dspm-ent-full-er`（rev 644，长期编辑/导入迁移图），默认画布为空点阵——表在视口之外，需手动缩放平移才能找到内容。

根因（frontend-rs/src/）：
1. `canvas_transform` 在 AppRoot 级创建一次，初始 `Transform::default()`（pan 0,0 / zoom 1.0，editor_panels.rs:11650），进图时既不重置也不定位；
2. 三个图加载挂点（房间 `on_enter_room` ~12176、个人建模 `on_open_personal_diagram` ~12231、share/冷启动 ~12429）`store.load()` 成功后均无任何视口定位；
3. `focus_table` 仅在新建表/搜索跳转/palette 等显式交互触发，无「加载完成 → 适应内容」路径。

原型 parity 缺口：主原型 `core-01-editor-prototype.html:420` 画布工具栏有「适应画布」（fit-canvas）按钮，生产端悬浮控件（−/+/100%/注释/维度/标签）缺失该入口。

## 变更类型

代码级（含测试用例 delta；纯前端 wasm，无 API/DB/后端变更）

## 变更范围

- 影响的需求文档：无
- 影响的功能规格：无（行为对齐既有主原型 fit-canvas 能力）
- 影响的业务场景：S04（进房加载）、S04.1A（个人建模加载）、S02 share 只读冷启动；画布交互 UT 归属 core-CR-canvas
- 影响的部署方案：无
- 影响的 API：无
- 影响的 DB 表：无
- 影响的编排测试：无
- 影响的 smoke 测试：无
- 影响的测试用例：`core-CR-canvas-test-cases.md`（ADDED UT-CR-FOCUS-03/04）、`core-S04-test-cases.md`（ADDED ST-S04-UI-19/20）

## 部署影响

- 是否需要部署：否
- 部署原因：纯前端行为修复，随常规前端构建发布，无独立部署动作
- 影响环境：无
- 是否涉及数据迁移：否
- 是否需要回滚预案：否
- 是否需要 smoke：否

## 变更概述

**A（bug 修复）— 加载已有图后自动 fit-to-content**：
- `editor_render.rs` 新增纯函数 `content_bounds(store快照)`（tables/notes/areas 世界坐标包围盒，表尺寸复用 `compute_table_render_size`）与 `fit_content_transform(bounds, viewport_w, viewport_h)`：内容整体缩放平移至视口内居中，zoom clamp `[ZOOM_MIN, 1.0]`（小图不放大超过 100%），留 10% padding；空内容返回 `None`（保持现状）。
- 三个加载挂点 `store.load()` 成功后：双 rAF（#67 先例）等画布容器挂载 → `resolve_canvas_viewport` 读真实容器尺寸 → 计算 fit → `transform.set` + `request_canvas_repaint`。仅当「本次加载的图非空」时执行，空图保持原点。

**B（原型 parity）— 悬浮控件加「适应画布」按钮**：
- `FloatingControls` 新增 `btn-fit-canvas`（ghost small 风格，文案「适应画布」，对齐原型 fit-canvas），点击对当前内容包围盒即时 fit（复用 A 的纯函数），随时手动找回内容；与 zoom_reset（回 100%）语义区分。

测试：UT-CR-FOCUS-03（fit 纯函数三态：远处图/近处图/空图）、UT-CR-FOCUS-04（三挂点 + 按钮锚点断言）；ST-S04-UI-19（e2e：mock 远坐标表进房后无需平移即可见）、ST-S04-UI-20（e2e：btn-fit-canvas 点击后远处表可见）；verify 全量回归。


## 需要合并的 Delta 文件

### 1. deltas/test/core-CR-canvas-test-cases.md

- Delta 文件：`logos/changes/fix-enter-diagram-fit-content/deltas/test/core-CR-canvas-test-cases.md`
- 目标目录：`logos/resources/test/`
- 操作：读取 delta 中的 ADDED / MODIFIED / REMOVED 标记，合并到目标目录中对应的主文档

### 2. deltas/test/core-S04-test-cases.md

- Delta 文件：`logos/changes/fix-enter-diagram-fit-content/deltas/test/core-S04-test-cases.md`
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
   git add -A && git commit -m "docs(fix-enter-diagram-fit-content): merge spec deltas"
   然后提示用户：按更新后的规格实现代码，代码完成后运行 `openlogos verify` 验收，验收通过后明确授权执行 `openlogos archive fix-enter-diagram-fit-content`。
