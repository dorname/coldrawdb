# 合并指令

## 变更提案
- 提案名称：feat-listview-relation-details
- 提案目录：logos/changes/feat-listview-relation-details/

## 提案内容

# 变更提案：feat-listview-relation-details

> module: core | created: 2026-10-31

## 变更原因
GitHub Issue #61：在 16 张表 / 19 条关系的电商模型中，列表视图左树只有语义模糊的 `⇄`，且存在关联表漏标；右侧仅显示关联表名和整体基数，缺少关系名、字段对、方向化基数及 ON DELETE / ON UPDATE，用户仍需回画布逐条核对。现有搜索也不能按关系定位表。

## 变更类型
设计级（既有 S01 列表视图的关系浏览与搜索体验完善，不改变数据模型和持久化合同）。

## 变更范围
- 影响的需求文档：无新增需求；沿用 S01 图表编辑与列表视图既有验收目标
- 影响的功能规格：`core-04-side-panel-tabs.md` §10.5、§11
- 影响的业务场景：S01（列表视图只读导航，不新增场景编号）
- 影响的部署方案：无
- 影响的 API：无
- 影响的 DB 表：无
- 影响的编排测试：无（非 API 编排变更）
- 影响的测试用例：`core-SP-side-panel-test-cases.md`，修订 UT/ST-SP-LIST-REL-01，新增 UT/ST-SP-LIST-REL-02
- 影响的 smoke 测试：无

## 部署影响
- 是否需要部署：否
- 部署原因：仅前端列表视图展示、搜索与测试变更，由常规验收覆盖
- 影响环境：无
- 是否涉及数据迁移：否
- 是否需要回滚预案：否
- 是否需要 smoke：否

## 变更概述
左侧表树为每张表独立计算关系方向，明确显示入边、出边或双向，自环同时计入入边和出边；关联高亮继续用于表达与当前选中表的关系，不再承担方向语义。右侧面板按 reference 逐条展示关系名、完整字段对、端点基数、ON DELETE 和 ON UPDATE，同源/同目标多关系不合并，并保留点击对端表导航。

复用列表视图现有搜索框，在表名、字段名和类型匹配之外增加关系名及完整字段对匹配；关系命中时两端表均可见并显示命中原因。搜索、树标记和关系导航均为纯视图状态，不写 diagram、不进撤销栈、不产生协作操作。画布关系标签已由既有 #65/#66 变更覆盖，本提案不重复实现。


## 需要合并的 Delta 文件

### 1. deltas/prd/2-product-design/1-feature-specs/core-04-side-panel-tabs.md

- Delta 文件：`logos/changes/feat-listview-relation-details/deltas/prd/2-product-design/1-feature-specs/core-04-side-panel-tabs.md`
- 目标目录：`logos/resources/prd/2-product-design/1-feature-specs/`
- 操作：读取 delta 中的 ADDED / MODIFIED / REMOVED 标记，合并到目标目录中对应的主文档

### 2. deltas/test/core-SP-side-panel-test-cases.md

- Delta 文件：`logos/changes/feat-listview-relation-details/deltas/test/core-SP-side-panel-test-cases.md`
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
   git add -A && git commit -m "docs(feat-listview-relation-details): merge spec deltas"
   然后提示用户：按更新后的规格实现代码，代码完成后运行 `openlogos verify` 验收，验收通过后明确授权执行 `openlogos archive feat-listview-relation-details`。
