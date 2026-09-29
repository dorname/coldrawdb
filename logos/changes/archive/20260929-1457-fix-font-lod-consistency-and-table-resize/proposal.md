# 变更提案：fix-font-lod-consistency-and-table-resize

> module: core | created: 2026-09-29

## 变更原因

用户反馈（截图）：画布在部分放大/缩放下，表卡内文本字号会自动变化（受 R-FONT-04 10px 屏幕下限钳制 / R-LOD-03 表维度可读性补偿影响），但：

1. **不可见部分没有同步变化**——缩小到全景后，之前离屏的表与一直可见的表字体大小不一致，出现「斑块式」字号差异。
2. **字体变化时表卡尺寸没有自适应**——字号变大后，表宽/行高仍按固定 `TABLE_WIDTH` / `FIELD_ROW_HEIGHT` / `TABLE_HEADER_HEIGHT` 计算，导致字段名/注释被截断或行内垂直空间不匹配。

本提案修复上述渲染一致性缺口，使表卡尺寸随 effective font size 自适应，并确保离屏表在缩放/字号变化后重新光栅。

## 变更类型

设计级 / 代码级修复。

## 变更范围

- **影响的需求文档**：无
- **影响的功能规格**：
  - `logos/resources/prd/2-product-design/1-feature-specs/core-01-editor-canvas.md` §5.10 表卡内容自适应宽度
  - `core-01-editor-canvas.md` §5.12 小缩放 LOD 渲染分档
  - `core-01-editor-canvas.md` §5.16 画布标签字号
- **影响的业务场景**：S01 编辑器画布（core-S01）
- **影响的部署方案**：`logos/resources/prd/3-technical-plan/3-deployment/core-01-deployment-plan.md` 部署注记
- **影响的 API**：无
- **影响的 DB 表**：无 schema 变更；`Table.width` / `Table.min_height` 语义不变，仅渲染侧 effective size 计算口径调整
- **影响的编排测试**：无新增场景文件
- **影响的单元/场景测试**：`logos/resources/test/core-CR-canvas-test-cases.md`
- **影响的 smoke 测试**：`logos/resources/test/smoke/core-smoke-test-cases.md` 中 SMOKE-core-STABLE-01 全量回归即可，不新增用例

## 部署影响

- **是否需要部署**：是
- **部署原因**：纯前端画布渲染逻辑变更，前端产物随镜像；无 DB migration、无 API 变更。
- **影响环境**：staging / 生产
- **是否涉及数据迁移**：否
- **是否需要回滚预案**：是（回退上一镜像即可）
- **是否需要 smoke**：是（既有套件全量回归）

## 变更概述

1. **表卡 effective 尺寸随 effective font size 自适应**：
   - `compute_table_render_size_for` 增加 `zoom`、`label_font_scale`、`tier` 参数；字段维度下字段行高按 `scaled_label_font_world(11.0, zoom, label_scale)` 对应的有效字号动态计算；表头高按表名有效字号动态计算。
   - `estimate_content_width` 增加 `zoom`、`label_font_scale`、`tier` 参数；auto 宽度按 effective 字号重新估算，仍夹在 `[TABLE_WIDTH, TABLE_WIDTH_MAX]`。
   - 用户手动 `table.width > 0` 时尊重固定宽，但行高仍按 effective 字号自适应。

2. **命中/锚点/精灵缓存与绘制同口径**：
   - `resolve_table_width`、`field_anchor_for_side`、`anchor_for_tier` 统一使用带 effective 字号的 `resolve_table_width_for`。
   - 精灵缓存指纹 `table_sprite_fingerprint` 混入 effective font clamp bucket / label_font_scale 档位，确保 zoom 变化导致的字号变化会触发重光栅，消除离屏表与可见表的字号不一致。

3. **规格与测试**：
   - 在 `core-01-editor-canvas.md` 新增 R-WIDTH-06~08、R-FONT-07、R-LOD-09 等合同条款。
   - 新增 UT-CR-FONT-02（effective 字号下表宽/行高自适应）、ST-CR-FONT-02（e2e：缩小到全景后所有表卡字体/尺寸一致）。
   - 更新部署方案 delta：纯前端镜像重建，既有 smoke 回归。
