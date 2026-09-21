# 变更提案：fix-table-header-contrast

> module: core | created: 2026-09-20
> 关联远程仓库：https://github.com/dorname/coldrawdb/issues/24

## 变更原因

GitHub Issue #24：画布表节点表头在浅色自定义色 / 浅色渐变下，表名旁注与次要文字仍使用主题浅色前景，对比度过低几乎不可读；用户自选背景色与字体色接近时同样失败。

代码对照证据（当前工作树 `frontend-rs/src/editor_render.rs`）：

| 位置 | 行为 | 问题 |
|------|------|------|
| `draw_table_body` ~L3891–3906 | 表头渐变取 `table.color`（非空）或 `palette.header_tint` | 背景可任意浅/深 |
| 同函数 ~L3912 / L3921 / L3930 | 表名 / 旁注 / 字段计数固定 `palette.text_strong` / `text_muted` | **不随表头实际亮度切换** |
| `PALETTE_DARK` | `text_strong=#f2fdfe`、`text_muted=#86a3ab` | 浅色 `table.color` 上近白字 → 不可读 |
| `PALETTE_LIGHT` | 深色字在极浅旁注色上仍可能对比不足 | 背景与字体过近时同样失败 |

根因：表头前景色仅绑全局主题调色板，未相对表头有效背景做亮度自适应。

## 变更类型

设计级变更（补渲染合同 + 测试用例 + 代码；无 API/DB/部署）

## 变更范围

- 影响的需求文档：无（存量可读性缺陷，不改产品 Why，不新增场景编号）
- 影响的功能规格：
  - `prd/2-product-design/1-feature-specs/core-01-editor-canvas.md`（§5.8 增补 R-COLOR-04；修订 R-CMT-01 表头文字取色口径）
  - `prd/2-product-design/1-feature-specs/core-01a-table-and-field.md`（§1.6 表色渲染交叉引用对比度合同，如有必要）
- 影响的业务场景：S01（画布编辑）；无新场景编号
- 影响的部署方案：无
- 影响的 API：无
- 影响的 DB 表：无
- 影响的编排测试：无
- 影响的测试用例：`test/core-CR-canvas-test-cases.md`（新增 UT-CR-COLOR-03；可选 ST 备注）
- 影响的 smoke 测试：无

## 部署影响

- 是否需要部署：否
- 部署原因：本地可验证的规格 + 前端画布渲染修复；生产部署由后续 release 变更单独执行
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
    description: "表头文字相对表头有效背景自适应深/浅前景，保证浅色表色与自定义近字体色下可读"
```

## 变更概述

在画布表头绘制路径引入**相对表头有效背景的前景自适应**：解析 `header_tint`（`table.color` 或主题 tint）并与表体底色合成后计算相对亮度；亮度高于阈值时使用深色强/次文本，否则使用浅色强/次文本（可复用 `PALETTE_LIGHT` / `PALETTE_DARK` 的 text 对）。默认半透明主题 tint 在合成后仍应与当前主题可读性一致。

同步更新 §5.8 渲染合同（R-COLOR-04）与 `core-CR-canvas` 单测，覆盖：暗主题 + 浅灰/白表色、亮主题 + 浅表色、表色接近主题字色等用例。字段行仍绘于表体底色之上，本变更仅约束**表头**文字（表名、旁注、字段计数）；字段行对比度若不达标另立 issue。
