# 合并指令

## 变更提案
- 提案名称：fix-table-header-width-autofit
- 提案目录：logos/changes/fix-table-header-width-autofit/

## 提案内容

# 变更提案：fix-table-header-width-autofit

> module: core | created: 2026-09-30

## 变更原因

用户截图反馈：表头「英文表名 + 中文注释」并排展示时被截断（如 `asset_kind_catalog 资产类型字典；客户确认分类后…`），表卡宽度未按表头内容自适应加宽。根因：`resolve_table_width_for` 对 auto 宽度按 `estimate_content_width_for(...).clamp(TABLE_WIDTH, max_w)` 钳制，Detail 维度 `max_w = TABLE_WIDTH_MAX = 480`；实测 `asset_kind_catalog`（18 ASCII ≈144px）+ 注释「资产类型字典；客户确认分类后由 seed 脚本初始化」（20 CJK + 5 ASCII ≈320px）+ 计数占位/留白 ≈507px > 480px，被钳到 480 后绘制侧 `truncate_to_width` 必然截断注释。#48（R-LOD-10）只放宽了 Topology 维度（640px），Detail 维度未覆盖。

## 变更类型

设计级变更

## 变更范围

- 影响的功能规格：`logos/resources/prd/2-product-design/1-feature-specs/core-01-editor-canvas.md`
  - §5.12 表/字段维度与可读性补偿（R-LOD 系列规则表）
- 影响的业务场景：S01 编辑器画布 / 表卡渲染
- 影响的 API：无
- 影响的 DB 表：无
- 影响的编排测试：无
- 影响的测试用例：`logos/resources/test/core-CR-canvas-test-cases.md`
  - 新增 UT-CR-HEADER-WIDTH-01：表头行宽度需求估算纯函数（不受 Tier 上限钳制）
  - 新增 ST-CR-HEADER-WIDTH-01：长中英文注释表头在 Detail 维度完整显示不截断

## 部署影响

- 是否需要部署：否
- 部署原因：纯前端渲染宽度逻辑调整，不涉 API、DB、配置或数据迁移；本地 dev / 已部署 staging 前端镜像重新 build 后即可生效
- 影响环境：无
- 是否涉及数据迁移：否
- 是否需要回滚预案：否
- 是否需要 smoke：否

## 变更概述

在 V1 边界内采用最小可行方案：

1. **表头行宽度需求不受 Tier 上限钳制**：`estimate_content_width_for` 拆分「表头行需求」（表名 + 注释 + 字段计数 + 留白）与「字段行需求」（各字段行最大值）。`resolve_table_width_for` 在 auto 宽（`width` 为 None/0）时：字段行需求沿用既有 `clamp(TABLE_WIDTH, Tier 上限)`（R-LOD-10 不回退）；表头行需求单独参与取最大，仅受新增绝对上限 `TABLE_WIDTH_MAX_HEADER = 720px` 钳制（防御超长注释）。即 `宽度 = max(clamp(字段行需求), min(表头行需求, 720))`。
2. **显式宽度语义不变**：`table.width` 为正数时仍按用户固定宽渲染，不做内容自适应（保持用户手动调宽意图，与本 bug 无关）。
3. 关系锚点（R-WIDTH-08）与宽度同源，随新宽度自动对齐，无需单独改动。

本方案不改变字体/字号/注释显示模式（R-CMT-01、R-FONT 系列），仅调整 auto 宽度的钳制结构。


## 需要合并的 Delta 文件

### 1. deltas/prd/2-product-design/1-feature-specs/core-01-editor-canvas.md

- Delta 文件：`logos/changes/fix-table-header-width-autofit/deltas/prd/2-product-design/1-feature-specs/core-01-editor-canvas.md`
- 目标目录：`logos/resources/prd/2-product-design/1-feature-specs/`
- 操作：读取 delta 中的 ADDED / MODIFIED / REMOVED 标记，合并到目标目录中对应的主文档

### 2. deltas/test/core-CR-canvas-test-cases.md

- Delta 文件：`logos/changes/fix-table-header-width-autofit/deltas/test/core-CR-canvas-test-cases.md`
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
   git add -A && git commit -m "docs(fix-table-header-width-autofit): merge spec deltas"
   然后提示用户：按更新后的规格实现代码，代码完成后运行 `openlogos verify` 验收，验收通过后明确授权执行 `openlogos archive fix-table-header-width-autofit`。
