# 合并指令

## 变更提案
- 提案名称：r5-inspector-tabs
- 提案目录：logos/changes/r5-inspector-tabs/

## 提案内容

# 变更提案：r5-inspector-tabs

> module: core | created: 2026-06-21

## 变更原因

R4 完成后，界面美学诊断 **R5** 仍待落地：Inspector 320px 内叠放 **7 个文字 Tab（换行拥挤）** + 搜索 + 类型筛选 + 列表，下方 **45% 固定分割** 的字段编辑区（`.cdb-side-panel--right`），像 V1 左右两栏硬塞进侧栏，缺少「间」；Tab 栏 `padding: 4px 8px` 与 `cdb-tabs--wrap` 进一步放大密度感。

## 变更类型

设计级 + 代码级（Inspector Tab 布局与 CSS，无 API/DB 变更）

## 变更范围

- 影响的功能规格：
  - `core-01-editor-canvas.md` — §6.5 Inspector Tab 图标栅格 + 字段 Tab
  - `core-07-design-tokens.md` — §15.3 Inspector Tab 尺寸 Token
- 影响的页面原型：`core-01-editor-prototype.html` — Inspector Tab 结构
- 影响的测试用例：`core-SP-side-panel-test-cases.md` — 8 Tab + 图标栏
- 影响的业务场景：S01（field-editor 锚点不变）
- 影响的 API / DB / 编排测试：无（保留 `tab-*` / `field-editor` testid）

## 部署影响

- 是否需要部署：否
- 是否需要 smoke：否
- 是否涉及数据迁移：否

## 变更概述

**R5 Inspector Tab 降密**：

1. **图标 Tab 栏**：7 业务 Tab + **字段 Tab** 改为 **4×2 图标栅格**（`.cdb-tabs--icon-grid`），`title` 提供 Tooltip；保留全部 `data-testid="tab-*"`。
2. **字段区独立 Tab**：移除 `.cdb-side-panel--right` 45% 底部分割；字段编辑迁入 `tab-fields`，内容区 **全高单栏**。
3. **选中联动**：画布/列表选中表时自动切至 **字段 Tab**（`field-editor` 行为与 S01 一致）。
4. **搜索/筛选**：仅在非字段 Tab 显示 `side-search` / `type-filter`。


## 需要合并的 Delta 文件

### 1. deltas/prd/2-product-design/1-feature-specs/core-01-editor-canvas.md

- Delta 文件：`logos/changes/r5-inspector-tabs/deltas/prd/2-product-design/1-feature-specs/core-01-editor-canvas.md`
- 目标目录：`logos/resources/prd/2-product-design/1-feature-specs/`
- 操作：读取 delta 中的 ADDED / MODIFIED / REMOVED 标记，合并到目标目录中对应的主文档

### 2. deltas/prd/2-product-design/1-feature-specs/core-07-design-tokens.md

- Delta 文件：`logos/changes/r5-inspector-tabs/deltas/prd/2-product-design/1-feature-specs/core-07-design-tokens.md`
- 目标目录：`logos/resources/prd/2-product-design/1-feature-specs/`
- 操作：读取 delta 中的 ADDED / MODIFIED / REMOVED 标记，合并到目标目录中对应的主文档

### 3. deltas/prd/2-product-design/2-page-design/core-01-editor-prototype.html

- Delta 文件：`logos/changes/r5-inspector-tabs/deltas/prd/2-product-design/2-page-design/core-01-editor-prototype.html`
- 目标目录：`logos/resources/prd/2-product-design/2-page-design/`
- 操作：读取 delta 中的 ADDED / MODIFIED / REMOVED 标记，合并到目标目录中对应的主文档

### 4. deltas/test/core-SP-side-panel-test-cases.md

- Delta 文件：`logos/changes/r5-inspector-tabs/deltas/test/core-SP-side-panel-test-cases.md`
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
   git add -A && git commit -m "docs(r5-inspector-tabs): merge spec deltas"
   然后提示用户：按更新后的规格实现代码，代码完成后运行 `openlogos verify` 验收，验收通过后明确授权执行 `openlogos archive r5-inspector-tabs`。
