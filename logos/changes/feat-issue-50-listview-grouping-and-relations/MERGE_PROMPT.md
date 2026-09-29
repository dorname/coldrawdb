# 合并指令

## 变更提案
- 提案名称：feat-issue-50-listview-grouping-and-relations
- 提案目录：logos/changes/feat-issue-50-listview-grouping-and-relations/

## 提案内容

# 变更提案：feat-issue-50-listview-grouping-and-relations

> module: core | created: 2026-09-29

## 变更原因

issue #50 反馈列表视图在表数量较多时左侧表树为扁平列表，缺少分组能力；同时选中某张表时无法直观看到与其关联的表。需要在 V1 边界内以最小可行方案增强列表视图的可浏览性与关系感知。

## 变更类型

设计级变更

## 变更范围

- 影响的功能规格：`logos/resources/prd/2-product-design/1-feature-specs/core-04-side-panel-tabs.md`
  - §10.5 列表视图（ListView 表树 + 单表字段网格）
- 影响的业务场景：S01 编辑器画布 / 列表视图
- 影响的 API：无
- 影响的 DB 表：无（复用 `Table.area` / `Reference` 现有结构）
- 影响的编排测试：无
- 影响的测试用例：`logos/resources/test/core-SP-side-panel-test-cases.md`
  - 新增 UT-SP-LIST-GROUP-01：按 Area 分组纯函数
  - 新增 ST-SP-LIST-GROUP-01：列表视图按 Area 分组可折叠/搜索
  - 新增 UT-SP-LIST-REL-01：关联表集合纯函数
  - 新增 ST-SP-LIST-REL-01：选中表时右侧展示关联表清单

## 部署影响

- 是否需要部署：否
- 部署原因：纯前端 UI 调整，不涉 API、DB、配置或数据迁移
- 影响环境：无
- 是否涉及数据迁移：否
- 是否需要回滚预案：否
- 是否需要 smoke：否

## 变更概述

在 V1 边界内采用最小可行方案：

1. **按 Area 分组**：左侧表树不再扁平渲染，而是按 `Table.area`（区域）分组；无 Area 的表归入「未分组」。分组支持折叠/展开，并保留现有搜索过滤能力——搜索时自动展开含匹配表的分组。
2. **关联表可视化**：右侧当前表字段网格上方新增「关联表」摘要区，列出当前选中表的入边/出边关联表（含关系类型 1:1 / 1:N / N:M），点击关联表名可在树中定位并切换选中。树中关联表节点以高亮样式区分，非关联表弱化，帮助用户感知表间关系。

本方案不引入主题域树、多表 Tab 条、逻辑实体、自定义分组等 V1 边界外能力；也不使用弹层或遮挡字段编辑区的重型 UI。


## 需要合并的 Delta 文件

### 1. deltas/prd/2-product-design/1-feature-specs/core-04-side-panel-tabs.md

- Delta 文件：`logos/changes/feat-issue-50-listview-grouping-and-relations/deltas/prd/2-product-design/1-feature-specs/core-04-side-panel-tabs.md`
- 目标目录：`logos/resources/prd/2-product-design/1-feature-specs/`
- 操作：读取 delta 中的 ADDED / MODIFIED / REMOVED 标记，合并到目标目录中对应的主文档

### 2. deltas/test/core-SP-side-panel-test-cases.md

- Delta 文件：`logos/changes/feat-issue-50-listview-grouping-and-relations/deltas/test/core-SP-side-panel-test-cases.md`
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
   git add -A && git commit -m "docs(feat-issue-50-listview-grouping-and-relations): merge spec deltas"
   然后提示用户：按更新后的规格实现代码，代码完成后运行 `openlogos verify` 验收，验收通过后明确授权执行 `openlogos archive feat-issue-50-listview-grouping-and-relations`。
