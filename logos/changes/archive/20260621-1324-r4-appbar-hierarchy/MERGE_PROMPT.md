# 合并指令

## 变更提案
- 提案名称：r4-appbar-hierarchy
- 提案目录：logos/changes/r4-appbar-hierarchy/

## 提案内容

# 变更提案：r4-appbar-hierarchy

> module: core | created: 2026-06-21

## 变更原因

R1–R3 完成后，界面美学诊断 **R4** 仍待落地：AppBar 48px 单行同时承载 Logo、撤销/重做、标题、保存状态、rev、导入/导出 pill、分享、代码视图、主题、Inspector 切换等 **10+ 控件**，色彩与视线锚点分散；保存状态与标题 dirty 点重复表达「未保存」；`revision-display` 在 AppBar 与 StatusBar **双份渲染**。

## 变更类型

设计级 + 代码级（AppBar 布局与 CSS，无 API/DB 变更）

## 变更范围

- 影响的功能规格：
  - `core-05-top-menu-modals.md` — §1 AppBar 三区布局 + 溢出菜单
  - `core-00-information-architecture.md` — §1 AppBar 信息架构描述
  - `core-07-design-tokens.md` — §15.2 AppBar 分区间距 Token
- 影响的页面原型：`core-01-editor-prototype.html` — AppBar 结构对齐
- 影响的业务场景：S01（保存状态 / revision 锚点不变）
- 影响的 API / DB / 编排测试：无（保留全部 `data-testid`）

## 部署影响

- 是否需要部署：否
- 部署原因：前端 WASM/CSS 变更随常规 `trunk build` 验证，无独立部署步骤
- 影响环境：无（开发构建即可验证）
- 是否涉及数据迁移：否
- 是否需要 smoke：否

## 变更概述

**R4 AppBar 信息分层**：将 AppBar 划分为 **品牌区**（Logo + Undo/Redo + 标题）、**状态区**（单一 `.cdb-status-chip`：圆点 + 文案 + rev）、**操作区**（分享 Primary + 代码视图 + `⋯` 溢出菜单）。导入/导出/主题切换迁入溢出菜单；移除 AppBar 上重复的 Inspector 切换（保留 StatusBar `btn-inspector-toggle`）；标题旁 dirty 点移除（由状态 Chip 统一表达）。

**视觉**：操作区间距对齐 4px 网格 Token；状态 Chip 使用 `bg-subtle + border` 单一容器，降低 48px 行内色彩竞争。


## 需要合并的 Delta 文件

### 1. deltas/prd/2-product-design/1-feature-specs/core-00-information-architecture.md

- Delta 文件：`logos/changes/r4-appbar-hierarchy/deltas/prd/2-product-design/1-feature-specs/core-00-information-architecture.md`
- 目标目录：`logos/resources/prd/2-product-design/1-feature-specs/`
- 操作：读取 delta 中的 ADDED / MODIFIED / REMOVED 标记，合并到目标目录中对应的主文档

### 2. deltas/prd/2-product-design/1-feature-specs/core-05-top-menu-modals.md

- Delta 文件：`logos/changes/r4-appbar-hierarchy/deltas/prd/2-product-design/1-feature-specs/core-05-top-menu-modals.md`
- 目标目录：`logos/resources/prd/2-product-design/1-feature-specs/`
- 操作：读取 delta 中的 ADDED / MODIFIED / REMOVED 标记，合并到目标目录中对应的主文档

### 3. deltas/prd/2-product-design/1-feature-specs/core-07-design-tokens.md

- Delta 文件：`logos/changes/r4-appbar-hierarchy/deltas/prd/2-product-design/1-feature-specs/core-07-design-tokens.md`
- 目标目录：`logos/resources/prd/2-product-design/1-feature-specs/`
- 操作：读取 delta 中的 ADDED / MODIFIED / REMOVED 标记，合并到目标目录中对应的主文档

### 4. deltas/prd/2-product-design/2-page-design/core-01-editor-prototype.html

- Delta 文件：`logos/changes/r4-appbar-hierarchy/deltas/prd/2-product-design/2-page-design/core-01-editor-prototype.html`
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
   git add -A && git commit -m "docs(r4-appbar-hierarchy): merge spec deltas"
   然后提示用户：按更新后的规格实现代码，代码完成后运行 `openlogos verify` 验收，验收通过后明确授权执行 `openlogos archive r4-appbar-hierarchy`。
