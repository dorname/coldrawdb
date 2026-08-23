# Delta — core-0b-dark-mode.md（修改）

> module: core | proposal: align-all-docs-to-unified-prototype

## ADDED — 0. 事实基线

唯一现行主题基线：`core-01-editor-prototype.html`。

- 根节点：`<html lang="zh-CN" data-mode="dark|light">`
- 默认演示为 **dark**；`document.documentElement.dataset.mode = state.theme` 同步
- 主题覆盖 **auth / rooms / room-editor（含 Modal、Drawer、Toast、Code）全页**，禁止仅编辑器局部换肤

## MODIFIED — 1. 概述

暗色模式通过 `data-mode` 切换整套表面 token（见 `core-07`）。目标：玻璃态高对比可读、WCAG AA 级正文/控件对比，且三态页面视觉语言一致。

## MODIFIED — 2. Token 暗色映射

| Token（主原型） | Dark 事实值 |
|---|---|
| `--bg` | `#050f13` |
| `--bg-deep` | `#08191f` |
| `--surface-solid` | `#10262d` |
| `--text` | `#f2fdfe` |
| `--brand` | `#5ee9dc` |
| `--accent` | `#b9a0ff` |

完整映射以 `core-07` dark 表为准。历史 Semi `darkBgTheme = #16161a` **移除为现行事实**。

## ADDED — 全页覆盖范围

| 页面 / 表面 | 要求 |
|---|---|
| auth | 故事区、表单、tabs、primary CTA、错误文案 |
| rooms | nav、房间卡、新建虚线卡、用户菜单 |
| room-editor | AppBar、ToolRail、画布表/关系、Inspector、StatusBar、Banner |
| 浮层 | Modal overlay、Popover、Command、Drawer、Toast |
| 代码 / 预览 | `.code-area` / `.preview` 使用暗色代码底（原型 `#061217`） |

主原型另有 `html[data-mode="dark"] …` 组件级增强（边框改 `--line-strong`、primary 按钮近黑字等），生产须等价实现或收进 token，避免漏表面。

## ADDED — 统一原型对齐补充：全页面主题切换入口

| 入口 | 行为 |
|---|---|
| rooms：主题按钮 | `aria-label="切换主题"`；dark↔light |
| editor：更多菜单 `btn-theme-toggle` | 同上 + Toast「主题已切换」 |
| 偏好设置 Modal | `<select>` 深色玻璃 / 浅色玻璃 |
| 命令面板 | 「切换主题」命令 |

生产可保留 Light / Dark / System 三态与 `localStorage["cdb-mode"]`；**未显式选择时**可跟随 `prefers-color-scheme`，但显式 `data-mode` 优先级更高。

## ADDED — WCAG AA

- 正文 `--text` 对 `--bg` / 实心表面对比度 ≥ 4.5:1
- 辅助 `--text-2` / `--text-3` 用于非关键文案；关键错误/按钮不得仅用 `--text-3`
- 暗色 primary 按钮：亮 brand 底 + 深字（`#050f13`），避免亮底亮字
- 焦点可见：输入/按钮 focus 使用 brand 边或 focus ring token
- 不依赖「仅色相」区分成功/错误（配合图标与文案）

## ADDED — 统一原型对齐补充：过时映射

- Light→Dark 表中以 `#4ba3c4` / `#16161a` / grey 反转为主的 Semi 映射作为唯一真值
- View 菜单 emoji（☀🌙💻）作为唯一入口文案 → 改用 SVG 图标 + 中文

## MODIFIED — 8. 验收约束

- 任意页面切换主题后，无未换肤白块/黑块
- `data-mode` 与可见主题一致
- AA：抽样 Auth 标题/正文、Primary 按钮、Banner 文案、Toast 标题
- Code View / Drawer preview 跟随暗色代码表面
- Monaco（若启用）主题与 `data-mode` 同步
