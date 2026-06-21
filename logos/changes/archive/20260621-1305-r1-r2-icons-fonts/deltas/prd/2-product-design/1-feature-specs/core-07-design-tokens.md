## MODIFIED — 10. 字体（对齐系统字体栈）

| Token | 值 | 用途 |
|---|---|---|
| `--cdb-font-family-base` | `"Plus Jakarta Sans", -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, "Helvetica Neue", Arial, "PingFang SC", "Hiragino Sans GB", "Microsoft YaHei", sans-serif` | 全局 UI（R2 落地，对齐 HTML 原型） |
| `--cdb-font-family-display` | `var(--cdb-font-family-base)` | Logo / 大标题 |
| `--cdb-font-family-mono` | `ui-monospace, SFMono-Regular, "SF Mono", Menlo, Consolas, "Liberation Mono", monospace` | Monaco / DBML / SQL |
| `--cdb-font-size-xs` | `11px` | 标签、徽章 |
| `--cdb-font-size-sm` | `12px` | 辅助文字 |
| `--cdb-font-size-base` | `14px` | 正文 |
| `--cdb-font-size-md` | `16px` | 标题 |
| `--cdb-font-size-lg` | `18px` | 大标题 |
| `--cdb-font-size-xl` | `24px` | 弹窗标题 |
| `--cdb-font-weight-regular` | `400` | 正文 |
| `--cdb-font-weight-medium` | `500` | 强调 |
| `--cdb-font-weight-semibold` | `600` | 标题 |
| `--cdb-font-weight-bold` | `700` | 重要提示 |
| `--cdb-line-height-tight` | `1.25` | 标题 |
| `--cdb-line-height-base` | `1.5` | 正文 |
| `--cdb-line-height-loose` | `1.75` | 长文本 |

> **R2 实现要求**：`index.html` 通过 Google Fonts 加载 Plus Jakarta Sans（400/500/600/700）；`body` 与表单控件必须使用 `var(--cdb-font-family-base)`。

## ADDED — 15. 图标尺寸（R1）

| Token | 值 | 用途 |
|---|---|---|
| `--cdb-icon-size-sm` | `16px` | AppBar 按钮、Modal 关闭、Undo/Redo |
| `--cdb-icon-size-md` | `20px` | ToolRail 工具按钮 |
| `--cdb-icon-size-lg` | `24px` | 空状态装饰（可选） |

**CSS 容器类**（`frontend-rs/src/styles.css`）：

| 类名 | 尺寸 Token |
|---|---|
| `.cdb-icon-wrap--sm` | `--cdb-icon-size-sm` |
| `.cdb-icon-wrap--md` | `--cdb-icon-size-md` |
| `.cdb-icon-wrap--lg` | `--cdb-icon-size-lg` |

SVG 内部 `width/height` 由容器 100% 撑满，`stroke="currentColor"` 继承按钮文字色。
