# Delta — core-07-design-tokens.md（修改）

> module: core | proposal: align-all-docs-to-unified-prototype

## ADDED — 0. 事实基线与命名约定

唯一现行视觉事实基线：`logos/resources/prd/2-product-design/2-page-design/core-01-editor-prototype.html`。

- 主原型使用短名 CSS 变量（`--bg`、`--brand`、`--r-sm` 等）作为**可演示的事实值**。
- 生产实现可继续使用 `--cdb-*` 命名空间；映射时必须以主原型色值/层级/布局尺寸为准，不得回退到 Semi Design `#175e7a` / `#16161a` 等历史色阶。
- **禁止硬编码**要求保留：组件选择器不得出现与主题相关的 `#rgb` / `rgba(` 字面量（非颜色属性除外）；必须以 token `var(...)` 引用。例外仅限主原型中已标明的演示专用表面（如代码区 `#08171c`），生产侧应升格为 `--cdb-color-code-bg` 等语义 token。

## MODIFIED — 1. 概述

本规格定义 coldrawdb 设计 token，对齐统一主原型的玻璃态工作空间（auth / rooms / room-editor），覆盖颜色、排版、间距、圆角、阴影、布局槽位、z-index 与响应式断点。E1–E6 历史 Semi 色阶仅作归档参考，**不再作为验收事实**。

## MODIFIED — 2. 主色色阶（teal，对齐 main `defaultBlue = #175e7a`）

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

## ADDED — 统一原型对齐补充：排版 / 圆角 / 间距 / 布局槽位

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

## MODIFIED — 13. z-index 体系（5 层 + 模态/抽屉/浮层/命令面板）

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

## ADDED — 响应式断点（主原型）

| 断点 | 行为摘要 |
|---|---|
| `max-width: 1179px` | Inspector 改为绝对浮层；隐藏房间徽章与 revision 长文案 |
| `max-width: 760px` | Auth 单列；AppBar 精简；ToolRail 底部横排；Drawer 全宽；远端光标 transition 关闭 |
| `@supports not backdrop-filter` | `.glass` 回退为 `--surface-solid` |
| `prefers-reduced-motion: reduce` | 动画/过渡压至 `0.01ms`（详见 `core-0c-motion.md`） |

## ADDED — 统一原型对齐补充：以 Semi / main 为事实源的表述

以下内容不再作为现行规格事实（可保留历史备注，但验收以主原型为准）：

- 主色 `#175e7a` 及 Semi grey-0..9 / darkBgTheme `#16161a` 色阶表作为唯一真值
- 「对齐 main 分支 Semi Design」作为 E1 验收前提
- Plus Jakarta Sans + Google Fonts 加载要求（主原型为 Inter / 系统栈）
- 旧 4px 网格强制与主原型布局槽位冲突时，以主原型槽位优先

## MODIFIED — 17. 验收约束

- auth / rooms / room-editor 全页使用同一套 light/dark token；切换 `data-mode` 不得出现未映射表面
- `grep` 组件层硬编码颜色字面量须为 0（演示升格 token 除外）
- 暗色 `--bg` 必须为 `#050f13`（或其 `--cdb-*` 等价映射）
- z-index 与断点行为与主原型一致；Playwright / 人工对照以 `core-01-editor-prototype.html` 为准
