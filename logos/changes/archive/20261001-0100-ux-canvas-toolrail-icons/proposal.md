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
