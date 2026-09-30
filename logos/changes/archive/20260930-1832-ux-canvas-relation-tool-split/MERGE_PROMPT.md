# 合并指令

## 变更提案
- 提案名称：ux-canvas-relation-tool-split
- 提案目录：logos/changes/ux-canvas-relation-tool-split/

## 提案内容

# 变更提案：ux-canvas-relation-tool-split

> module: core | created: 2026-09-30

## 变更原因

用户目标反馈：
1. 鼠标选中关系线位置时“所选即所得”体验不足；
2. 当前 ToolRail 只有一个“关系工具”按钮，同时承担“拖拽创建关系”与“点击选中关系”两种能力，容易造成误操作（例如在创建关系模式下想选中已有关系线，或在选择模式下误触发拖线）。

本提案将关系工具的**创建**与**选中**能力拆分为左侧 ToolRail 两个独立图标，用户通过菜单显式切换，从而提升画布操作的确定性与“所见即所得”感。

## 变更类型

设计级变更（ToolRail 交互入口拆分 + 关系工具状态机扩展 + 对应规格/测试更新）

## 变更范围

- 影响的需求文档：无
- 影响的功能规格：
  - `logos/resources/prd/2-product-design/1-feature-specs/core-01b-relationship.md` §3.1 关系工具模式拆分
  - `logos/resources/prd/2-product-design/1-feature-specs/core-04-side-panel-tabs.md` §1 Tool Rail 按钮清单新增两个关系按钮
  - `logos/resources/prd/2-product-design/1-feature-specs/core-08-icon-library.md` 新增关系创建 / 关系选中图标
- 影响的业务场景：S01 编辑器画布 / 关系创建与选中
- 影响的 API：无
- 影响的 DB 表：无
- 影响的编排测试：无
- 影响的测试用例：
  - `logos/resources/test/core-PB-relationship-test-cases.md` 新增 UT-PB-27 / ST-PB-16
  - `logos/resources/test/core-PA-layout-test-cases.md` 或对应 ToolRail 验收用例更新图标数量

## 部署影响

- 是否需要部署：否
- 部署原因：纯前端交互调整，不涉 API、DB、配置或数据迁移；前端重新 build 后生效
- 影响环境：无
- 是否涉及数据迁移：否
- 是否需要回滚预案：否
- 是否需要 smoke：否

## 变更概述

1. **ToolRail 拆分为两个关系按钮**：
   - `tool-relationship-create`（创建关系）：保留现有 `ActiveTool::Relationship` 行为，支持从字段拖出连线或点击两点创建关系。
   - `tool-relationship-select`（选中关系）：新增 `ActiveTool::RelationshipSelect`，进入后鼠标变为选择态，点击画布上可见的关系线即可选中（命中逻辑与 `Select` 工具下的关系线命中同口径，即“所选即所得”），不触发创建关系手势。
2. **状态机扩展**：`RelToolState` 保持 `PickSource / PickTarget / Dragging` 用于创建模式；`RelationshipSelect` 模式下保持 `RelToolState::Idle`，直接复用 `hit_test_reference_tier` 进行关系线命中并写入 `selection`。
3. **图标与提示**：在 `core-08-icon-library.md` 新增 `IconRelationshipCreate` / `IconRelationshipSelect` 两个图标（V1 可用现有 `IconRelationship` + 区分样式或新增 SVG 占位，后续 icon-refresh 变更再统一替换）；ToolRail tooltip 分别显示“创建关系 R”与“选中关系”。
4. **规格与测试同步**：更新 `core-01b-relationship.md` 与 `core-04-side-panel-tabs.md`；新增 UT-PB-27（ActiveTool 切换与选择态）和 ST-PB-16（e2e：切换选中关系工具后点击关系线选中）。


## 需要合并的 Delta 文件

### 1. deltas/prd/2-product-design/1-feature-specs/core-01b-relationship.md

- Delta 文件：`logos/changes/ux-canvas-relation-tool-split/deltas/prd/2-product-design/1-feature-specs/core-01b-relationship.md`
- 目标目录：`logos/resources/prd/2-product-design/1-feature-specs/`
- 操作：读取 delta 中的 ADDED / MODIFIED / REMOVED 标记，合并到目标目录中对应的主文档

### 2. deltas/prd/2-product-design/1-feature-specs/core-04-side-panel-tabs.md

- Delta 文件：`logos/changes/ux-canvas-relation-tool-split/deltas/prd/2-product-design/1-feature-specs/core-04-side-panel-tabs.md`
- 目标目录：`logos/resources/prd/2-product-design/1-feature-specs/`
- 操作：读取 delta 中的 ADDED / MODIFIED / REMOVED 标记，合并到目标目录中对应的主文档

### 3. deltas/prd/2-product-design/1-feature-specs/core-08-icon-library.md

- Delta 文件：`logos/changes/ux-canvas-relation-tool-split/deltas/prd/2-product-design/1-feature-specs/core-08-icon-library.md`
- 目标目录：`logos/resources/prd/2-product-design/1-feature-specs/`
- 操作：读取 delta 中的 ADDED / MODIFIED / REMOVED 标记，合并到目标目录中对应的主文档

### 4. deltas/test/core-PB-relationship-test-cases.md

- Delta 文件：`logos/changes/ux-canvas-relation-tool-split/deltas/test/core-PB-relationship-test-cases.md`
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
   git add -A && git commit -m "docs(ux-canvas-relation-tool-split): merge spec deltas"
   然后提示用户：按更新后的规格实现代码，代码完成后运行 `openlogos verify` 验收，验收通过后明确授权执行 `openlogos archive ux-canvas-relation-tool-split`。
