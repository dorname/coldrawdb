# Delta — core-07-design-tokens.md（fix-open-issues-26-33 / #31 + #32 + #33）

> 模块：core | 提案：fix-open-issues-26-33
> 目标：
> - #31/#32：画布可读性相关渲染参数 token 化。
> - #33：图表视觉体系统一 token——表卡、语义图标、关系端点、ToolRail 图标的设计真值登记。

## ADDED — §15.4 画布可读性 Token（fix-open-issues-26-33 / #31 / #32 / #30）

> 画布 canvas 渲染不直接消费 CSS 变量，本节 token 作为 `editor_render.rs` palette 常量的**设计真值源**，命名与取值在此登记，渲染层对齐引用。

| Token | Light | Dark | 用途 |
|---|---|---|---|
| `canvas.comment.font-size-ratio` | 0.85 | 0.85 | 字段行注释字号 / 字段名字号（表头注释为 0.8× 且 ≥11px，见 R-CMT-CONTRAST-03） |
| `canvas.comment.font-weight` | 400 | 400 | 注释最小字重（禁止 300 细字） |
| `canvas.comment.min-contrast` | 4.5:1 | 4.5:1 | 注释前景/背景最小对比度（小字 WCAG AA 口径） |
| `canvas.comment.chip-bg-alpha` | 0.45 | 0.45 | 注释对比度兜底衬底 alpha（有效区间 0.35～0.55，R-CMT-CONTRAST-02） |
| `canvas.highlight.selected-ring-scale` | 2.0 | 2.0 | 选中表描边线宽 / 默认选中环线宽（≥2×，§4.4 强化） |
| `canvas.highlight.related-line-scale` | 2.0 | 2.0 | 相关关系线线宽 / 默认线宽（≥1.5×，建议 2×） |
| `canvas.highlight.unrelated-line-alpha` | 0.25 | 0.25 | 非相关关系线不透明度上限 |
| `canvas.highlight.unrelated-table-alpha` | 0.5 | 0.5 | 非相关表不透明度建议值 |
| `canvas.lod.tier1-max-zoom` | 0.5 | 0.5 | LOD 拓扑档进入阈值（R-LOD-01），滞回 ±0.03 |
| `canvas.lod.min-screen-font-px` | 11 | 11 | 拓扑档表名屏幕最小字号（R-LOD-03） |
| `canvas.lod.line-min-screen-px` | 1.5 | 1.5 | 拓扑档关系线屏幕最小线宽（R-LOD-04） |

- 亮/暗主题同值时仍分别登记，便于后续按主题微调；
- 既有 `palette.selected` / `text_strong` / `text_muted` 语义不变，本节只新增可读性参数。

## ADDED — §15.5 图表视觉体系 Token（fix-open-issues-26-33 / #33）

> 目标：表卡 / 语义图标 / 关系端点 / ToolRail 图标形成一套统一视觉语言。以下为设计真值，主原型与前端渲染对齐引用；亮/暗主题成对设计。

| 类别 | Token | 取值 | 说明 |
|---|---|---|---|
| 表卡 | `canvas.table.corner-radius` | 8px | 表卡圆角统一值（现状多值并存收敛） |
| 表卡 | `canvas.table.shadow` | 复用 §8 shadow-1（hover shadow-2） | 表卡投影统一，不再自定义散值 |
| 表卡 | `canvas.table.header-gradient` | `linear-gradient(135deg, color → color@0.82)` | 表头渐变方向与止境统一；注释区按 R-CMT-CONTRAST-02 可弱化为近实色 |
| 表卡 | `canvas.table.field-row-height` | 24px | 字段行高统一（含 LOD 详情档） |
| 语义图标 | `canvas.badge.size` | 12px（LOD 详情档） | PK / FK / NOT NULL / UNIQUE 徽章统一外接尺寸 |
| 语义图标 | `canvas.badge.stroke` | 1.5px | 徽章图标描边统一 |
| 语义图标 | `canvas.badge.palette` | PK=`semantic.warning` 系 / FK=`semantic.info` 系 / NN·UQ=`grey.6` | 徽章色板统一，亮/暗成对 |
| 关系端点 | `canvas.rel.endpoint-style` |  crow's foot（基数记号）统一集 | 一端/多端记号同一几何族，禁止混用箭头与字块角标 |
| 关系端点 | `canvas.rel.endpoint-size` | 10px | 端点记号外接尺寸 |
| ToolRail | `toolrail.icon.stroke` | 1.5px | ToolRail 全部图标描边统一（对齐 §15 R1 尺寸容器） |
| ToolRail | `toolrail.icon.size` | 20px（IconBox 24px） | 激活态 = 主色描边 + 浅色底（`primary.1` 底 / `primary.6` 图标） |

- 落地顺序：先更新主原型 `core-01-editor-prototype.html` 与本 token 表 → 再改 `editor_render.rs` canvas 绘制与图标组件；
- 不引入新主题、不改动 §2～§6 既有色阶；徽章/端点色仅从既有语义色阶取值。
