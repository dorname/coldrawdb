## ADDED — 9.1 Spring 与 Focus Token（R6）

| Token | 值 | 用途 |
|---|---|---|
| `--cdb-easing-spring` | `cubic-bezier(0.34, 1.56, 0.64, 1)` | Inspector / IO Drawer / 溢出菜单 spring 入场 |
| `--cdb-color-focus-ring` | `rgba(23, 94, 122, 0.35)` | 按钮 / Tab / ToolRail 焦点环色 |
| `--cdb-shadow-focus` | `0 0 0 3px var(--cdb-color-focus-ring)` | `:focus-visible` 外环 |

暗色模式（`[data-mode="dark"]` 与 `prefers-color-scheme: dark`）覆盖：

| Token | 暗色值 |
|---|---|
| `--cdb-color-focus-ring` | `rgba(75, 163, 196, 0.45)` |

## MODIFIED — 17. 验收约束

在 R5 约束后追加：

- `:root` 含 `--cdb-easing-spring` 与 `--cdb-shadow-focus`
- 暗色块含 `--cdb-color-focus-ring` 映射
