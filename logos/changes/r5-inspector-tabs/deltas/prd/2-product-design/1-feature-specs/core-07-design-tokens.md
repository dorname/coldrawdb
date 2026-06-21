## ADDED — 15.3 Inspector Tab 栅格（R5）

| Token | 值 | 用途 |
|---|---|---|
| `--cdb-inspector-tab-size` | `36px` | 图标 Tab 单元格高度 |
| `--cdb-inspector-tab-gap` | `var(--cdb-space-1)` | 4×2 栅格间距（4px） |
| `--cdb-inspector-tab-columns` | `4` | Tab 栏列数 |

> R5 Tab 栏使用 `.cdb-tabs--icon-grid`；禁止 Inspector 内 Tab 使用非网格 `padding: 4px 8px` 文字换行。

## MODIFIED — 17. 验收约束

在 R4 约束后追加：

- Inspector Tab 栏必须为 `.cdb-tabs--icon-grid`（4 列 × 2 行）
- 8 个 `tab-*` testid 均存在（含 `tab-fields`）
- 不得存在 `.cdb-side-panel--right` 45% 高度分割
- `field-editor` 仅在 `tab-fields` 内容区渲染
