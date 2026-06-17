## MODIFIED — 顶部元数据剥离

> 模块：core | 提案：add-baseline-docs
> 路径：`logos/resources/prd/2-product-design/1-feature-specs/core-08-icon-library.md`
> 策略：移除文件开头的 `# Delta — xxx（新文件）` 包装块与紧随的 `## ADDED — 全文` 子块及其 `>` 元数据行，保留真实一级标题以下所有内容原样。

# Icon Library 规格（E2）

## 1. 概述

E2 把当前散落 drawdb-web 的 unicode emoji / 字符占位统一替换为 SVG 图标库。**E2 不引入第三方图标库**（不引入 `leptos-icons`），而是从 main `@douyinfe/semi-icons` 的命名与 SVG 路径中精选 ~50 个核心图标，转换为 Leptos 组件，**自建在 `frontend-rs/src/icons.rs`**。

**自建理由**：
- `leptos-icons` 等通用库版本漂移、与 Leptos 0.5 兼容性未验证
- main `@douyinfe/semi-icons` 是 React 专用，不可直接使用
- drawdb-web 实际需求 ~50 个，远小于 semi-icons 2000+ 个，复制 50 个 SVG path 成本极低

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

| 图标 | 用途 | main 源 |
|---|---|---|
| `IconAddTable` | 新建表（Tool Rail） | `src/icons/IconAddTable.jsx` |
| `IconAddArea` | 新建区域 | `src/icons/IconAddArea.jsx` |
| `IconAddNote` | 新建便签 | `src/icons/IconAddNote.jsx` |
| `IconRelationship` | 关系工具 | semi `IconLinkStroked` |
| `IconPan` | 平移工具 | semi `IconHandDragStroked` |

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

## 5. 尺寸规格

| 场景 | size | stroke-width | 备注 |
|---|---|---|---|
| AppBar 按钮 | 16 | 1.5 | 32px 按钮内 |
| Tool Rail 按钮 | 22 | 1.5 | 48px 按钮内（main `size="extra-large"`） |
| Table 字段类型徽章 | 12 | 1.5 | inline |
| Inspector 操作 | 14 | 1.5 | 24px 按钮内 |
| 模态关闭按钮 | 16 | 1.5 | 32px 圆形按钮内 |
| Command Palette 列表项 | 14 | 1.5 | — |

## 6. 颜色继承

所有图标通过 `currentColor` 继承父元素 `color` CSS 属性：

```css
.cdb-btn { color: var(--cdb-color-text-0); }
.cdb-btn:hover { color: var(--cdb-color-primary); }
.cdb-btn--primary { color: var(--cdb-color-primary-on); background: var(--cdb-color-primary); }
```

**禁止在图标组件内硬编码颜色**——通过父元素 `color` 覆盖。

## 7. 暗色模式（E5 接入）

`currentColor` 继承机制保证 E5 阶段无需修改图标组件，只需切换 `color` 即可。

## 8. 验收约束

- `frontend-rs/src/icons.rs` 中 `pub fn icon_*` 函数数 ≥ 50
- 50 个图标函数全部以 `Icon` 前缀导出（在 `pub use` 列表中）
- 所有 SVG `<path>` 不含硬编码 `fill` / `stroke`（除 `currentColor` / `none`）
- `grep -rn '🗑\|✓\|✎\|⋯\|×\|🔍' frontend-rs/src/` 匹配数 ≤ 0（unicode emoji 全部清除）

## 9. 不在 E2 范围

- 动画图标（loading spinner）— E6
- 自定义图标上传（用户上传 SVG）— V2+
- 图标包版本管理（lockfile）— 50 个 SVG 内联在 icons.rs，无外部依赖

