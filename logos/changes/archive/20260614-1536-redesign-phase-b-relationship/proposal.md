# 变更提案：redesign-phase-b-relationship

> module: core | created: 2026-06-14
> 前置：`redesign-phase-a-layout` 已归档（Phase A 布局 + 代码已落地）

## 变更原因

Phase A 完成布局重构后，核心建模能力仍有缺口：

1. **关系工具**（`tool-relationship`）为占位，无法通过 UI 创建外键关系。
2. **Inspector 关系面板**显示「关系属性 (Phase B)」占位，无法编辑 cardinality / onDelete / onUpdate。
3. **字段 Inspector** 主键复选框为 `disabled`，无法切换 PK。
4. 画布连线仍使用表头中点，未对齐字段级端点（与 `core-01b` 规格偏差）。

## 变更类型

设计级 + 代码实现（关系工具交互 + Inspector 增强）

## 变更范围

- `core-01b-relationship.md` — 新增关系工具模式 + 确认条
- `core-01-editor-canvas.md` — 关系工具手势、字段级连线渲染
- `core-06-inspector-panel.md`（归档 delta 对齐）— InspectorReference / 字段 PK
- **新增** `core-PB-relationship-test-cases.md`
- 代码：`editor_panels.rs`、`editor_render.rs`、`styles.css`

## 部署影响

- 是否需要部署：否
- 是否需要 smoke：否（本地 UT + 可选 e2e 增补）

## 变更概述

### Phase B 交付物

1. **关系工具双步选取**：激活 `🔗` → 点源字段 → 点目标字段 → 底部确认条。
2. **关系确认条**（非模态）：显示 `源.字段 → 目标.字段` + cardinality 下拉 + 创建/取消。
3. **InspectorReference**：编辑 `type_`（cardinality）、`on_delete`、`on_update`、翻转、删除。
4. **字段 PK 切换**：Inspector 主键复选框可交互（单表仅一个 PK）。
5. **字段级贝塞尔连线**：`draw_canvas` 使用 `start_field_id` / `end_field_id` 定位端点。
