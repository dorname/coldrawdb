# 合并指令

## 变更提案
- 提案名称：fix-related-relation-identification
- 提案目录：logos/changes/fix-related-relation-identification/

## 提案内容

# 变更提案：完整的关系标签与识别体验

> module: core | created: 2026-10-31 | issue: #65, #66

## 变更原因

Issue #65 指出画布关系线缺少可读语义，Issue #66 进一步指出：即使选中表后相关线已加粗，用户仍无法知道每条高亮关系的关系名、字段对、基数和级联策略，只能逐条点线确认。

本次不以“最小修复”为目标，而是建立完整且不过载的关系识别体验：悬停摘要提供完整定义，选中表/关系时自动出现上下文标签，检查器提供可点击明细，并支持评审场景下一键显示全部关系标签。密集画布通过优先级和碰撞避让保持可读性。

## 变更类型

设计级变更（画布关系标签、悬停信息、表检查器信息架构与定位交互）。

## 变更范围

- 影响的功能规格：`core-01b-relationship.md` §4.8 悬停摘要，新增表关系明细与画布关系标签合同。
- 影响的业务场景：S01 画布关系浏览、识别、评审与选择。
- 影响的 API：无。
- 影响的 DB 表：无。
- 影响的测试：`core-PB-relationship-test-cases.md` 新增 UT-PB-34、UT-PB-35、ST-PB-18、ST-PB-19，并同步 ST-PB-12 的悬停内容预期。
- 不影响：关系数据模型、路由与命中规则、叠线轮选、持久化数据结构、撤销栈和协作协议。
- 主原型无对应关系标签与表关系明细组件，本次无 HTML 原型变更；生产 Leptos 画布与检查器以本规格为验收依据。

## 部署影响

- 是否需要部署：否。
- 部署原因：本地 Leptos 前端展示与交互优化，无部署或数据迁移。
- 影响环境：无。
- 是否涉及数据迁移：否。
- 是否需要回滚预案：否。
- 是否需要 smoke：否。

## 变更概述

关系悬停摘要升级为关系名、完整字段对、显示化基数、`ON DELETE` 和既有轮选提示。表检查器新增入边、出边、自环关系明细，点击条目复用既有关系选中通路，定位到关系检查器与画布高亮，不移动画布、不改数据。

画布在选中表/关系时自动绘制相关上下文标签；浮动工具条新增“关系标签”开关，可在评审时展开全部标签，偏好保存于本机 `localStorage`，不落库、不进撤销栈、不同步协作。标签包含关系名、基数和字段对，使用屏幕可读字号、半透明衬底、候选位置与碰撞检测；选中关系优先于相关关系，相关关系优先于普通关系，普通标签空间不足时省略，完整信息仍可由悬停和检查器获得。

新增 UT-PB-34/35 覆盖明细视图模型、标签内容/显隐与碰撞规则；ST-PB-18 验证 16 表 / 19 关系中 `orders` 的 6 条明细及定位，ST-PB-19 验证完整悬停、上下文标签、全局开关、刷新持久化和无写入副作用。


## 需要合并的 Delta 文件

### 1. deltas/prd/2-product-design/1-feature-specs/core-01b-relationship.md

- Delta 文件：`logos/changes/fix-related-relation-identification/deltas/prd/2-product-design/1-feature-specs/core-01b-relationship.md`
- 目标目录：`logos/resources/prd/2-product-design/1-feature-specs/`
- 操作：读取 delta 中的 ADDED / MODIFIED / REMOVED 标记，合并到目标目录中对应的主文档

### 2. deltas/test/core-PB-relationship-test-cases.md

- Delta 文件：`logos/changes/fix-related-relation-identification/deltas/test/core-PB-relationship-test-cases.md`
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
   git add -A && git commit -m "docs(fix-related-relation-identification): merge spec deltas"
   然后提示用户：按更新后的规格实现代码，代码完成后运行 `openlogos verify` 验收，验收通过后明确授权执行 `openlogos archive fix-related-relation-identification`。
