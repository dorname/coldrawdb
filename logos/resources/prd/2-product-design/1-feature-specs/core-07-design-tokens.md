# Design Tokens 规格（E1）

## 0. 事实基线与命名约定

唯一现行视觉事实基线：`logos/resources/prd/2-product-design/2-page-design/core-01-editor-prototype.html`。

- 主原型使用短名 CSS 变量（`--bg`、`--brand`、`--r-sm` 等）作为**可演示的事实值**。
- 生产实现可继续使用 `--cdb-*` 命名空间；映射时必须以主原型色值/层级/布局尺寸为准，不得回退到 Semi Design `#175e7a` / `#16161a` 等历史色阶。
- **禁止硬编码**要求保留：组件选择器不得出现与主题相关的 `#rgb` / `rgba(` 字面量（非颜色属性除外）；必须以 token `var(...)` 引用。例外仅限主原型中已标明的演示专用表面（如代码区 `#08171c`），生产侧应升格为 `--cdb-color-code-bg` 等语义 token。

## 1. 概述

本规格定义 coldrawdb 设计 token，对齐统一主原型的玻璃态工作空间（auth / rooms / room-editor），覆盖颜色、排版、间距、圆角、阴影、布局槽位、z-index 与响应式断点。E1–E6 历史 Semi 色阶仅作归档参考，**不再作为验收事实**。

## 2. 主色色阶（teal，对齐 main `defaultBlue = #175e7a`）

### Light（默认 `:root`）

| 主原型变量 | 事实值 | 生产建议映射 |
|---|---|---|
| `--bg` | `#eef5f7` | `--cdb-color-bg-page` |
| `--bg-deep` | `#dce9ee` | `--cdb-color-bg-deep` |
| `--surface` | `rgba(255,255,255,.72)` | `--cdb-color-surface` |
| `--surface-solid` | `#fff` | `--cdb-color-surface-solid` |
| `--surface-soft` | `rgba(248,252,253,.62)` | `--cdb-color-surface-soft` |
| `--surface-hover` | `rgba(255,255,255,.9)` | `--cdb-color-surface-hover` |
| `--line` | `rgba(49,78,88,.14)` | `--cdb-color-border` |
| `--line-strong` | `rgba(49,78,88,.24)` | `--cdb-color-border-strong` |
| `--text` | `#142c34` | `--cdb-color-text-0` |
| `--text-2` | `#506770` | `--cdb-color-text-2` |
| `--text-3` | `#7b8d93` | `--cdb-color-text-3` |
| `--brand` | `#1e8393` | `--cdb-color-primary` |
| `--brand-strong` | `#116477` | `--cdb-color-primary-strong` |
| `--brand-soft` | `rgba(30,131,147,.13)` | `--cdb-color-primary-soft` |
| `--accent` | `#7c5ce7` | `--cdb-color-accent` |
| `--green` | `#19a974` | `--cdb-color-success` |
| `--amber` | `#e59b24` | `--cdb-color-warning` |
| `--red` | `#e05858` | `--cdb-color-error` |
| `--blue` | `#3788e5` | `--cdb-color-info` |
| `--shadow` | `0 22px 70px rgba(32,67,78,.14)` | `--cdb-shadow-lg` |
| `--shadow-soft` | `0 10px 36px rgba(25,58,68,.1)` | `--cdb-shadow-md` |
| `--blur` | `blur(22px) saturate(145%)` | `--cdb-blur-glass` |

### Dark（`html[data-mode="dark"]`）

| 主原型变量 | 事实值 |
|---|---|
| `--bg` | `#050f13` |
| `--bg-deep` | `#08191f` |
| `--surface` | `rgba(16,38,45,.86)` |
| `--surface-solid` | `#10262d` |
| `--surface-soft` | `rgba(22,48,56,.78)` |
| `--surface-hover` | `rgba(34,68,78,.96)` |
| `--line` | `rgba(194,232,238,.16)` |
| `--line-strong` | `rgba(194,232,238,.30)` |
| `--text` | `#f2fdfe` |
| `--text-2` | `#b8d2d8` |
| `--text-3` | `#86a3ab` |
| `--brand` | `#5ee9dc` |
| `--brand-strong` | `#8cf0e6` |
| `--brand-soft` | `rgba(79,209,197,.18)` |
| `--accent` | `#b9a0ff` |
| `--green` | `#5ee2aa` |
| `--amber` | `#f5c45c` |
| `--red` | `#ff8a8a` |
| `--blue` | `#7ab8f5` |
| `--shadow` | `0 24px 90px rgba(0,0,0,.45)` |
| `--shadow-soft` | `0 12px 42px rgba(0,0,0,.32)` |

暗色 Primary 按钮文字使用近黑 `#050f13`（与 `--bg` 同阶），不得再使用浅色模式的纯白反色假定。

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

### 9.1 Spring 与 Focus Token（R6）

| Token | 值 | 用途 |
|---|---|---|
| `--cdb-easing-spring` | `cubic-bezier(0.34, 1.56, 0.64, 1)` | Inspector / IO Drawer / 溢出菜单 spring 入场 |
| `--cdb-color-focus-ring` | `rgba(23, 94, 122, 0.35)` | 按钮 / Tab / ToolRail 焦点环色 |
| `--cdb-shadow-focus` | `0 0 0 3px var(--cdb-color-focus-ring)` | `:focus-visible` 外环 |

暗色模式（`[data-mode="dark"]` 与 `prefers-color-scheme: dark`）覆盖：

| Token | 暗色值 |
|---|---|
| `--cdb-color-focus-ring` | `rgba(75, 163, 196, 0.45)` |

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

### 10.1 Web Font 加载契约（R3 — fix-canvas-hidpi-rendering 增量）

| ID | 约束 |
|---|---|
| F-WF-01 | Google Fonts URL 必须包含 `&display=optional`，避免 swap 期间降级闪烁影响画布栅格 |
| F-WF-02 | 首帧渲染（Canvas + DOM）必须在 `document.fonts.ready` resolve 后入队；超时 3000s 兜底后强制首帧 |
| F-WF-03 | `document.fonts.check("1em \"Plus Jakarta Sans\"")` 返回 false 时，画布 `CANVAS_FONT` 临时降级为 `ui-monospace, monospace`，避免 fill_text 期间字体未注册导致 0 宽渲染 |
| F-WF-04 | Canvas 字号按 DPR 上浮：`dpr ≥ 1.5` 时 13→14、11→12、10→11、9→10（仅画布文字，DOM 字号不变） |

### 10.2 字号 DPR 系数表

| Token | dpr=1 | dpr=1.5 | dpr=2 |
|---|---|---|---|
| `--cdb-font-size-base`（14） | 14 | 14 | 14 |
| `--cdb-font-size-md`（16） | 16 | 16 | 16 |
| `--cdb-font-size-lg`（18） | 18 | 18 | 18 |
| 画布表头（13） | 13 | 14 | 14 |
| 画布字段名（11） | 11 | 12 | 12 |
| 画布 type 标签（10） | 10 | 11 | 11 |
| 画布 PK 标记（9） | 9 | 10 | 10 |

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

| 层 | 主原型 z | 内容 |
|---|---|---|
| L0 画布关系 | `1` | `.relation-layer` |
| L1 画布对象 | `2`–`3` | 便签/区域、表 |
| L1.5 远端光标 | `7` | `.remote-cursor` |
| L1.6 画布顶栏 | `8` | `.canvas-top` |
| L1.7 Banner | `12` | `.banner` |
| L2 Status / Rooms nav | `20` | `.statusbar`、`.rooms-nav` |
| L2.5 ToolRail | `25` | `.toolrail` |
| L3 Inspector | `30` | `.inspector` |
| L3.5 Drawer | `35`（移动端 Inspector `36`） | `.drawer` |
| L4 AppBar | `40` | `.appbar` |
| L4.5 Popover | `46` | `.popover` |
| L4.6 Tooltip | `50`（ToolRail tip） | `.tool-tip` |
| L5 Overlay/Modal | `50` | `.overlay` |
| L5.5 Command | `55` | `.command` |
| L6 Toast | `60` | `.toast-region` |

生产 `--cdb-z-*` 须与上表语义对齐；不得再以「Inspector=IO 抽屉同层 30」覆盖主原型 Drawer=`35` 的事实。

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

## 15.2 AppBar 分区间距（R4）

| Token | 值 | 用途 |
|---|---|---|
| `--cdb-app-bar-gap` | `var(--cdb-space-3)` | AppBar 主 flex 分区间距（12px） |
| `--cdb-app-bar-brand-gap` | `var(--cdb-space-2)` | 品牌区内控件间距（8px） |
| `--cdb-app-bar-actions-gap` | `var(--cdb-space-2)` | 操作区按钮间距（8px） |

> R4 禁止 AppBar 使用非网格魔法数（如 `gap: 6px; padding: 4px` 的 IO pill 容器）；溢出菜单复用 `--cdb-z-popover` 层级。

## 15.3 Inspector Tab 栅格（R5）

| Token | 值 | 用途 |
|---|---|---|
| `--cdb-inspector-tab-size` | `36px` | 图标 Tab 单元格高度 |
| `--cdb-inspector-tab-gap` | `var(--cdb-space-1)` | 4×2 栅格间距（4px） |
| `--cdb-inspector-tab-columns` | `4` | Tab 栏列数 |

> R5 Tab 栏使用 `.cdb-tabs--icon-grid`；禁止 Inspector 内 Tab 使用非网格 `padding: 4px 8px` 文字换行。

## 16. 主题切换接口

| 接口 | 类型 | 说明 |
|---|---|---|
| `<html data-mode="light|dark">` | DOM 属性 | 全局主题；JS 通过 `document.documentElement.setAttribute('data-mode', ...)` 切换 |
| `prefers-color-scheme: dark` | 媒体查询 | E5 阶段：用户未显式选择时跟随系统 |
| `localStorage["cdb-mode"]` | 持久化 | E5 阶段：用户选择覆盖系统偏好 |

## 17. 验收约束

- auth / rooms / room-editor 全页使用同一套 light/dark token；切换 `data-mode` 不得出现未映射表面
- `grep` 组件层硬编码颜色字面量须为 0（演示升格 token 除外）
- 暗色 `--bg` 必须为 `#050f13`（或其 `--cdb-*` 等价映射）
- z-index 与断点行为与主原型一致；Playwright / 人工对照以 `core-01-editor-prototype.html` 为准

## 18. 不在 E1 范围

- 暗色模式具体值（→ E5 `core-0b-dark-mode.md`）
- 动效曲线应用（→ E6 `core-0c-motion.md`）
- Token 在组件中的具体使用（→ E3 `core-09-core-components.md`）

## 排版、圆角、间距与布局槽位

| 主原型 | 事实值 | 用途 |
|---|---|---|
| `font-family` | `Inter,"SF Pro Display","PingFang SC","Microsoft YaHei",system-ui,sans-serif` | 全局 UI |
| 等宽 | `ui-monospace,SFMono-Regular,Consolas,monospace` | 代码区 / 邀请链接 |
| `--r-sm` / `--r-md` / `--r-lg` | `10px` / `15px` / `22px` | 按钮 / 卡片 / 大面板 |
| `--ease` | `cubic-bezier(.2,.8,.2,1)` | 默认过渡 |
| `--appbar` | `64px`（≤760px 时 `58px`） | AppBar 行高 |
| `--status` | `34px` | StatusBar |
| `--rail` | `64px`（≤760px 时 `54px`） | ToolRail |
| `--inspector` | `330px`（≤1179px 折叠为浮层） | Inspector 宽 |

按钮基准：`min-height: 38px`、内边距 `0 14px`、图标间距 `8px`。Tag：`min-height: 24px`、字号 `11px`、胶囊圆角 `999px`。

## 响应式断点（主原型）

| 断点 | 行为摘要 |
|---|---|
| `max-width: 1179px` | Inspector 改为绝对浮层；隐藏房间徽章与 revision 长文案 |
| `max-width: 760px` | Auth 单列；AppBar 精简；ToolRail 底部横排；Drawer 全宽；远端光标 transition 关闭 |
| `@supports not backdrop-filter` | `.glass` 回退为 `--surface-solid` |
| `prefers-reduced-motion: reduce` | 动画/过渡压至 `0.01ms`（详见 `core-0c-motion.md`） |

## 历史 Semi / main 表述（不再作为现行事实）

以下内容不再作为现行规格事实（可保留历史备注，但验收以主原型为准）：

- 主色 `#175e7a` 及 Semi grey-0..9 / darkBgTheme `#16161a` 色阶表作为唯一真值
- 「对齐 main 分支 Semi Design」作为 E1 验收前提
- Plus Jakarta Sans + Google Fonts 加载要求（主原型为 Inter / 系统栈）
- 旧 4px 网格强制与主原型布局槽位冲突时，以主原型槽位优先
