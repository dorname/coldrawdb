# Core Components 规格（E3）

## 0. 事实基线

唯一现行组件行为与视觉基线：`core-01-editor-prototype.html`。本 delta 对齐 Button / Popover / Modal / SideSheet(Drawer) / Tag / Banner / Toast；其余（Dropdown 菜单项、Tooltip、Collapse）沿用主原型等价行为。

统一约束：视觉引用 `core-07` token；键盘可达；遮罩/浮层关闭后 **DOM 与交互层不得残留**（无透明拦截、无遗留 `pointer-events`、无僵尸 overlay）。

## 1. 概述

E3 定义 drawdb-web 的 8 个核心 UI 组件规格，对齐 main `@douyinfe/semi-ui` 的视觉与行为。每个组件以 Leptos 函数组件形式落地在 `frontend-rs/src/components/`，命名 `snake_case.rs`（如 `button.rs` / `modal.rs`），通过 `pub use` 导出。

**所有组件统一约束**：
- Props 不可变优先（`#[prop(into)]` 仅在必要时使用）
- 视觉 class 全部 `cdb-{component}` 前缀
- token 引用全部 `var(--cdb-*)`，**禁止硬编码颜色/尺寸**
- 键盘可达：Tab 可达 + Enter/Space 触发
- ARIA 属性：button/role=button/dialog/listbox 等按 WAI-ARIA 1.2

## 2. Button

对齐主原型 `.btn` 族：

| 变体 | 行为摘要 |
|---|---|
| 默认 / soft | `--surface-soft` 底 + `--line` 边；hover 上浮 1px |
| `--primary` | brand 渐变；浅色模式白字，暗色模式近黑字 `#050f13` |
| `--danger` | `--red` 字色与淡红边 |
| `--ghost` / `--icon` | 透明底；图标按钮方形 |
| `--sm` | 紧凑高度（画布工具、Banner 动作） |

- `disabled`：不可点、无 hover 位移
- `loading` / `aria-busy`：Auth 提交显示 spinner +「正在验证…」
- 过渡：`.18s var(--ease)`；active `scale(.98)`

## 3. Modal

对齐 `.overlay`（z=50）+ `.modal`：

| 属性 | 事实 |
|---|---|
| 遮罩 | `rgba(2,12,16,.54)` + `blur(8px)`；`data-overlay` |
| 宽度 | 常规 `min(520px,100%)`；宽版 `.modal--wide` → `min(720px,100%)` |
| 结构 | `modal-head` / `modal-body` / `modal-foot` |
| a11y | `role="dialog"` `aria-modal="true"` + `aria-labelledby` |
| 关闭 | 关闭按钮 `close-layer`；点击遮罩（`event.target` 为 overlay）；表单取消 |
| 入场 | `fade` `.18s` + `modal-in` `.22s` |

**遮罩关闭后不残留（强制）**：

1. `layer` 置空后整段 overlay 从渲染树移除（主原型：条件渲染返回空串）。
2. 不得使用 `visibility:hidden` / `opacity:0` 却保留 `position:fixed; inset:0` 拦截点击。
3. 打开 Drawer / Code / Command 时与 Modal 互斥；关闭路径必须清空对应状态。
4. 生产实现若使用延迟卸载动画，动画结束后必须真正卸载；超时失败亦须强制移除。

典型 Modal：`modal-create-room`、`modal-invite`、分享、偏好设置、删除确认、原型诊断。

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

对齐 `.popover`（z≈46）：

- 触发：AppBar 更多菜单、用户菜单、rooms 用户菜单（`state.layer` 切换）
- 视觉：玻璃态、宽约 `230px`、圆角 `14px`；`.menu-item` 高 `39px`，可带 shortcut
- 关闭：再次点击触发器、选择菜单项、打开 Modal/Drawer、Esc（与全局 layer 清理一致）
- 关闭后菜单节点不渲染，不得留下可点击幽灵层

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

对齐 `.drawer`（z=35，宽 `min(420px, calc(100% - 72px))`）：

| Drawer | `data-testid` |
|---|---|
| 成员 | `room-members-panel` |
| 活动 | `activity-feed` |
| 导入 | `import-drawer` |
| 导出 | `export-drawer` |

- 入场：`drawer-in` `.24s`（translateX 25px → 0）
- 关闭：`close-drawer` / 打开 Modal 时 `drawer=null`
- ≤760px：全宽 + 圆角收紧
- 关闭后同样不得残留遮挡画布的透明层（Drawer 无全屏 mask 时，也不得留下不可见 hit-area）

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

- Button / Popover / Modal / Drawer / Tag / Banner / Toast 均可在主原型对应路径演示
- 关闭 Modal/Command/Popover/Drawer/Banner/Toast 后，无残留 fixed 遮罩或拦截层（可用诊断「浮层状态」或 DOM 断言）
- 组件颜色/阴影仅通过 token；硬编码禁止规则见 `core-07`

## 12. 不在 E3 范围

- 主题切换按钮（→ E5）
- Modal/SideSheet 入场动画（→ E6）
- 复杂组合组件（DatePicker / ColorPicker / Tree）— V2+

## Tag

对齐 `.tag` / `.tag--brand` / `.tag--warn`：

- 胶囊、11px 粗体、可内嵌状态点
- 用途：角色、在线人数、待同步计数、代码「实时生成」、房间徽章

## Banner

对齐画布顶部 `.banner`（z=12）：

| 态 | 样式 | 用途 |
|---|---|---|
| 默认（警告） | amber soft | 重连中 / 同步中（`reconnect-banner`） |
| `--danger` | red soft | 离线 / 仅本地编辑 |

含文案 + 可选 `.banner-actions` 按钮；连接恢复为 `connected` 时 Banner **整段卸载**。

## Toast

对齐 `#toast-region.toast-region`（z=60，右上）：

- 结构：图标列 + 标题/正文 + 关闭按钮；玻璃态；`toast-in` `.25s`
- 错误：`.is-error` + info 图标
- 区域：`aria-live="polite"`；约 3600ms 自动消失；可手动 `dismiss-toast`
- `pointer-events:none` 在 region，单项 toast `pointer-events:auto`，避免挡住整页

## REMOVED / 降级

- 以 SemiUI Modal/SideSheet API 像素对齐作为唯一验收 → 改为主原型行为与层级
- Warning 变体按钮若与主原型 `--danger` 冲突，以 danger 语义为准
- Tooltip 黑底白字强制 → 主原型 ToolRail tip 为表面色玻璃 tip（可保留生产增强，但不得与暗色对比度冲突）
