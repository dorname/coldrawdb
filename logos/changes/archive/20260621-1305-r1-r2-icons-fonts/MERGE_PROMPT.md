# 合并指令

## 变更提案
- 提案名称：r1-r2-icons-fonts
- 提案目录：logos/changes/r1-r2-icons-fonts/

## 提案内容

# 变更提案：r1-r2-icons-fonts

> module: core | created: 2026-06-21

## 变更原因

界面美学诊断（R1+R2）发现：编辑器 UI 虽已具备 E2 图标库（`icons.rs` 50 个 SVG）与 E1 设计 Token，但 **`editor_panels.rs` 未接入 SVG**，ToolRail / AppBar / IO Drawer 仍使用 Emoji/Unicode（`↖` `🔗` `◐` 等），跨平台渲染不一致；同时 **`body` 未使用 `--cdb-font-family-base`**，与 HTML 原型（Plus Jakarta Sans）存在字体断层，削弱专业工具气质。

## 变更类型

设计级 + 代码级（视觉系统落地，无 API/DB 变更）

## 变更范围

- 影响的需求文档：无
- 影响的功能规格：
  - `core-07-design-tokens.md` — 新增图标尺寸 Token、更新 `--cdb-font-family-base`
  - `core-08-icon-library.md` — 补充 `IconSelect` / `IconSidebar` / `IconBox` 用法
- 影响的业务场景：S01（编辑与保存，编辑器 UI 呈现）
- 影响的 API：无
- 影响的 DB 表：无
- 影响的编排测试：无（纯前端视觉，e2e testid 不变）

## 部署影响

- 是否需要部署：否
- 部署原因：前端 WASM/CSS 变更随常规 `trunk build` 发布，无独立部署步骤
- 影响环境：无（开发构建即可验证）
- 是否涉及数据迁移：否
- 是否需要回滚预案：否（纯 UI，可回滚至上一版静态资源）
- 是否需要 smoke：否（无功能行为变更，现有 e2e 覆盖交互 testid）

## 变更概述

**R1 图标**：在 `icons.rs` 新增 `IconBox` 尺寸容器及 `IconSelect` / `IconSidebar`；将 `editor_panels.rs` 中全部 Emoji/Unicode 图标替换为 SVG 组件，统一 sm=16px（AppBar/Modal）、md=20px（ToolRail）。

**R2 字体**：在 `index.html` 加载 Plus Jakarta Sans；`styles.css` 更新 `--cdb-font-family-base` / `--cdb-font-family-display`，`body` 与表单控件继承 Token；等宽区域改用 `--cdb-font-family-mono`。未保存指示由 `*` 改为 CSS 圆点 `.cdb-dirty-dot`。


## 需要合并的 Delta 文件

### 1. deltas/prd/2-product-design/1-feature-specs/core-07-design-tokens.md

- Delta 文件：`logos/changes/r1-r2-icons-fonts/deltas/prd/2-product-design/1-feature-specs/core-07-design-tokens.md`
- 目标目录：`logos/resources/prd/2-product-design/1-feature-specs/`
- 操作：读取 delta 中的 ADDED / MODIFIED / REMOVED 标记，合并到目标目录中对应的主文档

### 2. deltas/prd/2-product-design/1-feature-specs/core-08-icon-library.md

- Delta 文件：`logos/changes/r1-r2-icons-fonts/deltas/prd/2-product-design/1-feature-specs/core-08-icon-library.md`
- 目标目录：`logos/resources/prd/2-product-design/1-feature-specs/`
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
   git add -A && git commit -m "docs(r1-r2-icons-fonts): merge spec deltas"
   然后提示用户：按更新后的规格实现代码，代码完成后运行 `openlogos verify` 验收，验收通过后明确授权执行 `openlogos archive r1-r2-icons-fonts`。
