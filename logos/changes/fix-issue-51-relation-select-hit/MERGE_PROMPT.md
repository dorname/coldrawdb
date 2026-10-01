# 合并指令

## 变更提案
- 提案名称：fix-issue-51-relation-select-hit
- 提案目录：logos/changes/fix-issue-51-relation-select-hit/

## 提案内容

# 变更提案：fix-issue-51-relation-select-hit

> module: core | created: 2026-10-01 | issue: #51

## 变更原因

人工测试（导入约 26 表 / 42 关系的资产域模型）反馈：画布上很难正常选中关系线。

根因两类：

1. **默认选择工具表抢选**：`pointerdown` 先 `hit_test_with`（表 AABB）再关系命中（R-HIT-05）。连线贴近表边缘或弧线掠过表卡时，点击视觉上的「线」仍落入表 AABB → 选中表。
2. **主线命中带宽偏窄**：主线热区仅 8 屏幕像素（R-HIT-02），缩放与密集汇聚场景下对准成本高；「选中关系」工具虽只走关系命中，仍受同一窄热区约束。

命令面板可正确选中关系 → 选中/检查器链路正常，问题在画布命中与优先级。

关联：#44 / #46 / #47（已关）与 `ux-canvas-relation-tool-split` 之后的剩余体验缺口。

## 变更类型

设计级变更（命中合同修订 + 前端命中/优先级实现 + 测试）

## 变更范围

- 影响的需求文档：无
- 影响的功能规格：`logos/resources/prd/2-product-design/1-feature-specs/core-01b-relationship.md` §4.7
  - 修订 R-HIT-02 主线带宽 8→12 屏幕像素
  - 修订 R-HIT-05：同时命中表与关系时，关系屏幕距离 ≤ 主线命中带宽（12px）优先选关系
  - R-HIT-07 AABB 外扩同步按 `max(12, REL_ENDPOINT_SIZE)` 计算
- 影响的业务场景：S01 编辑器画布 / 关系选中
- 影响的 API：无
- 影响的 DB 表：无
- 影响的编排测试：无
- 影响的测试用例：`logos/resources/test/core-PB-relationship-test-cases.md`
  - 新增 UT-PB-28（表+关系同时命中时近线优先关系）
  - 新增 UT-PB-29（主线 12px 带宽边界）
  - 修订既有 UT-PB-18/22/24 等对 8px 硬编码的断言口径（随 R-HIT-02）
- 影响的 smoke 测试：无

## 部署影响

- 是否需要部署：否
- 部署原因：纯前端命中逻辑与规格修订，不涉 API/DB/配置/数据迁移
- 影响环境：无
- 是否涉及数据迁移：否
- 是否需要回滚预案：否
- 是否需要 smoke：否

## 变更概述

1. **加宽主线命中**：引入常量 `REL_LINE_HIT_PX = 12.0`，替换硬编码 `8.0`；AABB 预过滤外扩同步。
2. **近线优先于表**：默认选择路径在「表命中 ∩ 关系命中」时，若关系最近距离（屏幕像素）≤ `REL_OVER_TABLE_PREFER_PX`（= `REL_LINE_HIT_PX` = 12），选中关系而非表；深点表体（距线超出命中带宽）仍选表。只读分享路径同步。
3. **RelationshipSelect** 继续仅走关系命中，自动享受 12px 带宽。
4. 规格与 UT 同步；不改 API/DB/部署。


## 需要合并的 Delta 文件

### 1. deltas/prd/2-product-design/1-feature-specs/core-01b-relationship.md

- Delta 文件：`logos/changes/fix-issue-51-relation-select-hit/deltas/prd/2-product-design/1-feature-specs/core-01b-relationship.md`
- 目标目录：`logos/resources/prd/2-product-design/1-feature-specs/`
- 操作：读取 delta 中的 ADDED / MODIFIED / REMOVED 标记，合并到目标目录中对应的主文档

### 2. deltas/test/core-PB-relationship-test-cases.md

- Delta 文件：`logos/changes/fix-issue-51-relation-select-hit/deltas/test/core-PB-relationship-test-cases.md`
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
   git add -A && git commit -m "docs(fix-issue-51-relation-select-hit): merge spec deltas"
   然后提示用户：按更新后的规格实现代码，代码完成后运行 `openlogos verify` 验收，验收通过后明确授权执行 `openlogos archive fix-issue-51-relation-select-hit`。
