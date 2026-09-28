# Delta — core-PE-design-system-test-cases.md（fix-issues-34-35 / #34）

> 模块：core | 提案：fix-issues-34-35-header-comment-lod-anchor
> 目标：UT-PE-CMT-01 断言随 R-CMT-CONTRAST-01 双端点判据修订；表头色覆盖从蓝/绿/紫扩到橙/粉/紫/棕（#34 截图问题色）。不新增用例 ID。

## MODIFIED — UT-PE-CMT-01 — 注释对比度前景计算纯函数（#32；fix-issues-34-35 / #34 双端点判据）

- **位置**：`frontend-rs/src/editor_render.rs`（注释前景/衬底决策纯函数 `comment_foreground(header_tint, table_bg, ...)` / `needs_comment_chip(contrast)`）
- **断言**：
  1. 注释区有效背景（渐变段色）亮度高于阈值 → 选深色前景对；低于阈值 → 浅色前景对（与 R-COLOR-04 同阈值口径）
  2. 前景对对比度仍 < 4.5:1 → `needs_comment_chip` 返回 true（启用衬底/弱化渐变兜底）；≥ 4.5:1 → false
  3. **双端点 worst-case（#34）**：表头渐变场景下，输出前景（或前景+chip 组合）须在**满强度 tint 合成端**与**纯表体底色端**两端对比度同时 ≥ 4.5:1——对深色实色表头（棕/暗橙），亮段（衰减端）不得出现浅色字无衬底直绘
  4. 橙 / 粉 / 紫 / 棕（#34 截图问题色）+ 蓝 / 绿 表头色 × 亮 / 暗主题共 12 组输入，输出前景均满足双端 ≥ 4.5:1（含兜底生效情形）
  5. 字段行注释（tint 入参全透明）双端点退化为同一样本，输出与单端点行为一致（无回归）
  6. 空注释输入不产生任何注释渲染参数（空注释不占位不回归）

## MODIFIED — ID 登记表（OpenLogos ledger，更新行）

| ID | 层级 | 说明 |
|---|---|---|
| UT-PE-CMT-01 | UT | 注释对比度前景计算（渐变双端点 worst-case + 4.5:1 + 衬底兜底；橙/粉/紫/棕/蓝/绿 × 双主题） |
