# 变更提案：fix-issues-34-35-header-comment-lod-anchor

> module: core | created: 2026-09-28

## 变更原因

GitHub 两个新建 BUG issue：

- **#34**：亮主题下，表头中文注释压在 135° 渐变亮段（橙/粉/紫/棕表头右段）上几乎不可读。根因：`comment_foreground` 对比度按**满强度 tint 合成背景**单点评估，但注释实际渲染在渐变中右段——该处 tint 已衰减近透明，有效背景 ≈ 表体底色。表名在左段（满强度 tint）不受影响，注释恰好落在衰减区。现行合同 R-CMT-CONTRAST-01 本就要求「按注释实际渲染位置的有效背景计算」，实现未达标，属合同符合性缺陷。
- **#35**：拓扑档（zoom ≤ 0.5 仅表头视图）两处：① 中文注释不渲染——R-LOD-02 本就要求「表名 + 可选注释一行」，实现仅画 primary 主文本，属合同符合性缺陷；② 关系线仍按字段纵向锚点扇形散开——字段行已隐藏，线却挂在不可见字段高度上。锚定口径现行合同未覆盖，需新增 R-LOD-08（拓扑档表级锚定）。

## 变更类型

设计级变更（一处新增合同条款 R-LOD-08 + 两处既有合同符合性修复 + 测试口径更新）

## 变更范围

- 影响的需求文档：无（S01 需求不变）
- 影响的功能规格：
  - `core-01a-table-and-field.md` R-CMT-CONTRAST 段（MODIFIED：#34 双端点 worst-case 判据明文化）
  - `core-01-editor-canvas.md` §5.12 LOD（ADDED R-LOD-08 表级锚定；R-LOD-02 注释渲染补实现注记）
- 影响的业务场景：S01（画布编辑）
- 影响的 API：无
- 影响的 DB 表：无
- 影响的编排测试：无
- 影响的测试用例：
  - `core-PE-design-system-test-cases.md` UT-PE-CMT-01（MODIFIED：渐变双端点断言 + 橙/粉/紫/棕表头）
  - `core-CR-canvas-test-cases.md` UT-CR-LOD-01 / ST-CR-LOD-01（MODIFIED：拓扑档注释 + 表级锚点断言）
- 影响的代码：
  - `frontend-rs/src/editor_render.rs`（comment_foreground 双端点、draw_table_topology_body 注释、calc_path/calc_orthogonal_path/calc_straight_path tier 化、draw_relation/draw_canvas 透传、lod 探针字段）
  - `frontend-rs/tests/fix_issues_19_22_ut.rs`、`frontend-rs/tests/canvas_path_resize_ghost_ut.rs`（calc_* 签名对齐）
  - `frontend-rs/scripts/test-spec-parity-d.mjs`（ST-CR-LOD-01 断言）

## 部署影响

- 是否需要部署：否
- 部署原因：纯前端 canvas 渲染层调整，openlogos verify（含 parity-d e2e）闭环验收
- 影响环境：无
- 是否涉及数据迁移：否
- 是否需要回滚预案：否
- 是否需要 smoke：否

## 变更概述

**#34**：`comment_foreground` 决策改为渐变双端点 worst-case——候选前景（muted→strong→反向 strong→chip 兜底）须在「满强度 tint 合成背景」与「纯表体背景（渐变衰减端）」两端同时 ≥ 4.5:1 才免 chip；chip 兜底同样双端合成评估取最优。字段行注释入参 tint 本为全透明，两端点退化为同一样本，行为不变。

**#35**：① 拓扑档卡体在 NameComment 模式且注释非空时渲染注释副文本（字号 = 拓扑表名字号 × 0.8、字重 ≥ 500，前景走 #34 同一对比度决策，宽度沿用 `resolve_table_width` 已有的注释感知估算，超长截断）；② 关系路径锚点 tier 化——拓扑档两端锚点收敛为表级端口（表头纵向中线 × 左/右缘，选侧沿用 `pick_port_sides`），详情档保持字段锚定不变；跨档切换由既有精灵指纹/逐帧重绘覆盖。LOD 探针暴露锚定模式供 e2e 断言。
