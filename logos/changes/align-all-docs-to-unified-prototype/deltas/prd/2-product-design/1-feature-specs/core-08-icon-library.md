# Delta — core-08-icon-library.md（修改）

> module: core | proposal: align-all-docs-to-unified-prototype

## ADDED — 0. 事实基线

唯一现行图标事实基线：`core-01-editor-prototype.html` 内联 `<svg class="sr-only" aria-hidden="true">` + `<symbol id="i-*">`，通过 `<svg class="icon"><use href="#i-{name}"></use></svg>` 引用。

生产前端可用 Leptos 组件或 sprite 等价实现，但**语义 ID、线宽风格、尺寸档与 a11y 规则**必须对齐主原型，不得以 emoji / Unicode 装饰符作为功能图标。

## MODIFIED — 1. 概述

Icon Library 定义工作空间（auth / rooms / editor）功能图标集：内联 SVG symbol，无第三方图标包依赖要求。品牌字标（如 brand-mark 内 logo）可保留矢量符号；**禁止**用 🔍🗑✎⋯ 等 emoji 充当 ToolRail / AppBar / Modal / Toast 图标。

## ADDED — 主原型 symbol 清单与语义

| Symbol ID | 语义 | 典型场景 |
|---|---|---|
| `i-logo` | 产品品牌 | brand-mark |
| `i-arrow` / `i-back` | 前进 / 返回 | 登录 CTA、退出 |
| `i-plus` | 添加 | 创建房间、邀请、加字段 |
| `i-table` | 数据表 | ToolRail、Inspector 空态 |
| `i-relation` | 关系 | 关系工具 / Banner 提示 |
| `i-note` / `i-area` | 便签 / 区域 | 画布工具（扩展） |
| `i-search` | 搜索 | 命令面板 |
| `i-undo` / `i-redo` | 撤销 / 重做 | AppBar |
| `i-share` | 分享 | 分享模态 |
| `i-users` | 成员 | 成员抽屉、邀请 |
| `i-code` | 代码视图 | AppBar `btn-code-view` |
| `i-more` | 更多菜单 | `btn-more-menu` |
| `i-close` / `i-check` | 关闭 / 确认 | Modal、Toast |
| `i-copy` | 复制 | 代码视图、邀请链接 |
| `i-download` / `i-upload` | 导出 / 导入 | IO 抽屉 |
| `i-sun` | 主题切换 | rooms nav / more 菜单 |
| `i-settings` | 设置 / Inspector | StatusBar 折叠 |
| `i-activity` | 动态 / 诊断 | 活动流、诊断 |
| `i-info` | 信息 / 警告 | Banner、错误 Toast |
| `i-trash` | 删除 | 删表 / 删字段 / 移除成员 |
| `i-eye` | 显示密码 | Auth |
| `i-bolt` | 演示 / 会话 | 协作模拟器 |
| `i-wifi` | 连接状态 | 重连 Banner |

生产命名可映射为 `IconTable` 等 PascalCase，但须维护与上表一对一的语义字典。

## MODIFIED — 7. 尺寸规格

| 档 | 主原型类 | 尺寸 | 用途 |
|---|---|---|---|
| sm | `.icon--sm` | `15×15` | 按钮内辅助、Toast、菜单项 |
| md | `.icon` 默认 | ~`18–20` | ToolRail / 常规按钮 |
| lg | `.icon--lg` | `22×22` | 品牌 / 空态装饰 |

- `viewBox="0 0 24 24"`；描边图标继承 `currentColor`。
- 填充型（如 `i-more` 圆点）允许 `fill="currentColor"`。
- 装饰性 SVG（aurora、symbol sprite 根节点）使用 `aria-hidden="true"`；功能按钮用 `aria-label`，图标本身 `aria-hidden="true"`。

## ADDED — 无障碍（a11y）

- 仅图标按钮必须提供可读 `aria-label`（主原型：撤销、重做、主题、成员、代码、更多、关闭通知等）。
- 带可见文案的按钮可不重复朗读图标（图标 `aria-hidden`）。
- Toast 关闭按钮：`aria-label="关闭通知"`；全局 `#toast-region`：`aria-live="polite"`。
- Auth 密码切换：`aria-label` 在「显示密码 / 隐藏密码」间切换。
- 禁止依赖颜色 alone 传达状态；图标语义需与文案或 Tag 并存（如 Banner + wifi/info）。

## ADDED — 统一原型对齐补充：以 Semi-icons / emoji 替换表为现行验收的表述

- 「从 `@douyinfe/semi-icons` 精选 ~50 个 path」不再作为唯一来源；主原型 symbol 优先。
- 「替换原 emoji」验收仍有效，但目标集以主原型 symbol 为准，而非 Semi 命名全集。
- 字段类型彩色文字标签若保留，不得冒充功能图标库条目。

## MODIFIED — 10. 验收约束

- 功能 UI 路径中 emoji / 装饰 Unicode 匹配数为 0（文案中的数学符号 −／＋ 缩放除外）
- 所有功能图标可追溯到 symbol ID 或生产等价组件
- 仅图标控件具备 `aria-label`；装饰 sprite 不进入可访问名
