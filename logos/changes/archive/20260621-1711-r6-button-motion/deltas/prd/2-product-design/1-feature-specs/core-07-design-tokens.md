## MODIFIED — 9. 动效（对齐 Semi 内置 transition）

**merge 时在 §9 表格末尾追加**：

| `--cdb-easing-spring` | `cubic-bezier(0.34, 1.56, 0.64, 1)` | R6 面板 spring 入场（Inspector / IO Drawer / 溢出菜单） |
| `--cdb-shadow-focus` | `0 0 0 3px var(--cdb-color-primary-soft)` | R6 键盘焦点环（按钮 / ToolRail / Tab） |

## MODIFIED — 17. 验收约束

在 R5 约束后追加：

- 存在 `--cdb-easing-spring` 与 `--cdb-shadow-focus` Token 定义
- `styles.css` 仅一处 `@keyframes cdb-pulse`（scale）；保存圆点使用 `cdb-pulse-opacity`
