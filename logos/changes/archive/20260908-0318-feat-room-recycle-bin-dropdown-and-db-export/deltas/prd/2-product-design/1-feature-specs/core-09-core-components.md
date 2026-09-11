# Delta: prd/2-product-design/1-feature-specs/core-09-core-components.md

> 提案：feat-room-recycle-bin-dropdown-and-db-export

## ADDED — 13. 表单下拉（Form Select）

> 提案：feat-room-recycle-bin-dropdown-and-db-export（修复原生 select「内容仿佛展示不出来」的视觉缺陷）

### 13.1 适用范围

所有原生 `<select>` 表单控件：`.cdb-form-select`（IO 抽屉引擎下拉等）与 `.cdb-select`（创建房间 dbtype 下拉等）。自绘菜单（`.cdb-menu-dropdown` / `.cdb-app-bar__overflow-menu`）不在本节范围。

### 13.2 视觉条款

1. **去原生外观**：`appearance: none`（含 `-webkit-`/`-moz-` 前缀），消除 OS 默认箭头与立体边框。
2. **自定义箭头**：右侧 chevron 使用内联 SVG 背景图（`background-image` + `background-repeat: no-repeat` + `background-position: right 10px center`），箭头颜色跟随 `currentColor` 语义（通过 `fill` 嵌入色值，亮暗两套）；为箭头预留 `padding-right: 32px`，文字不得与箭头重叠。
3. **显式配色**：必须显式声明 `color: var(--cdb-color-text)` 与 `background-color: var(--cdb-color-bg-secondary)`（或对应语义 token），不得依赖 UA 默认——暗色模式下未显式配色的 select 会渲染成白底白字。
4. **暗色适配**：`[data-mode="dark"]` 下 select 声明 `color-scheme: dark`（控制原生弹层配色），亮模式声明 `color-scheme: light`；`option` 元素显式声明前景/背景色，避免弹层内文字与背景同色。
5. **防裁切**：`line-height` 不得小于 1.4；`height` 不硬编码（由 padding + line-height 撑起，约 36px）；禁用会导致文字垂直裁切的 `padding`/`height` 组合。
6. **焦点**：`:focus-visible` 复用 `--cdb-shadow-focus`，与 `.cdb-btn` 焦点口径一致；`:disabled` 降透明度 0.5 且 `cursor: not-allowed`。
7. 弹层（原生 dropdown list）的展开高度/滚动由 OS 控制，样式无法干预；本节保证的是**关闭态控件本身**与**弹层配色**不再出现半隐/同色缺陷。

### 13.3 验收锚点

UT-PC-19（`include_str!` 锚点口径）断言 `styles.css` 中 `.cdb-form-select` 规则含：`appearance`、chevron `background-image`、`color-scheme`、`option` 配色、`padding-right`。
