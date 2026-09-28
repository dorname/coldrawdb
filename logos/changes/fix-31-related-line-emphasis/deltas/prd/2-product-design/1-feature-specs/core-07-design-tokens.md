# Delta — core-07-design-tokens.md（fix-31-related-line-emphasis / #31 提亮）

> 模块：core | 提案：fix-31-related-line-emphasis
> 目标：§15.4 highlight token 与实现对齐——`related-line-scale` 勘误为 1.5（与 `RELATED_RELATION_WIDTH_FACTOR` 同源）；新增相关线主色/光晕 token（强制选中色族提亮）。

## MODIFIED — §15.4 画布可读性 Token（fix-open-issues-26-33 / #31 / #32 / #30；fix-31-related-line-emphasis / #31 提亮）

> 画布 canvas 渲染不直接消费 CSS 变量，本节 token 作为 `editor_render.rs` palette 常量的**设计真值源**，命名与取值在此登记，渲染层对齐引用。

| Token | Light | Dark | 用途 |
|---|---|---|---|
| `canvas.comment.font-size-ratio` | 0.85 | 0.85 | 字段行注释字号 / 字段名字号（表头注释为 0.8× 且 ≥11px，见 R-CMT-CONTRAST-03） |
| `canvas.comment.font-weight` | 400 | 400 | 注释最小字重（禁止 300 细字） |
| `canvas.comment.min-contrast` | 4.5:1 | 4.5:1 | 注释前景/背景最小对比度（小字 WCAG AA 口径） |
| `canvas.comment.chip-bg-alpha` | 0.45 | 0.45 | 注释对比度兜底衬底 alpha（有效区间 0.35～0.55，R-CMT-CONTRAST-02） |
| `canvas.highlight.selected-ring-scale` | 2.0 | 2.0 | 选中表描边线宽 / 默认选中环线宽（≥2×，§4.4 强化） |
| `canvas.highlight.related-line-scale` | 1.5 | 1.5 | 相关关系线线宽 / 默认线宽（=1.5×；fix-31 勘误：原登记 2.0 与实现 `RELATED_RELATION_WIDTH_FACTOR=1.5` 不符，以实现为准统一） |
| `canvas.highlight.related-line-color` | selected | selected | 相关关系线主色 = `palette.selected`（light `#1e8393` / dark `#5ee9dc`；fix-31 / #31 提亮，替代原「沿用关系解析色」） |
| `canvas.highlight.related-line-halo` | selected_soft 8px | selected_soft 8px | 相关关系线光晕 = `palette.selected_soft`，宽 8px（选中关系自身 10px，保持层级区分） |
| `canvas.highlight.unrelated-line-alpha` | 0.25 | 0.25 | 非相关关系线不透明度上限 |
| `canvas.highlight.unrelated-table-alpha` | 0.5 | 0.5 | 非相关表不透明度建议值 |
| `canvas.lod.tier1-max-zoom` | 0.5 | 0.5 | LOD 拓扑档进入阈值（R-LOD-01），滞回 ±0.03 |
| `canvas.lod.min-screen-font-px` | 11 | 11 | 拓扑档表名屏幕最小字号（R-LOD-03） |
| `canvas.lod.line-min-screen-px` | 1.5 | 1.5 | 拓扑档关系线屏幕最小线宽（R-LOD-04） |

- 亮/暗主题同值时仍分别登记，便于后续按主题微调；
- 既有 `palette.selected` / `text_strong` / `text_muted` 语义不变，本节只新增可读性参数；
- `related-line-color` / `related-line-halo` 为语义引用（取值跟随 `palette.selected` / `palette.selected_soft`），不复制色值散值。
