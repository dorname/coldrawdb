# Icon Library 规格（E2）

## 0. 事实基线

唯一现行图标事实基线：`core-01-editor-prototype.html` 内联 `<svg class="sr-only" aria-hidden="true">` + `<symbol id="i-*">`，通过 `<svg class="icon"><use href="#i-{name}"></use></svg>` 引用。

生产前端可用 Leptos 组件或 sprite 等价实现，但**语义 ID、线宽风格、尺寸档与 a11y 规则**必须对齐主原型，不得以 emoji / Unicode 装饰符作为功能图标。

## 1. 概述

Icon Library 定义工作空间（auth / rooms / editor）功能图标集：内联 SVG symbol，无第三方图标包依赖要求。品牌字标（如 brand-mark 内 logo）可保留矢量符号；**禁止**用 🔍🗑✎⋯ 等 emoji 充当 ToolRail / AppBar / Modal / Toast 图标。

## 2. 命名规范

| 规则 | 约定 |
|---|---|
| 前缀 | `Icon`（与 main 一致） |
| 命名 | PascalCase，**动宾或名词**：`IconAddTable` / `IconEdit` / `IconChevronDown` |
| 文件 | 单文件 `icons.rs`，每个图标一个函数 `pub fn icon_add_table() -> impl IntoView` |
| 尺寸 | `<svg width={size} height={size}>`，默认 `size=16` |
| 颜色 | `stroke="currentColor"` / `fill="currentColor"`（继承父元素 `color`） |
| 线宽 | 默认 1.5（main `strokeWidth="1.5"`），粗体版 2 |

## 3. 组件签名

```rust
// frontend-rs/src/icons.rs
use leptos::*;

#[component]
pub fn Icon(
    #[prop(into)] path: String,         // SVG path d 属性
    #[prop(default = 16)] size: u32,
    #[prop(default = 1.5)] stroke_width: f32,
    #[prop(default = "currentColor")] color: &'static str,
    #[prop(default = "none")] fill: &'static str,
    #[prop(optional)] class: Option<&'static str>,
    #[prop(optional)] style: Option<String>,
) -> impl IntoView {
    view! {
        <svg
            width=size
            height=size
            viewBox="0 0 26 26"
            fill=fill
            stroke=color
            stroke-width=stroke_width
            class=class
            style=style
        >
            <path d=path />
        </svg>
    }
}

// 50 个具体图标函数
pub fn icon_add_table() -> impl IntoView {
    view! { <Icon path="M4 2 L20 2 A4 4 0 0 1 22 4 L22 14 ...".to_string() /> }
}
```

**使用示例**：

```rust
use crate::icons::{IconAddTable, IconChevronDown};

view! {
    <button class="cdb-btn">
        <IconAddTable />
        <span>"新建表"</span>
    </button>
}
```

## 4. 50 个核心图标清单

### 4.1 基础操作（8 个）

| 图标 | 用途 | 替换原 emoji | semi-icons 源 |
|---|---|---|---|
| `IconAdd` | 通用添加（菜单） | `+` | `IconPlus` |
| `IconMinus` | 减少 | `−` | `IconMinus` |
| `IconClose` | 关闭（模态/抽屉） | `×` | `IconClose` |
| `IconEdit` | 编辑 | `✎` | `IconEdit` |
| `IconDelete` | 删除 | `🗑` | `IconDeleteStroked` |
| `IconMore` | 更多操作 | `⋯` | `IconMore` |
| `IconCheck` | 确认 | `✓` | `IconCheckboxTick` |
| `IconSearch` | 搜索（Command Palette） | `🔍` | `IconSearch` |

### 4.2 导航（8 个）

| 图标 | 用途 | semi-icons 源 |
|---|---|---|
| `IconChevronUp` | 上 | `IconChevronUp` |
| `IconChevronDown` | 下 | `IconChevronDown` |
| `IconChevronLeft` | 左 | `IconChevronLeft` |
| `IconChevronRight` | 右 | `IconChevronRight` |
| `IconCaretDown` | 下拉箭头 | `IconCaretdown` |
| `IconArrowLeft` | 返回 | `IconArrowLeft` |
| `IconArrowRight` | 前进 | `IconArrowRight` |
| `IconExternalLink` | 外链 | `IconLink` |

### 4.3 撤销重做 / 保存（4 个）

| 图标 | 用途 | semi-icons 源 |
|---|---|---|
| `IconUndo` | 撤销 | `IconUndo` |
| `IconRedo` | 重做 | `IconRedo` |
| `IconSave` | 保存 | `IconSaveStroked` |
| `IconShare` | 分享 | `IconShareStroked` |

### 4.4 画布对象（5 个，main 自建）

| 图标 | 用途 | 替换原 emoji | 接线位置 |
|---|---|---|---|
| `IconSelect` | 选择工具 | `↖` | ToolRail `tool-select` |
| `IconAdd` | 新建菜单 | `⊕` | ToolRail `tool-new-menu` |
| `IconRelationship` | 关系工具 | `🔗` | ToolRail `tool-relationship` |
| `IconPan` | 平移工具 | `✋` | ToolRail `tool-pan` |
| `IconAddTable` | 新建表（菜单项，可选） | — | tool-new-menu dropdown |

> **R1 验收**：`editor_panels.rs` 中 ToolRail / AppBar / StatusBar / IO Drawer 不得再使用 Emoji/Unicode 作为图标占位；Logo 字母 `C` 保留为品牌字标。

### 4.5 字段类型徽章（12 个，Phase 1a spec §3）

| 图标 | 用途 | semi-icons 源 |
|---|---|---|
| `IconKey` | 主键 | `IconKeyStroked` |
| `IconLink` | 外键 | `IconLinkStroked` |
| `IconIndex` | 索引 | semi `IconOrderedListStroked` |
| `IconUnique` | 唯一约束 | semi `IconAsteriskStroked` |
| `IconNotNull` | 非空 | semi `IconAlertStroked` |
| `IconString` | 字符串类型 | `text-orange-500` 标签替代 |
| `IconInt` | 整数类型 | `text-yellow-500` 标签替代 |
| `IconDecimal` | 小数类型 | `text-lime-500` 标签替代 |
| `IconBoolean` | 布尔类型 | `text-violet-500` 标签替代 |
| `IconDate` | 日期类型 | `text-cyan-500` 标签替代 |
| `IconEnum` | 枚举类型 | `text-sky-500` 标签替代 |
| `IconBinary` | 二进制类型 | `text-emerald-500` 标签替代 |

### 4.6 IO 抽屉 / 导出导入（5 个）

| 图标 | 用途 | semi-icons 源 |
|---|---|---|
| `IconImport` | 导入 | semi `IconImportStroked` |
| `IconExport` | 导出 | semi `IconExportStroked` |
| `IconCopy` | 复制（Code View） | `IconCopy` |
| `IconDownload` | 下载 | semi `IconDownloadStroked` |
| `IconUpload` | 上传 | semi `IconUploadStroked` |

### 4.7 Inspector 操作（4 个）

| 图标 | 用途 | semi-icons 源 |
|---|---|---|
| `IconMove` | 移动对象 | semi `IconMoveStroked` |
| `IconCopy` | 复制对象（同 4.6） | — |
| `IconColorPicker` | 颜色选择 | semi `IconPalette` |
| `IconLock` | 锁定 | semi `IconLockStroked` |

### 4.8 主题与设置（4 个）

| 图标 | 用途 | semi-icons 源 |
|---|---|---|
| `IconSun` | 浅色模式 | semi `IconSun` |
| `IconMoon` | 暗色模式（E5 切换按钮） | semi `IconMoon` |
| `IconSettings` | 设置 | semi `IconSettingStroked` |
| `IconHelp` | 帮助 | semi `IconHelpCircleStroked` |

## 5. IconBox 尺寸容器（R1）

`frontend-rs/src/icons.rs` 提供统一包装组件，避免在 UI 层硬编码像素：

```rust
#[component]
pub fn IconBox(
    #[prop(default = "sm")] size: &'static str,  // "sm" | "md" | "lg"
    children: Children,
) -> impl IntoView
```

| size prop | CSS 类 | 场景 |
|---|---|---|
| `"sm"` | `.cdb-icon-wrap--sm` (16px) | `.cdb-btn--icon`、Drawer 标题、Modal 关闭 |
| `"md"` | `.cdb-icon-wrap--md` (20px) | `.cdb-tool-btn` |
| `"lg"` | `.cdb-icon-wrap--lg` (24px) | EmptyGuide 装饰 |

## 6. R1 新增图标（画布工具）

| 图标 | 用途 | 替换原占位 |
|---|---|---|
| `IconSelect` | 选择工具 | `↖` |
| `IconSidebar` | Inspector 侧栏切换 | `☰` |

## 7. 尺寸规格

| 档 | 主原型类 | 尺寸 | 用途 |
|---|---|---|---|
| sm | `.icon--sm` | `15×15` | 按钮内辅助、Toast、菜单项 |
| md | `.icon` 默认 | ~`18–20` | ToolRail / 常规按钮 |
| lg | `.icon--lg` | `22×22` | 品牌 / 空态装饰 |

- `viewBox="0 0 24 24"`；描边图标继承 `currentColor`。
- 填充型（如 `i-more` 圆点）允许 `fill="currentColor"`。
- 装饰性 SVG（aurora、symbol sprite 根节点）使用 `aria-hidden="true"`；功能按钮用 `aria-label`，图标本身 `aria-hidden="true"`。

## 8. 颜色继承

所有图标通过 `currentColor` 继承父元素 `color` CSS 属性：

```css
.cdb-btn { color: var(--cdb-color-text-0); }
.cdb-btn:hover { color: var(--cdb-color-primary); }
.cdb-btn--primary { color: var(--cdb-color-primary-on); background: var(--cdb-color-primary); }
```

**禁止在图标组件内硬编码颜色**——通过父元素 `color` 覆盖。

## 9. 暗色模式（E5 接入）

`currentColor` 继承机制保证 E5 阶段无需修改图标组件，只需切换 `color` 即可。

## 10. 验收约束

- 功能 UI 路径中 emoji / 装饰 Unicode 匹配数为 0（文案中的数学符号 −／＋ 缩放除外）
- 所有功能图标可追溯到 symbol ID 或生产等价组件
- 仅图标控件具备 `aria-label`；装饰 sprite 不进入可访问名

## 11. 不在 E2 范围

- 动画图标（loading spinner）— E6
- 自定义图标上传（用户上传 SVG）— V2+
- 图标包版本管理（lockfile）— 50 个 SVG 内联在 icons.rs，无外部依赖

## 主原型 symbol 清单与语义

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

## 无障碍（a11y）

- 仅图标按钮必须提供可读 `aria-label`（主原型：撤销、重做、主题、成员、代码、更多、关闭通知等）。
- 带可见文案的按钮可不重复朗读图标（图标 `aria-hidden`）。
- Toast 关闭按钮：`aria-label="关闭通知"`；全局 `#toast-region`：`aria-live="polite"`。
- Auth 密码切换：`aria-label` 在「显示密码 / 隐藏密码」间切换。
- 禁止依赖颜色 alone 传达状态；图标语义需与文案或 Tag 并存（如 Banner + wifi/info）。

## 历史 Semi-icons / emoji 表述（不再作为现行验收）

- 「从 `@douyinfe/semi-icons` 精选 ~50 个 path」不再作为唯一来源；主原型 symbol 优先。
- 「替换原 emoji」验收仍有效，但目标集以主原型 symbol 为准，而非 Semi 命名全集。
- 字段类型彩色文字标签若保留，不得冒充功能图标库条目。
