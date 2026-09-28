# 合并指令

## 变更提案
- 提案名称：fix-issue-47-relation-hit-precision-and-highlight
- 提案目录：logos/changes/fix-issue-47-relation-hit-precision-and-highlight/

## 提案内容

# 变更提案：fix-issue-47-relation-hit-precision-and-highlight

> module: core | created: 2026-09-28

## 变更原因

用户反馈（Image #1）：

1. **Bug**：关系连线点击不够精确——鼠标位置（黄圈）明显离关系线还有一段距离，却已命中/显示摘要。根因：`hit_test_reference_tier` 的 8px 阈值以**世界坐标**度量，缩放较低时（如截图约 62%）屏幕命中区被放大，导致“隔空命中”。
2. **新增需求**：点击关系线后，关系线与对应的两张表应同时高亮。现状：选中关系线自身有加粗（3.5×），但两端表仅保持不透明，没有明显高亮（选中环/边框）。

## 变更类型

设计级变更（命中合同修正 + 选中高亮交互增强；规格口径更新）

## 变更范围

- 影响的需求文档：无（用户反馈即需求来源）
- 影响的功能规格：`core-01b-relationship.md`（§4.7 R-HIT 命中阈值改为屏幕像素等价；§4.9 新增关系线选中时两端表高亮合同）
- 影响的业务场景：S01（画布交互域，时序图不变）
- 影响的 API：无
- 影响的 DB 表：无
- 影响的编排测试：无（纯前端）
- 影响的测试文档：`core-PB-relationship-test-cases.md`（UT：命中阈值屏幕像素、端点表高亮；ST：点击关系线后两端表高亮 e2e）
- 影响的部署方案：`core-01-deployment-plan.md`（前端镜像重建注记；无 migration、无新 smoke 用例）

## 部署影响

- 是否需要部署：是
- 部署原因：纯前端画布代码变更，需重建 compose 前端镜像生效
- 影响环境：本地 / 生产（compose）
- 是否涉及数据迁移：否
- 是否需要回滚预案：否（纯前端交互/高亮逻辑，后向兼容）
- 是否需要 smoke：是（部署后既有 smoke 套件回归，不新增用例）

## 变更概述

1. **命中精度修复**：`hit_test_reference_tier` 增加 `zoom: f64` 入参，阈值判断从 `d <= 8.0`（世界像素）改为 `d * zoom <= 8.0`（等价 8 屏幕像素）。这样在任意缩放下图纸关系线的可点击带宽恒为 8 屏幕像素，低 zoom 时不再“隔空命中”。
2. **关系线选中高亮两端表**：`draw_canvas` 在计算表的 `is_sel` 时，若 `selected_ref_id` 命中某条关系线，则该关系线 `start_table_id` / `end_table_id` 对应的表也视为视觉高亮（传入 `draw_table` 的 `selected=true`），实现“点击关系线 → 关系线加粗 + 两端表高亮”的联动效果；不影响真实选中集合（仅视觉）。
3. **规格与测试对齐**：R-HIT 合同明确命中阈值为“8 屏幕像素等价”；新增 R-HL-REF 高亮合同；UT-PB-22/23 与 ST-PB-13 入账。


## 需要合并的 Delta 文件

### 1. deltas/prd/2-product-design/1-feature-specs/core-01b-relationship.md

- Delta 文件：`logos/changes/fix-issue-47-relation-hit-precision-and-highlight/deltas/prd/2-product-design/1-feature-specs/core-01b-relationship.md`
- 目标目录：`logos/resources/prd/2-product-design/1-feature-specs/`
- 操作：读取 delta 中的 ADDED / MODIFIED / REMOVED 标记，合并到目标目录中对应的主文档

### 2. deltas/prd/3-technical-plan/3-deployment/core-01-deployment-plan.md

- Delta 文件：`logos/changes/fix-issue-47-relation-hit-precision-and-highlight/deltas/prd/3-technical-plan/3-deployment/core-01-deployment-plan.md`
- 目标目录：`logos/resources/prd/3-technical-plan/3-deployment/`
- 操作：读取 delta 中的 ADDED / MODIFIED / REMOVED 标记，合并到目标目录中对应的主文档

### 3. deltas/test/core-PB-relationship-test-cases.md

- Delta 文件：`logos/changes/fix-issue-47-relation-hit-precision-and-highlight/deltas/test/core-PB-relationship-test-cases.md`
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
   git add -A && git commit -m "docs(fix-issue-47-relation-hit-precision-and-highlight): merge spec deltas"
   然后提示用户：按更新后的规格实现代码，代码完成后运行 `openlogos verify` 验收，验收通过后明确授权执行 `openlogos archive fix-issue-47-relation-hit-precision-and-highlight`。
