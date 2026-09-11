## MODIFIED — 3. CSS 实现

`@media (prefers-color-scheme: dark)` 内 `:root:not([data-mode="light"]):not([data-mode="dark"])` **必须** 与 `[data-mode="dark"]` 块包含相同 Token 集合（primary / grey / semantic / text / bg / border / shadow / surface），不得仅覆写 3 个 Token。

新增 surface Token 暗色映射：

| Token | Dark 值 |
|---|---|
| `--cdb-color-canvas-grid` | `rgba(255, 255, 255, 0.06)` |
| `--cdb-color-inspector-edge` | `rgba(255, 255, 255, 0.04)` |
| `--cdb-color-focus-error` | `rgba(248, 113, 113, 0.2)` |
| `--cdb-color-error-hover-bg` | `rgba(248, 113, 113, 0.12)` |
