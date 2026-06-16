# Delta — core-09-core-components.md（新文件）

> merge 时作为新文件写入 `logos/resources/prd/2-product-design/1-feature-specs/core-09-core-components.md`

## ADDED — 全文

> 模块：core | 提案：redesign-phase-e-design-system-migration（E3）
> 路径：`logos/resources/prd/2-product-design/1-feature-specs/core-09-core-components.md`
> 对齐参考源：main `@douyinfe/semi-ui` Button/Modal/Dropdown/Tooltip/Popover/Tag/Collapse/SideSheet
> 最后更新：2026-06-15

# Core Components 规格（E3）

## 1. 概述

E3 定义 drawdb-web 的 8 个核心 UI 组件规格，对齐 main `@douyinfe/semi-ui` 的视觉与行为。每个组件以 Leptos 函数组件形式落地在 `frontend-rs/src/components/`，命名 `snake_case.rs`（如 `button.rs` / `modal.rs`），通过 `pub use` 导出。

**所有组件统一约束**：
- Props 不可变优先（`#[prop(into)]` 仅在必要时使用）
- 视觉 class 全部 `cdb-{component}` 前缀
- token 引用全部 `var(--cdb-*)`，**禁止硬编码颜色/尺寸**
- 键盘可达：Tab 可达 + Enter/Space 触发
- ARIA 属性：button/role=button/dialog/listbox 等按 WAI-ARIA 1.2

## 2. Button

```rust
#[component]
pub fn Button(
    children: Children,
    #[prop(default = ButtonVariant::Secondary)] variant: ButtonVariant,
    #[prop(default = ButtonSize::Medium)] size: ButtonSize,
    #[prop(default = false)] disabled: bool,
    #[prop(default = false)] loading: bool,
    #[prop(default = false)] block: bool,
    #[prop(optional)] on_click: Option<Callback<ev::MouseEvent>>,
) -> impl IntoView
```

| Variant | 背景 | 文字 | 边框 | 用途 |
|---|---|---|---|---|
| `Primary` | `--cdb-color-primary` | `--cdb-color-text-on-primary` | none | 主操作（保存、确认） |
| `Secondary` | `--cdb-color-bg-0` | `--cdb-color-text-0` | `--cdb-color-border` | 默认（导入、导出） |
| `Tertiary` | transparent | `--cdb-color-text-0` | none | 文本按钮（取消、链接） |
| `Warning` | `--cdb-color-warning` | `--cdb-color-text-on-primary` | none | 危险操作（删除） |
| `Ghost` | transparent | `--cdb-color-primary` | `--cdb-color-primary` | 次要主操作 |

| Size | 高度 | padding-x | 字号 |
|---|---|---|---|
| `Small` | 24px | 8px | `--cdb-font-size-sm` |
| `Medium` | 32px | 12px | `--cdb-font-size-base` |
| `Large` | 40px | 16px | `--cdb-font-size-md` |

**行为**：
- hover：背景变 `--cdb-color-primary-hover`（200ms `--cdb-easing-out`）
- active：背景变 `--cdb-color-primary-active`
- disabled：opacity 0.5 + `cursor: not-allowed` + 无 hover
- loading：替换 children 为 spinner + 禁用点击
- block：`width: 100%`

## 3. Modal

```rust
#[component]
pub fn Modal(
    children: Children,
    visible: RwSignal<bool>,
    #[prop(default = None)] title: Option<String>,
    #[prop(default = ModalWidth::Medium)] width: ModalWidth,
    #[prop(default = true)] centered: bool,
    #[prop(default = true)] closable: bool,
    #[prop(default = true)] mask_closable: bool,
    #[prop(default = true)] esc_closable: bool,
    #[prop(default = None)] on_ok: Option<Callback<()>>,
    #[prop(default = None)] on_cancel: Option<Callback<()>>,
    #[prop(default = None)] ok_text: Option<String>,
    #[prop(default = None)] cancel_text: Option<String>,
) -> impl IntoView
```

| Width | 值 | 场景 |
|---|---|---|
| `Small` | 400px | 确认（删除、放弃） |
| `Medium` | 640px | 表单（新建表、重命名） |
| `Large` | 800px | 复杂（导入设置、共享） |
| `XLarge` | 1200px | Code View（E4） |
| `Full` | `calc(100vw - 64px)` | 全屏内容 |

**行为**（对齐 main `SemiUIModal`）：
- 居中（`centered=true`）
- 打开时 body 锁滚动（`overflow: hidden`）
- ESC 关闭（`esc_closable=true`）
- 点击遮罩关闭（`mask_closable=true`）
- focus trap：打开时焦点移入 modal，循环 Tab
- 入场动画：`fade-in` 200ms + `slide-down` 200ms（E6 接入）
- 关闭动画：反向 200ms

**z-index**：`--cdb-z-modal`（L5），遮罩 `--cdb-z-modal - 1` = 49

## 4. Dropdown

```rust
#[component]
pub fn Dropdown(
    children: Children,
    menu: DropdownMenu,           // 嵌套 DropdownMenu 组件
    #[prop(default = DropdownTrigger::Click)] trigger: DropdownTrigger,
    #[prop(default = DropdownPosition::BottomLeft)] position: DropdownPosition,
) -> impl IntoView

#[component]
pub fn DropdownMenu(children: Children) -> impl IntoView
#[component]
pub fn DropdownItem(
    children: Children,
    #[prop(default = None)] icon: Option<View>,
    #[prop(default = false)] active: bool,
    #[prop(default = false)] disabled: bool,
    #[prop(optional)] on_click: Option<Callback<ev::MouseEvent>>,
) -> impl IntoView
#[component]
pub fn DropdownDivider() -> impl IntoView
```

| Trigger | 行为 |
|---|---|
| `Click` | 点击切换（AppBar 菜单、Layout 下拉） |
| `Hover` | hover 250ms 后展开（不与 Modal 混用） |

| Position | 含义 |
|---|---|
| `BottomLeft` | 触发器左下对齐菜单左上（默认） |
| `BottomRight` | 触发器右下对齐菜单右上 |
| `TopLeft` / `TopRight` | 同上反向 |

**视觉**：菜单 180–240px 宽，`--cdb-shadow-md`，`--cdb-radius-lg`，`--cdb-bg-0`

**z-index**：`--cdb-z-popover`（L4.5）

## 5. Tooltip

```rust
#[component]
pub fn Tooltip(
    children: Children,
    content: String,
    #[prop(default = TooltipPlacement::Top)] placement: TooltipPlacement,
    #[prop(default = 200)] delay_ms: u32,
) -> impl IntoView
```

| Placement | 含义 |
|---|---|
| `Top` / `Bottom` / `Left` / `Right` | 触发器对应方向居中 |

**行为**：
- 鼠标 hover / focus 后 `delay_ms` 毫秒显示（默认 200ms）
- 鼠标离开 100ms 隐藏
- 文本超出 16 字符时换行省略
- 禁用元素不显示

**视觉**：黑底白字（`--cdb-color-grey-9` + `--cdb-color-text-on-primary`），`--cdb-shadow-md`，`--cdb-radius-sm`，`--cdb-font-size-sm`

**z-index**：`--cdb-z-tooltip`（L4）

## 6. Popover

```rust
#[component]
pub fn Popover(
    children: Children,
    content: View,                 // 复杂内容（嵌套组件）
    #[prop(default = PopoverTrigger::Click)] trigger: PopoverTrigger,
    #[prop(default = PopoverPlacement::BottomLeft)] placement: PopoverPlacement,
    #[prop(default = false)] controlled: bool,
) -> impl IntoView
```

| Trigger | 行为 |
|---|---|
| `Click` | 点击切换（TableInfo、字段详情） |
| `Hover` | hover 200ms 后展开 |

**与 Tooltip 区别**：Popover 可承载复杂内容（表单、列表、表格），Tooltip 仅文本。

**z-index**：`--cdb-z-popover`（L4.5）

## 7. Tag

```rust
#[component]
pub fn Tag(
    children: Children,
    #[prop(default = TagColor::Neutral)] color: TagColor,
    #[prop(default = TagSize::Small)] size: TagSize,
    #[prop(default = false)] closable: bool,
    #[prop(default = false)] bordered: bool,
    #[prop(optional)] on_close: Option<Callback<ev::MouseEvent>>,
) -> impl IntoView
```

| Color | 背景 | 文字 | 边框 | 用途 |
|---|---|---|---|---|
| `Neutral` | `--cdb-color-grey-1` | `--cdb-color-text-1` | `--cdb-color-border` | 默认 |
| `Primary` | `--cdb-color-primary-soft` | `--cdb-color-primary` | none | 选中项 |
| `Success` | `--cdb-color-success-soft` | `--cdb-color-success` | none | 成功状态 |
| `Warning` | `--cdb-color-warning-soft` | `--cdb-color-warning` | none | Issues 徽章 |
| `Error` | `--cdb-color-error-soft` | `--cdb-color-error` | none | 错误状态 |
| `Info` | `--cdb-color-info-soft` | `--cdb-color-info` | none | 信息 |

**视觉**：inline-flex，`--cdb-radius-sm`，`height: 20-24px`

**字段类型徽章用例**（E2 + 1a）：Tag `color=Primary` + IconXxx 显示字段类型

## 8. Collapse

```rust
#[component]
pub fn Collapse(
    children: Children,
    #[prop(default = true)] lazy_render: bool,
    #[prop(default = false)] keep_dom: bool,
    #[prop(default = CollapseBordered::Default)] bordered: CollapseBordered,
) -> impl IntoView

#[component]
pub fn CollapsePanel(
    children: Children,
    header: View,
    item_key: String,
    #[prop(default = false)] disabled: bool,
) -> impl IntoView
```

**行为**：
- 点击 header 展开/收起对应 panel
- `lazy_render=true` 时内容只在首次展开时渲染
- `keep_dom=true` 时收起不卸载 DOM（动画需要）

**视觉**（对齐 main `Issues.jsx`）：
- header 高度 40px，hover `--cdb-color-grey-1`
- panel 边框 `--cdb-color-border`，展开时 0 间距
- 整个 Collapse 无外边框

## 9. SideSheet

```rust
#[component]
pub fn SideSheet(
    children: Children,
    visible: RwSignal<bool>,
    #[prop(default = None)] title: Option<String>,
    #[prop(default = SideSheetPlacement::Right)] placement: SideSheetPlacement,
    #[prop(default = 400)] width: u32,
    #[prop(default = true)] mask: bool,
    #[prop(default = true)] mask_closable: bool,
) -> impl IntoView
```

| Placement | 方向 |
|---|---|
| `Right` | 右侧抽屉（IO 抽屉，Phase C） |
| `Left` | 左侧抽屉（备用） |

**行为**：
- 与 Modal 共享 z-index（`--cdb-z-drawer` L3，因为 IO 抽屉与 Inspector 互斥）
- 关闭时 body 解锁滚动
- 关闭动画：200ms `slide-out-{placement}`（E6 接入）

**视觉**：
- 阴影 `--cdb-shadow-lg`
- 圆角外侧 `--cdb-radius-xl`（仅 Right/Left 外侧圆角）
- 内部 `sidesheet-theme` 背景（`--cdb-color-bg-1`）

**Phase C 升级**：E3 后 `ImportDrawer` / `ExportDrawer` 用 `<SideSheet placement=Right />` 替代内嵌 `<aside>` 实现。

## 10. 组件间层级关系

```
Modal (L5)
  └─ 可包含任意内容，包括 Dropdown / Tag / Button

SideSheet (L3) / Inspector (L3)
  └─ 与 Modal 互斥（开 Modal 时折叠 SideSheet）

Dropdown (L4.5)
  └─ 触发器通常是 Button

Popover (L4.5)
  └─ 触发器通常是 Table 行 / Icon

Tooltip (L4)
  └─ 触发器是任意元素

Tag (无 z-index)
  └─ 内联在文本中
```

## 11. 验收约束

- `frontend-rs/src/components/` 8 个 `.rs` 文件存在（`button.rs` / `modal.rs` / `dropdown.rs` / `tooltip.rs` / `popover.rs` / `tag.rs` / `collapse.rs` / `sidesheet.rs`）
- 每个组件至少 1 个 `data-testid` 属性（`cdb-button` / `cdb-modal` / 等）
- 8 个组件 Props 签名与本规格一一对应
- `grep -rn '#[0-9a-f]\{3,6\}' frontend-rs/src/components/` 匹配 ≤ 0（无硬编码颜色）
- 8 组件视觉对齐 Playwright 截图（HP-01~HP-05）

## 12. 不在 E3 范围

- 主题切换按钮（→ E5）
- Modal/SideSheet 入场动画（→ E6）
- 复杂组合组件（DatePicker / ColorPicker / Tree）— V2+
