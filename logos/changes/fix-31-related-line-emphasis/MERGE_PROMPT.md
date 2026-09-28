# 合并指令

## 变更提案
- 提案名称：fix-31-related-line-emphasis
- 提案目录：logos/changes/fix-31-related-line-emphasis/

## 提案内容

# 变更提案：fix-31-related-line-emphasis

> module: core | created: 2026-09-28

## 变更原因

GitHub issue #31 被重开：变更 fix-open-issues-26-33 批次 C4 已实现「相关连线 1.5× 加粗、非相关线 alpha ≤ 0.25、非相关表 alpha ≤ 0.5」，verify PASS 并关闭；用户 2026-09-28 截图反馈「目前选中表对应的关系连线的效果还不明显」。

根因分析：当前相关连线仅**加粗**（2.0 → 3.0 世界宽），主色仍沿用 `palette.relation`（暗主题 `rgb(128,234,225)`、亮主题 `rgb(34,115,129)`），halo 为暗色 `relation_halo`；在暗主题点阵背景上明暗差不足以「一眼可辨」。§4.4 合同允许「颜色沿用关系解析色或 palette.selected」，实现取了前者——需修订合同为强制提亮到选中色族。

## 变更类型

设计级变更（视觉合同修订 + 同源代码实现对齐）

## 变更范围

- 影响的需求文档：无（S01 需求不变，验收条件由设计规格承载）
- 影响的功能规格：
  - `core-01b-relationship.md` §4.4 密度降噪（MODIFIED：相关关系线颜色/光晕/端点口径）
  - `core-07-design-tokens.md` §15.5 canvas.highlight（MODIFIED：related-line-scale 勘误对齐 + 新增相关线色/光晕 token）
- 影响的业务场景：S01（画布编辑）
- 影响的 API：无
- 影响的 DB 表：无
- 影响的编排测试：无（非 API 编排项）
- 影响的测试用例：`core-PE-design-system-test-cases.md` UT-PE-HL-01 / ST-PE-10（MODIFIED 断言口径）
- 影响的代码：`frontend-rs/src/editor_render.rs`（draw_relation related 分支、端点着色）、`frontend-rs/scripts/test-spec-parity-d.mjs`（ST-PE-10 断言）

## 部署影响

- 是否需要部署：否
- 部署原因：纯前端 canvas 渲染视觉参数调整，openlogos verify（含 parity-d e2e）即可闭环验收，无运行时服务变更
- 影响环境：无
- 是否涉及数据迁移：否
- 是否需要回滚预案：否
- 是否需要 smoke：否

## 变更概述

选中表/关系时，相关关系线从「原色 + 1.5× 加粗」强化为「选中色族 + 发光」：主线色强制 `palette.selected`（暗主题亮青 #5ee9dc / 亮主题 #1e8393），光晕换用 `palette.selected_soft`（8px，选中关系自身 10px 保持层级区分），线宽维持 1.5×；crow's foot 端点记号与主线同色（顺带修正选中线端点仍用基线色的不一致）。非相关线/表降透明口径不变。

同步勘误：`core-07` §15.5 `related-line-scale` 登记值 2.0 与实现 1.5 不一致，统一为 1.5（与 `RELATED_RELATION_WIDTH_FACTOR` 同源）。验收沿用 UT-PE-HL-01（断言更新）+ ST-PE-10（探针断言更新），不新增用例 ID。


## 需要合并的 Delta 文件

### 1. deltas/prd/2-product-design/1-feature-specs/core-01b-relationship.md

- Delta 文件：`logos/changes/fix-31-related-line-emphasis/deltas/prd/2-product-design/1-feature-specs/core-01b-relationship.md`
- 目标目录：`logos/resources/prd/2-product-design/1-feature-specs/`
- 操作：读取 delta 中的 ADDED / MODIFIED / REMOVED 标记，合并到目标目录中对应的主文档

### 2. deltas/prd/2-product-design/1-feature-specs/core-07-design-tokens.md

- Delta 文件：`logos/changes/fix-31-related-line-emphasis/deltas/prd/2-product-design/1-feature-specs/core-07-design-tokens.md`
- 目标目录：`logos/resources/prd/2-product-design/1-feature-specs/`
- 操作：读取 delta 中的 ADDED / MODIFIED / REMOVED 标记，合并到目标目录中对应的主文档

### 3. deltas/test/core-PE-design-system-test-cases.md

- Delta 文件：`logos/changes/fix-31-related-line-emphasis/deltas/test/core-PE-design-system-test-cases.md`
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
   git add -A && git commit -m "docs(fix-31-related-line-emphasis): merge spec deltas"
   然后提示用户：按更新后的规格实现代码，代码完成后运行 `openlogos verify` 验收，验收通过后明确授权执行 `openlogos archive fix-31-related-line-emphasis`。
