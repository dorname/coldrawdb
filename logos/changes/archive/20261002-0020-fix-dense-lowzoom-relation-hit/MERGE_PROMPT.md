# 合并指令

## 变更提案
- 提案名称：fix-dense-lowzoom-relation-hit
- 提案目录：logos/changes/fix-dense-lowzoom-relation-hit/

## 提案内容

# 变更提案：fix-dense-lowzoom-relation-hit

> module: core | created: 2026-10-01 | follow-up of #51

## 变更原因

#51（主线 12px + 表∩关系近线优先）验收通过后，在 **DSPM** 协作空间（约 24 表 / 42 直线关系、多线汇聚 `asset_object`）全景约 **10%** 缩放下，用户仍反馈「很难点到关系线」。

根因（相对 #51 残留）：

1. **固定屏幕带宽在低缩放下对准成本仍高**：线在屏幕上极细；12px 热区对全景点选偏紧。
2. **低缩放世界阈值膨胀带来多线竞争**：`hit = 12/zoom` 世界像素，zoom=0.1 时约 120 世界像素——最近距离仍生效，但易选中「非用户意图」的邻近线，体感为难点/点错。
3. **字段维度表卡叠压**：全景下 AABB 大面积重叠，默认工具虽有近线优先，细线仍难瞄准。

命令面板 / 「选中关系」工具可命中 → 选中链路正常；缺口在**默认/只读路径的低缩放命中手感**。

## 变更类型

设计级变更（命中合同修订 + 前端命中实现 + 测试）

## 变更范围

- 影响的需求文档：无
- 影响的功能规格：`logos/resources/prd/2-product-design/1-feature-specs/core-01b-relationship.md` §4.7
  - 修订 R-HIT-02：主线命中带宽改为 **随 zoom 自适应屏幕像素**（高缩放保持 12，低缩放放宽至 20）
  - R-HIT-05 / R-HIT-07 同步使用同一自适应带宽常量函数
- 影响的业务场景：S01 编辑器画布 / 关系选中（DSPM 类密集图）
- 影响的 API：无
- 影响的 DB 表：无
- 影响的编排测试：无
- 影响的测试用例：`logos/resources/test/core-PB-relationship-test-cases.md`
  - 新增 UT-PB-30（自适应命中带宽边界）
  - 新增 UT-PB-31（低缩放近线优先与自适应带宽对齐）
  - 既有 UT-PB-18/22/28/29 在 zoom=1 口径不变
- 影响的 smoke 测试：无

## 部署影响

- 是否需要部署：否
- 部署原因：纯前端命中逻辑与规格修订，不涉 API/DB/配置/数据迁移（本地 Docker 可自行 `--build` 验证）
- 影响环境：无
- 是否涉及数据迁移：否
- 是否需要回滚预案：否
- 是否需要 smoke：否

## 变更概述

1. 引入 `rel_line_hit_px(zoom)`：zoom≥1 → 12px；zoom≤0.25 → 20px；中间线性插值。
2. `hit_test_reference_tier*`、AABB 外扩、`should_prefer_reference_over_table` 全部改用该自适应阈值（近线优先阈值始终 = 当前 zoom 下主线带宽）。
3. 规格 R-HIT-02/05/07 与 UT-PB-30/31 同步；不改 API/DB/部署。


## 需要合并的 Delta 文件

### 1. deltas/prd/2-product-design/1-feature-specs/core-01b-relationship.md

- Delta 文件：`logos/changes/fix-dense-lowzoom-relation-hit/deltas/prd/2-product-design/1-feature-specs/core-01b-relationship.md`
- 目标目录：`logos/resources/prd/2-product-design/1-feature-specs/`
- 操作：读取 delta 中的 ADDED / MODIFIED / REMOVED 标记，合并到目标目录中对应的主文档

### 2. deltas/test/core-PB-relationship-test-cases.md

- Delta 文件：`logos/changes/fix-dense-lowzoom-relation-hit/deltas/test/core-PB-relationship-test-cases.md`
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
   git add -A && git commit -m "docs(fix-dense-lowzoom-relation-hit): merge spec deltas"
   然后提示用户：按更新后的规格实现代码，代码完成后运行 `openlogos verify` 验收，验收通过后明确授权执行 `openlogos archive fix-dense-lowzoom-relation-hit`。
