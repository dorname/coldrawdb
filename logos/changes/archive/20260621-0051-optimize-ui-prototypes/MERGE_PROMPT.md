# 合并指令

## 变更提案
- 提案名称：optimize-ui-prototypes
- 提案目录：logos/changes/optimize-ui-prototypes/

## 提案内容

# 变更提案：optimize-ui-prototypes

> module: core | created: 2026-06-20

## 变更原因

S01–S05 HTML 原型在 `refresh-editor-prototype` 归档后已可用，但对照 **ui-ux-pro-max** 交付清单与 **core-07/08/0a** 规格仍存在差距：

1. **图标**：Tool Rail / AppBar 仍用 unicode emoji（↶、⊕、🔗 等），违反 E2「禁止 emoji 占位」
2. **Token 重复**：5 个原型各自内联 CSS，dark mode / z-index / shadow 不一致
3. **缺失 E4 交互**：编辑器原型未演示 Command Palette（`Ctrl+K`）与 Code View 全屏模态
4. **无障碍**：部分原型缺少 `:focus-visible` 环、`prefers-reduced-motion`、icon-only 按钮 `aria-label`

## 变更类型

设计级变更（Phase 2 产品设计层 — 原型与交互规格微调）

## 变更范围

- 影响的需求文档：无
- 影响的功能规格：`core-S01-edit-and-save-design.md`（补充 E4 原型锚点）
- 影响的业务场景：S01（主）、S03/S04/S05（视觉统一）
- 影响的 API：无
- 影响的 DB 表：无
- 影响的编排测试：无

## 部署影响

- 是否需要部署：**否**（仅规格/原型文档）
- 部署原因：文档变更，不涉及运行时
- 影响环境：无
- 是否涉及数据迁移：否
- 是否需要回滚预案：否
- 是否需要 smoke：否

## 变更概述

1. **新增** `core-00-prototype-shared.css`：统一 token 子集、按钮/图标/焦点/动效降级基类，供 5 个 HTML 原型 `@import`。
2. **升级** `core-01-editor-prototype.html`：SVG 图标、Command Palette（`Ctrl+K`）、Code View 模态、键盘 Esc 关闭。
3. **统一** S03/S04/S05 原型：引用共享 CSS、替换 emoji、补齐 focus-visible 与 reduced-motion。
4. **更新** S01 交互设计文档：补充 `[data-testid="command-palette"]` / `[data-testid="code-view-modal"]` 锚点说明。


## 需要合并的 Delta 文件

### 1. deltas/prd/2-product-design/1-feature-specs/core-S01-edit-and-save-design.md

- Delta 文件：`logos/changes/optimize-ui-prototypes/deltas/prd/2-product-design/1-feature-specs/core-S01-edit-and-save-design.md`
- 目标目录：`logos/resources/prd/2-product-design/1-feature-specs/`
- 操作：读取 delta 中的 ADDED / MODIFIED / REMOVED 标记，合并到目标目录中对应的主文档

### 2. deltas/prd/2-product-design/2-page-design/core-00-prototype-shared.css

- Delta 文件：`logos/changes/optimize-ui-prototypes/deltas/prd/2-product-design/2-page-design/core-00-prototype-shared.css`
- 目标目录：`logos/resources/prd/2-product-design/2-page-design/`
- 操作：读取 delta 中的 ADDED / MODIFIED / REMOVED 标记，合并到目标目录中对应的主文档

### 3. deltas/prd/2-product-design/2-page-design/core-01-editor-prototype.html

- Delta 文件：`logos/changes/optimize-ui-prototypes/deltas/prd/2-product-design/2-page-design/core-01-editor-prototype.html`
- 目标目录：`logos/resources/prd/2-product-design/2-page-design/`
- 操作：读取 delta 中的 ADDED / MODIFIED / REMOVED 标记，合并到目标目录中对应的主文档

### 4. deltas/prd/2-product-design/2-page-design/core-03-auth-prototype.html

- Delta 文件：`logos/changes/optimize-ui-prototypes/deltas/prd/2-product-design/2-page-design/core-03-auth-prototype.html`
- 目标目录：`logos/resources/prd/2-product-design/2-page-design/`
- 操作：读取 delta 中的 ADDED / MODIFIED / REMOVED 标记，合并到目标目录中对应的主文档

### 5. deltas/prd/2-product-design/2-page-design/core-04-collab-prototype.html

- Delta 文件：`logos/changes/optimize-ui-prototypes/deltas/prd/2-product-design/2-page-design/core-04-collab-prototype.html`
- 目标目录：`logos/resources/prd/2-product-design/2-page-design/`
- 操作：读取 delta 中的 ADDED / MODIFIED / REMOVED 标记，合并到目标目录中对应的主文档

### 6. deltas/prd/2-product-design/2-page-design/core-05-ot-collab-prototype.html

- Delta 文件：`logos/changes/optimize-ui-prototypes/deltas/prd/2-product-design/2-page-design/core-05-ot-collab-prototype.html`
- 目标目录：`logos/resources/prd/2-product-design/2-page-design/`
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
   git add -A && git commit -m "docs(optimize-ui-prototypes): merge spec deltas"
   然后提示用户：按更新后的规格实现代码，代码完成后运行 `openlogos verify` 验收，验收通过后明确授权执行 `openlogos archive optimize-ui-prototypes`。
