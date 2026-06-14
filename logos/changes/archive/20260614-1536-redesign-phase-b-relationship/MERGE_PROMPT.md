# 合并指令

## 变更提案
- 提案名称：redesign-phase-b-relationship
- 提案目录：logos/changes/redesign-phase-b-relationship/

## 提案内容

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


## 需要合并的 Delta 文件

### 1. deltas/prd/2-product-design/1-feature-specs/core-01b-relationship.md

- Delta 文件：`logos/changes/redesign-phase-b-relationship/deltas/prd/2-product-design/1-feature-specs/core-01b-relationship.md`
- 目标目录：`logos/resources/prd/2-product-design/1-feature-specs/`
- 操作：读取 delta 中的 ADDED / MODIFIED / REMOVED 标记，合并到目标目录中对应的主文档

### 2. deltas/test/core-PB-relationship-test-cases.md

- Delta 文件：`logos/changes/redesign-phase-b-relationship/deltas/test/core-PB-relationship-test-cases.md`
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
   git add -A && git commit -m "docs(redesign-phase-b-relationship): merge spec deltas"
   然后提示用户：按更新后的规格实现代码，代码完成后运行 `openlogos verify` 验收，验收通过后明确授权执行 `openlogos archive redesign-phase-b-relationship`。
