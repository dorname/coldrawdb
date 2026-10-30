# 合并指令

## 变更提案
- 提案名称：fix-dense-relation-hit-discoverability
- 提案目录：logos/changes/fix-dense-relation-hit-discoverability/

## 提案内容

# 变更提案：提升密集关系叠线轮选可发现性

> module: core | created: 2026-10-31 | issue: #60

## 变更原因

Issue #60 在 16 表 / 19 关系模型中观察到部分关系无法通过单次点击扫描命中，且重叠区域存在选择歧义。代码已具备 R-HIT-08 的 3px 叠线簇重复点击轮选，完全重合路径也能按 refs 顺序逐条环回；实际缺口是界面没有告知用户该交互，且测试只覆盖人造距离数组，没有覆盖完全重合路径和大图真实点击。

本次采用最小修复，不调整关系几何、不引入候选弹窗：在现有关系悬停摘要中补充叠线轮选提示，并用完全重合路径与 16 表 / 19 关系回归证明每条关系均可通过重复点击到达。

## 变更类型

设计级变更（既有叠线轮选交互的可发现性与回归覆盖修复）。

## 变更范围

- 影响的功能规格：`core-01b-relationship.md` §4.7 R-HIT-08、§4.8 R-HOV-02。
- 影响的业务场景：S01 画布关系浏览与选择。
- 影响的 API：无。
- 影响的 DB 表：无。
- 影响的测试：`core-PB-relationship-test-cases.md` UT-PB-20 文案预期，新增 UT-PB-33、ST-PB-17。
- 不影响：关系路由、命中带宽、候选排序、轮选算法、选中/悬停状态结构、数据持久化。
- 主原型无关系悬停摘要组件，本次无原型变更。

## 部署影响

- 是否需要部署：否。
- 部署原因：本地 Leptos 前端提示和测试，无部署或数据迁移。
- 影响环境：无。
- 是否涉及数据迁移：否。
- 是否需要回滚预案：否。
- 是否需要 smoke：否。

## 变更概述

关系悬停摘要在现有端点文案后追加“若线条重叠，重复点击可切换关系”。保持 R-HIT-08 既有轮选语义不变，让完全重合关系的选择方式可被发现。

新增 UT-PB-33 验证详情/拓扑两种档位下完全重合路径可逐条环回；新增 ST-PB-17 在 16 表 / 19 关系模型中验证提示可见、同一坐标重复点击能选中全部四条完全重合关系，且过程不产生落账。


## 需要合并的 Delta 文件

### 1. deltas/prd/2-product-design/1-feature-specs/core-01b-relationship.md

- Delta 文件：`logos/changes/fix-dense-relation-hit-discoverability/deltas/prd/2-product-design/1-feature-specs/core-01b-relationship.md`
- 目标目录：`logos/resources/prd/2-product-design/1-feature-specs/`
- 操作：读取 delta 中的 ADDED / MODIFIED / REMOVED 标记，合并到目标目录中对应的主文档

### 2. deltas/test/core-PB-relationship-test-cases.md

- Delta 文件：`logos/changes/fix-dense-relation-hit-discoverability/deltas/test/core-PB-relationship-test-cases.md`
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
   git add -A && git commit -m "docs(fix-dense-relation-hit-discoverability): merge spec deltas"
   然后提示用户：按更新后的规格实现代码，代码完成后运行 `openlogos verify` 验收，验收通过后明确授权执行 `openlogos archive fix-dense-relation-hit-discoverability`。
