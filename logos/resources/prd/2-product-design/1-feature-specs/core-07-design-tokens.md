# Design Tokens 规格（E1）

## 1. 概述

本规格定义 drawdb-web 的完整设计 token 体系，对齐 main 分支（Semi Design）色阶与阴影层级，作为 E1–E6 视觉重写的根。E1 之前的 `--cdb-color-primary` 等 18 个 token（见 `frontend-rs/src/styles.css` L16–L59）保留为**子集**，E1 在此基础上扩展为完整 13 类约 100 个 token。

**所有 token 必须以 `--cdb-` 前缀命名**（coldrawdb 命名空间），命名风格：`--cdb-{category}-{name}[-{state}]`。

## 2. 主色色阶（teal，对齐 main `defaultBlue = #175e7a`）

| Token | 值 | 用途 |
|---|---|---|
| `--cdb-color-primary` | `#175e7a` | AppBar 主操作、激活态、链接、聚焦边框 |
| `--cdb-color-primary-hover` | `#134c63` | 按钮 hover |
| `--cdb-color-primary-active` | `#0e3a4d` | 按钮 active / 按下 |
| `--cdb-color-primary-disabled` | `#a3bcc6` | 禁用态 |
| `--cdb-color-primary-soft` | `#e6f1f5` | 浅色背景（Tag / Badge / 选中高亮） |
| `--cdb-color-primary-soft-hover` | `#d2e4ec` | 浅色背景 hover |
| `--cdb-color-primary-on` | `#ffffff` | 主色按钮上的文字色（反白） |

## 3. 中性色阶（grey，对齐 main `--semi-grey-0..9`）

| Token | 值 | 用途 |
|---|---|---|
| `--cdb-color-grey-0` | `#ffffff` | 纯白（modal/popover 背景） |
| `--cdb-color-grey-1` | `#f9fafb` | toolbar / toolbar-theme |
| `--cdb-color-grey-2` | `#f3f4f6` | 画布背景、hover-2 背景 |
| `--cdb-color-grey-3` | `#e5e7eb` | 边框、scrollbar thumb hover |
| `--cdb-color-grey-4` | `#d1d5db` | 占位符、disabled 文字 |
| `--cdb-color-grey-5` | `#9ca3af` | 图标 muted |
| `--cdb-color-grey-6` | `#6b7280` | 次要文字 |
| `--cdb-color-grey-7` | `#4b5563` | 正文文字（与 `--cdb-color-text` 合并） |
| `--cdb-color-grey-8` | `#374151` | 标题 |
| `--cdb-color-grey-9` | `#1f2937` | 标题强调（与 `--cdb-color-text` 合并） |

## 4. 语义色（success / warning / error / info）

| Token | 值 | 用途 |
|---|---|---|
| `--cdb-color-success` | `#10b981` | 成功提示 |
| `--cdb-color-success-soft` | `#d1fae5` | Tag success 浅色背景 |
| `--cdb-color-warning` | `#f59e0b` | 警告（Issues 徽章） |
| `--cdb-color-warning-soft` | `#fef3c7` | Tag warning 浅色背景 |
| `--cdb-color-error` | `#ef4444` | 错误（删除按钮、表单验证） |
| `--cdb-color-error-soft` | `#fee2e2` | Tag error 浅色背景 |
| `--cdb-color-info` | `#3b82f6` | 信息提示 |
| `--cdb-color-info-soft` | `#dbeafe` | Tag info 浅色背景 |

## 5. 文字色（对齐 main `--semi-color-text-0..4`）

| Token | 值 | 用途 |
|---|---|---|
| `--cdb-color-text-0` | `#1f2937` | 标题、主要正文 |
| `--cdb-color-text-1` | `#374151` | 次要正文（main `text-color`） |
| `--cdb-color-text-2` | `#6b7280` | 辅助文字、占位符（main `text-color` muted） |
| `--cdb-color-text-3` | `#9ca3af` | 禁用文字 |
| `--cdb-color-text-on-primary` | `#ffffff` | 主色按钮反白 |

## 6. 背景色（对齐 main `--semi-color-bg-0..3`）

| Token | 值 | 用途 |
|---|---|---|
| `--cdb-color-bg-0` | `#ffffff` | 主背景（modal / popover / card） |
| `--cdb-color-bg-1` | `#f9fafb` | toolbar / sidesheet（sidesheet-theme） |
| `--cdb-color-bg-2` | `#f3f4f6` | popover 嵌套层（popover-theme） |
| `--cdb-color-bg-3` | `#e5e7eb` | 画布背景（bg-canvas） |
| `--cdb-color-bg-overlay` | `rgba(0, 0, 0, 0.45)` | 模态遮罩 |

## 7. 边框色

| Token | 值 | 用途 |
|---|---|---|
| `--cdb-color-border` | `#e5e7eb` | 主边框（main `table-border`） |
| `--cdb-color-border-strong` | `#d1d5db` | 强调边框（hover 态） |
| `--cdb-color-border-muted` | `rgba(31, 41, 55, 0.08)` | 弱边框（main `border-color`） |

## 8. 阴影层级（5 档，对齐 Semi `shadow-1/2/3` + 扩展）

| Token | 值 | 用途 |
|---|---|---|
| `--cdb-shadow-xs` | `0 1px 2px rgba(0, 0, 0, 0.04)` | 内联标签 |
| `--cdb-shadow-sm` | `0 1px 3px rgba(0, 0, 0, 0.06), 0 1px 2px rgba(0, 0, 0, 0.04)` | AppBar / Card |
| `--cdb-shadow-md` | `0 4px 6px rgba(0, 0, 0, 0.07), 0 2px 4px rgba(0, 0, 0, 0.06)` | Dropdown / Popover |
| `--cdb-shadow-lg` | `0 10px 15px rgba(0, 0, 0, 0.1), 0 4px 6px rgba(0, 0, 0, 0.05)` | Modal / SideSheet |
| `--cdb-shadow-xl` | `0 20px 25px rgba(0, 0, 0, 0.1), 0 10px 10px rgba(0, 0, 0, 0.04)` | Command Palette（最高浮层） |

## 9. 动效（对齐 Semi 内置 transition）

| Token | 值 | 用途 |
|---|---|---|
| `--cdb-duration-fast` | `120ms` | hover / focus 反馈 |
| `--cdb-duration-base` | `200ms` | 按钮过渡、tooltip 显示 |
| `--cdb-duration-slow` | `300ms` | 模态 / 抽屉 / 命令面板 |
| `--cdb-easing-in` | `cubic-bezier(0.4, 0, 1, 1)` | 退出动画 |
| `--cdb-easing-out` | `cubic-bezier(0, 0, 0.2, 1)` | 进入动画 |
| `--cdb-easing-in-out` | `cubic-bezier(0.4, 0, 0.2, 1)` | 状态切换 |

## 10. 字体（对齐系统字体栈）

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

## 11. 圆角（4 档）

| Token | 值 | 用途 |
|---|---|---|
| `--cdb-radius-sm` | `4px` | Tag、徽章、checkbox |
| `--cdb-radius-md` | `6px` | 按钮、输入框 |
| `--cdb-radius-lg` | `8px` | 卡片、popover |
| `--cdb-radius-xl` | `12px` | Modal、SideSheet、CommandPalette |

## 12. 间距（4px grid 8 档）

| Token | 值 | 用途 |
|---|---|---|
| `--cdb-space-1` | `4px` | 内联 |
| `--cdb-space-2` | `8px` | 紧凑 |
| `--cdb-space-3` | `12px` | 通用 |
| `--cdb-space-4` | `16px` | 标准 |
| `--cdb-space-5` | `20px` | — |
| `--cdb-space-6` | `24px` | 章节 |
| `--cdb-space-8` | `32px` | 大块 |
| `--cdb-space-10` | `40px` | 模态内边距 |
| `--cdb-space-12` | `48px` | 大块留白 |

## 13. z-index 体系（5 层 + 模态/抽屉/浮层/命令面板）

| Token | 值 | 层级 | 内容 |
|---|---|---|---|
| `--cdb-z-base` | `1` | L0 | 默认 |
| `--cdb-z-canvas-overlay` | `10` | L1 | 画布选中框、连线 hover |
| `--cdb-z-app-bar` | `20` | L2 | AppBar、StatusBar |
| `--cdb-z-side-rail` | `25` | L2.5 | Tool Rail 悬浮按钮 |
| `--cdb-z-inspector` | `30` | L3 | Inspector 抽屉（与 IO 抽屉互斥） |
| `--cdb-z-drawer` | `30` | L3 | IO 抽屉 |
| `--cdb-z-tooltip` | `40` | L4 | Tooltip |
| `--cdb-z-popover` | `45` | L4.5 | Popover、Dropdown |
| `--cdb-z-modal` | `50` | L5 | Modal、命令面板居中浮层（CommandPalette） |
| `--cdb-z-notification` | `60` | L6 | Toast、Notification |

**互斥规则**：
- Inspector ↔ IO 抽屉：同层 L3，业务互斥（开抽屉时折叠 Inspector）
- Modal 打开时遮罩 `--cdb-z-modal - 1` = 49
- CommandPalette（E4）使用 `--cdb-z-modal`（L5）

## 14. 暗色模式 token 覆盖（`[data-mode="dark"]`）

> 完整规格见 `core-0b-dark-mode.md`（E5），本节仅约定覆盖接口

`E1` 阶段**只定义**覆盖接口（`[data-mode="dark"]` 选择器），不实现具体映射。E5 阶段填充具体值。

```css
:root {
  /* light 模式为默认 */
  --cdb-color-bg-0: #ffffff;
  /* ... 完整 light token ... */
}

[data-mode="dark"] {
  --cdb-color-bg-0: #16161a;  /* main darkBgTheme */
  --cdb-color-text-0: #f9fafb;
  /* ... E5 填充完整映射 ... */
}
```

## 15. 图标尺寸（R1）

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

## 15.1 Surface 语义色（R3，light 默认）

| Token | Light 值 | 用途 |
|---|---|---|
| `--cdb-color-canvas-grid` | `rgba(0, 0, 0, 0.04)` | 画布点阵网格线 |
| `--cdb-color-inspector-edge` | `rgba(15, 23, 42, 0.02)` | Inspector 左侧微妙分隔阴影 |
| `--cdb-color-focus-error` | `rgba(239, 68, 68, 0.1)` | 表单 invalid focus ring |
| `--cdb-color-error-hover-bg` | `rgba(239, 68, 68, 0.08)` | danger 按钮 hover 背景 |

> 组件层 **禁止** 直接使用 `#fef2f2` / `rgba(15,23,42,…)` 等字面量；语义背景统一使用 `--cdb-color-{semantic}-soft`（如 error-soft / warning-soft / info-soft）。

## 16. 主题切换接口

| 接口 | 类型 | 说明 |
|---|---|---|
| `<html data-mode="light|dark">` | DOM 属性 | 全局主题；JS 通过 `document.documentElement.setAttribute('data-mode', ...)` 切换 |
| `prefers-color-scheme: dark` | 媒体查询 | E5 阶段：用户未显式选择时跟随系统 |
| `localStorage["cdb-mode"]` | 持久化 | E5 阶段：用户选择覆盖系统偏好 |

## 17. 验收约束

- `frontend-rs/src/styles.css` 中 `var(--cdb-` 引用数 ≥ 100（E1 后）
- E1 delta 完成后，`grep -c '^--cdb-' frontend-rs/src/styles.css` ≥ 100
- 所有 token 必须出现在 `core-07-design-tokens.md` §2–§15 中
- Token 定义块（`:root` / `[data-mode="dark"]` / `@media`）之外，组件选择器不得出现 `#rgb` / `rgba(` 颜色字面量（`white-space` 等非颜色属性除外）
- Issue / Badge / Overlay / Canvas grid 必须使用语义 Token
- `prefers-color-scheme: dark` 须与 `[data-mode="dark"]` 保持同一套 Token 映射

## 18. 不在 E1 范围

- 暗色模式具体值（→ E5 `core-0b-dark-mode.md`）
- 动效曲线应用（→ E6 `core-0c-motion.md`）
- Token 在组件中的具体使用（→ E3 `core-09-core-components.md`）

