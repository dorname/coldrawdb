# 合并指令

## 变更提案
- 提案名称：fix-issue-46-hover-perf-and-corner-summary
- 提案目录：logos/changes/fix-issue-46-hover-perf-and-corner-summary/

## 提案内容

# 变更提案：fix-issue-46-hover-perf-and-corner-summary

> module: core | created: 2026-09-28

## 变更原因

GitHub issue #46「关系线关联文本导致画布抖动；密集场景下选中难 + 展示策略需优化」（#43 悬浮 tooltip 的后续反馈）：

- **抖动（P0）**：`hover_ref` 信号携带 client 坐标，pointermove 每事件坐标都变 → `set` 每事件触发 → tooltip DOM 每事件重建 → 画布间歇闪动
- **展示策略（P1）**：tooltip 用视口 client 坐标 `position:fixed` 绑定，实测出现在画布左下角——坐标系与产品意图不一致；issue 明确「在选中/密集展示未解决前，固定角落摘要区是更合理的选择」
- **选中（P2）**：#44 命中合同（最近距离优先/阈值外不命中/悬停点击同口径）已交付，本变更保持回归不增强

## 变更类型

设计级变更（展示策略选型 + hover 信号合同变更；规格口径更新）

## 变更范围

- 影响的需求文档：无（issue 即需求来源）
- 影响的功能规格：`core-01b-relationship.md`（§4.8 R-HOV-*：展示策略从「光标右下 12px 跟随」改为「B1 固定角落摘要区」+ 新增 R-PERF-HOV 防抖/守卫合同）
- 影响的业务场景：S01（画布交互域，时序图不变——纯前端渲染路径优化）
- 影响的 API：无
- 影响的 DB 表：无
- 影响的编排测试：无（纯前端）
- 影响的测试文档：`core-PB-relationship-test-cases.md`（UT/ST：hover 守卫同 ref 不反复 set、角落定位锚点、坐标不绑定 client；ST-PB-12 口径修订为角落摘要）
- 影响的部署方案：`core-01-deployment-plan.md`（前端镜像重建注记；无 migration、无新 smoke 用例）

## 部署影响

- 是否需要部署：是
- 部署原因：纯前端画布代码变更，需重建 compose 前端镜像生效
- 影响环境：本地 / 生产（compose）
- 是否涉及数据迁移：否
- 是否需要回滚预案：否（纯前端展示/性能逻辑，后向兼容）
- 是否需要 smoke：是（部署后既有 smoke 套件回归，不新增用例）

## 变更概述

1. **展示策略选型 B1（固定角落摘要区）**：tooltip 从「光标右下 12px 跟随」改为画布容器内**左下角固定摘要区**（`position:absolute; left/bottom` 锚定 `.cdb-canvas-stack`），悬停命中关系线时显示 `源表.源字段 → 目标表.目标字段`（详情档）/ `源表 → 目标表`（拓扑档）。理由：①密集场景下固定角落稳定可预期、不挡线、不依赖线旁几何（issue 作者认可）；②坐标不再来自指针事件，从根上消除坐标绑定 bug 类。
2. **抖动消除（R-PERF-HOV）**：`hover_ref` 信号瘦身——只存 `ref_id`（坐标移出信号）；**守卫**：同 `ref_id` 命中时不再 `set`（`get_untracked` 比较跳过），仅命中变化/消失时写信号；tooltip 文案不变 → DOM 不重建。DOM 浮层与 canvas paint 本就解耦（不触发 schedule_paint），守卫后 pointermove 常态路径零信号写入。
3. **命中节流**：pointermove 的 hover 命中检测用 rAF 合并（每帧至多一次 hit_test），密集关系大图下避免每事件全量遍历。
4. **选中保持 #44 合同回归**：hit_test_reference_tier 线型同源+最近距离优先不动；ST-PB-11 继续绿。


## 需要合并的 Delta 文件

### 1. deltas/prd/2-product-design/1-feature-specs/core-01b-relationship.md

- Delta 文件：`logos/changes/fix-issue-46-hover-perf-and-corner-summary/deltas/prd/2-product-design/1-feature-specs/core-01b-relationship.md`
- 目标目录：`logos/resources/prd/2-product-design/1-feature-specs/`
- 操作：读取 delta 中的 ADDED / MODIFIED / REMOVED 标记，合并到目标目录中对应的主文档

### 2. deltas/prd/3-technical-plan/3-deployment/core-01-deployment-plan.md

- Delta 文件：`logos/changes/fix-issue-46-hover-perf-and-corner-summary/deltas/prd/3-technical-plan/3-deployment/core-01-deployment-plan.md`
- 目标目录：`logos/resources/prd/3-technical-plan/3-deployment/`
- 操作：读取 delta 中的 ADDED / MODIFIED / REMOVED 标记，合并到目标目录中对应的主文档

### 3. deltas/test/core-PB-relationship-test-cases.md

- Delta 文件：`logos/changes/fix-issue-46-hover-perf-and-corner-summary/deltas/test/core-PB-relationship-test-cases.md`
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
   git add -A && git commit -m "docs(fix-issue-46-hover-perf-and-corner-summary): merge spec deltas"
   然后提示用户：按更新后的规格实现代码，代码完成后运行 `openlogos verify` 验收，验收通过后明确授权执行 `openlogos archive fix-issue-46-hover-perf-and-corner-summary`。
