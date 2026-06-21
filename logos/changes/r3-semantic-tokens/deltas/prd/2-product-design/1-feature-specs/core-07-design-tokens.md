## ADDED — 15.1 Surface 语义色（R3，light 默认）

| Token | Light 值 | 用途 |
|---|---|---|
| `--cdb-color-canvas-grid` | `rgba(0, 0, 0, 0.04)` | 画布点阵网格线 |
| `--cdb-color-inspector-edge` | `rgba(15, 23, 42, 0.02)` | Inspector 左侧微妙分隔阴影 |
| `--cdb-color-focus-error` | `rgba(239, 68, 68, 0.1)` | 表单 invalid focus ring |
| `--cdb-color-error-hover-bg` | `rgba(239, 68, 68, 0.08)` | danger 按钮 hover 背景 |

> 组件层 **禁止** 直接使用 `#fef2f2` / `rgba(15,23,42,…)` 等字面量；语义背景统一使用 `--cdb-color-{semantic}-soft`（如 error-soft / warning-soft / info-soft）。

## MODIFIED — 17. 验收约束

- 组件选择器区域（`:root` / `[data-mode="dark"]` / `@media` 块之外）不得出现 `#rgb` / `rgba(` 字面量（`white-space` 等非颜色属性除外）
- Issue / Badge / Overlay / Canvas grid 必须使用语义 Token
- `prefers-color-scheme: dark` 须与 `[data-mode="dark"]` 保持同一套 Token 映射
