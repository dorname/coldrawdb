# 变更提案：fix-table-auto-width-include-comment

> module: core | created: 2026-09-21
> 关联远程仓库：https://github.com/dorname/coldrawdb/issues/25

## 变更原因

GitHub #25：表/字段名称过长时画布表宽会自适应，但在默认「英文名+注释」（`NameComment`）模式下，**自适应宽度未把中文注释按并排占用计入**，注释被挤到类型旁并省略。

#20 已交付 `estimate_content_width` / `resolve_table_width`（R-WIDTH-01～05），但实现里对 secondary 注释使用 `max(...)`，与 `draw_table_body` 的并排布局（名称右缘 ↔ 类型左缘之间画注释）不一致。`measure_text_approx` 已按 CJK≈14px 计宽，根因是布局公式漏加，不是测宽算法。

## 变更类型

代码级修复（规格 R-WIDTH-01 已要求计入注释；本提案澄清「并排累加」测量口径，并补 UT；无 API/DB/部署变更）

## 变更范围

- 影响的需求文档：无
- 影响的功能规格：
  - `prd/2-product-design/1-feature-specs/core-01-editor-canvas.md` §5.10 — 澄清 R-WIDTH-01：NameComment 下表头/字段行为「主文本 + 间距 + 注释 +（字段）类型」横向累加，而非各段取 max
  - `prd/2-product-design/1-feature-specs/core-01a-table-and-field.md` §1.7 — 交叉引用测量口径
- 影响的业务场景：S01（画布编辑）；无新场景编号
- 影响的部署方案：无
- 影响的 API：无
- 影响的 DB 表：无
- 影响的编排测试：无
- 影响的测试用例：`test/core-CR-canvas-test-cases.md` — 新增 UT-CR-WIDTH-03（中文注释并排撑宽）
- 影响的 smoke 测试：无

## 部署影响

- 是否需要部署：否
- 部署原因：本地可验证的规格澄清 + 前端纯函数修复；生产部署由后续 release 变更单独执行
- 影响环境：本地
- 是否涉及数据迁移：否
- 是否需要回滚预案：否
- 是否需要 smoke：否

## UI/UX 变更声明

```yaml
ui_impact: true
design_system_mode: generated
design_system_fallback_reason: ""
pages:
  - id: editor
    prototype: core-01-editor-prototype.html
    description: "auto 表宽在 NameComment 下正确计入表/字段中文注释并排宽度；截断仍受 TABLE_WIDTH_MAX=480 约束"
```

## 变更概述

1. **规格**：在 R-WIDTH-01 补充测量公式——`NameComment` 且 secondary 非空时，表头/字段行需求宽为并排字符串宽度之和（含间距、PK 偏移、类型列），禁止用「仅注释宽」与「名+类型宽」取 max 代替累加；`Name` / `Comment`（无 secondary）行为不变。
2. **代码**：修正 `frontend-rs/src/editor_render.rs` 的 `estimate_content_width`；主键字段计入 PK 徽章水平偏移。
3. **测试**：新增 UT-CR-WIDTH-03（短英文名 + 中文注释 → NameComment 估算严格大于无注释；近似 ≥ 名+注释+类型之和）；登记 OpenLogos reporter；既有 UT-CR-WIDTH-01/02 不回归。

夹紧区间仍为 `[230, 480]`；手动正数 `table.width` 仍优先（R-WIDTH-02）。
