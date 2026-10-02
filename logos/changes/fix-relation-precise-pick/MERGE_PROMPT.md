# 合并指令

## 变更提案
- 提案名称：fix-relation-precise-pick
- 提案目录：logos/changes/fix-relation-precise-pick/

## 提案内容

# 变更提案：fix-relation-precise-pick

> module: core | created: 2026-10-02 | follow-up of #51 / fix-dense-lowzoom-relation-hit

## 变更原因

DSPM 等密集图复测：默认选择与「选中关系」工具均**难以精准选中目标关系线**——常点到邻近线或感觉“粘手”。

根因：

1. 低缩放命中带宽放宽至 **20px** 后，多线同时落入带宽；仅「最近距离」在近距并列时易选错。
2. 仿真（42 线）：zoom=0.1 时约 **24/42** 的中点处，最近与次近线屏幕距离差 &lt; 2px（不可消歧）。
3. 两工具共用 `hit_test_reference_tier`，问题不在工具分流而在命中消歧。

目标：**精准选中**——点在意图线上时选中该线；叠线区支持再次点击轮选。

## 变更类型

设计级变更（命中合同修订 + 前端实现 + 测试）

## 变更范围

- 影响的需求文档：无
- 影响的功能规格：`core-01b-relationship.md` §4.7
  - 收紧 `REL_LINE_HIT_PX_LOW`：20→**14**
  - 新增 R-HIT-08：近距叠线簇内点击轮选（cluster）
- 影响的业务场景：S01 画布关系选中
- 影响的 API / DB / 编排 / smoke：无
- 影响的测试用例：`core-PB-relationship-test-cases.md`
  - 修订 UT-PB-30/31（低缩放 14px 口径）
  - 新增 UT-PB-32（叠线簇轮选）

## 部署影响

- 是否需要部署：否
- 部署原因：纯前端命中逻辑
- 影响环境：无
- 是否涉及数据迁移：否
- 是否需要回滚预案：否
- 是否需要 smoke：否

## 变更概述

1. 低缩放主线带宽 20→14 屏幕像素，降低“粘连”误选。
2. 带宽内按屏幕距离排序；与最近线相差 ≤ `REL_HIT_CLUSTER_PX`（3px）的候选构成叠线簇；若当前已选中簇内某线，再次点击轮到下一根，否则取最近。
3. 悬停仍不轮选（`prev=None`）；默认工具 / 选中关系工具 / 近线优先共用同一消歧。


## 需要合并的 Delta 文件

### 1. deltas/prd/2-product-design/1-feature-specs/core-01b-relationship.md

- Delta 文件：`logos/changes/fix-relation-precise-pick/deltas/prd/2-product-design/1-feature-specs/core-01b-relationship.md`
- 目标目录：`logos/resources/prd/2-product-design/1-feature-specs/`
- 操作：读取 delta 中的 ADDED / MODIFIED / REMOVED 标记，合并到目标目录中对应的主文档

### 2. deltas/test/core-PB-relationship-test-cases.md

- Delta 文件：`logos/changes/fix-relation-precise-pick/deltas/test/core-PB-relationship-test-cases.md`
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
   git add -A && git commit -m "docs(fix-relation-precise-pick): merge spec deltas"
   然后提示用户：按更新后的规格实现代码，代码完成后运行 `openlogos verify` 验收，验收通过后明确授权执行 `openlogos archive fix-relation-precise-pick`。
