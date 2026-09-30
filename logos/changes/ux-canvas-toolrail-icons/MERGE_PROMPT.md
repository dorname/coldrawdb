# 合并指令

## 变更提案
- 提案名称：ux-canvas-toolrail-icons
- 提案目录：logos/changes/ux-canvas-toolrail-icons/

## 提案内容

# 变更提案：ux-canvas-toolrail-icons

> module: core | created: 2026-09-30

## 变更原因
用户在 `/goal` 中提出「左侧菜单的图标太丑了寻找合适的替换」。当前 ToolRail 中部分图标为占位实现或视觉风格不统一，影响编辑器整体质感，需要统一替换为更简洁、语义清晰的 SVG 图标。

## 变更类型
设计级 / 代码级

## 变更范围
- 影响的需求文档：无
- 影响的功能规格：
  - `logos/resources/prd/2-product-design/1-feature-specs/core-04-side-panel-tabs.md` §1 ToolRail 按钮清单（更新图标组件名）
  - `logos/resources/prd/2-product-design/1-feature-specs/core-08-icon-library.md`（新增/更新图标条目）
- 影响的业务场景：无新增/删除场景
- 影响的 API：无
- 影响的 DB 表：无
- 影响的编排测试：
  - 更新 `UT-PB-27` 源码锚点（ToolRail 按钮与 testid 存在性）
  - 新增 `UT-E2-ICON-03`（可选）锁定图标组件存在

## 部署影响
- 是否需要部署：是
- 部署原因：前端 SVG 组件/CSS 变更需重新 build
- 影响环境：生产前端
- 是否涉及数据迁移：否
- 是否需要回滚预案：否
- 是否需要 smoke：否

## 变更概述
1. 在 `frontend-rs/src/icons.rs` 中新增/重绘 ToolRail 所需图标组件：`IconAddTable`、`IconSelect`、`IconMarquee`、`IconRelationshipCreate`、`IconRelationshipSelect`、`IconNewArea`、`IconNewNote`、`IconPan`、`IconSearchCommand`、`IconDictionary`、`IconSettings`。
2. 保持所有按钮 `data-testid`、快捷键、disabled 门控不变；仅替换 `<IconBox>` 内的 SVG 组件。
3. 同步更新 `core-08-icon-library.md` 图标清单与 `core-04-side-panel-tabs.md` ToolRail 按钮清单中的图标引用。
4. 更新 `UT-PB-27` 断言，确认新图标组件名在 `icons.rs` 中存在；保持 e2e 对 `tool-relationship-create/select` 等 testid 的依赖不变。


## 需要合并的 Delta 文件

### 1. deltas/prd/2-product-design/1-feature-specs/core-04-side-panel-tabs.md

- Delta 文件：`logos/changes/ux-canvas-toolrail-icons/deltas/prd/2-product-design/1-feature-specs/core-04-side-panel-tabs.md`
- 目标目录：`logos/resources/prd/2-product-design/1-feature-specs/`
- 操作：读取 delta 中的 ADDED / MODIFIED / REMOVED 标记，合并到目标目录中对应的主文档

### 2. deltas/prd/2-product-design/1-feature-specs/core-08-icon-library.md

- Delta 文件：`logos/changes/ux-canvas-toolrail-icons/deltas/prd/2-product-design/1-feature-specs/core-08-icon-library.md`
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
   git add -A && git commit -m "docs(ux-canvas-toolrail-icons): merge spec deltas"
   然后提示用户：按更新后的规格实现代码，代码完成后运行 `openlogos verify` 验收，验收通过后明确授权执行 `openlogos archive ux-canvas-toolrail-icons`。
