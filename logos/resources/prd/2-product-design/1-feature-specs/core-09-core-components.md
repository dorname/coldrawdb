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

## 13. 表单下拉（Form Select）

> 提案：feat-room-recycle-bin-dropdown-and-db-export（修复原生 select「内容仿佛展示不出来」的视觉缺陷）

### 13.1 适用范围

所有原生 `<select>` 表单控件：`.cdb-form-select`（IO 抽屉引擎下拉等）与 `.cdb-select`（创建房间 dbtype 下拉等）。自绘菜单（`.cdb-menu-dropdown` / `.cdb-app-bar__overflow-menu`）不在本节范围。

### 13.2 视觉条款

1. **去原生外观**：`appearance: none`（含 `-webkit-`/`-moz-` 前缀），消除 OS 默认箭头与立体边框。
2. **自定义箭头**：右侧 chevron 使用内联 SVG 背景图（`background-image` + `background-repeat: no-repeat` + `background-position: right 10px center`），箭头颜色跟随 `currentColor` 语义（通过 `fill` 嵌入色值，亮暗两套）；为箭头预留 `padding-right: 32px`，文字不得与箭头重叠。
3. **显式配色**：必须显式声明 `color: var(--cdb-color-text)` 与 `background-color: var(--cdb-color-bg-secondary)`（或对应语义 token），不得依赖 UA 默认——暗色模式下未显式配色的 select 会渲染成白底白字。
4. **暗色适配**：`[data-mode="dark"]` 下 select 声明 `color-scheme: dark`（控制原生弹层配色），亮模式声明 `color-scheme: light`；`option` 元素显式声明前景/背景色，避免弹层内文字与背景同色。
5. **防裁切**：`line-height` 不得小于 1.4；`height` 不硬编码（由 padding + line-height 撑起，约 36px）；禁用会导致文字垂直裁切的 `padding`/`height` 组合。
6. **焦点**：`:focus-visible` 复用 `--cdb-shadow-focus`，与 `.cdb-btn` 焦点口径一致；`:disabled` 降透明度 0.5 且 `cursor: not-allowed`。
7. 弹层（原生 dropdown list）的展开高度/滚动由 OS 控制，样式无法干预；本节保证的是**关闭态控件本身**与**弹层配色**不再出现半隐/同色缺陷。
8. **禁止 `background` 简写覆盖**（fix-select-bg-diagram-delete-and-log-format）：任何同时命中 select 的复合选择器（如 `.cdb-create-room-modal .cdb-input`）**不得**使用 `background:` 简写——简写会把 `background-image`/`background-repeat` 重置，压过 `.cdb-select` 的 chevron 单箭头（优先级 0,2,0 > 0,1,0），暗色规则重设 image 后 repeat 失控 → 箭头平铺成满框 `˅˅˅`。必须改用 `background-color:` 单独声明。锚点：UT-PC-23。

### 13.3 验收锚点

UT-PC-19（`include_str!` 锚点口径）断言 `styles.css` 中 `.cdb-form-select` 规则含：`appearance`、chevron `background-image`、`color-scheme`、`option` 配色、`padding-right`。

## 14. Splitter 分隔条

> 提案：fix-canvas-zoom-invite-comment-resize（问题4：布局面板可拖动调整）

### 14.1 适用范围

布局面板的宽度拖动组件，落地两处：

| 实例 | 位置 | 拖动目标 | 默认宽度 | localStorage key |
|---|---|---|---|---|
| `splitter-inspector` | `.cdb-main` 第 2/3 列之间（Inspector 左缘） | CSS 变量 `--cdb-inspector-w` | 330px | `cdb.inspector.width` |
| `splitter-list-tree` | ListView `.cdb-list-view-body` 第 1/2 列之间（树列右缘） | 树列内联 `grid-template-columns` | 230px | `cdb.list-tree.width` |

### 14.2 交互

1. **拖动**：`pointerdown` 在分隔条上时 `set_pointer_capture`，`pointermove` 实时改写宽度，`pointerup` / `pointercancel` 结束并持久化。
2. **拖动方向语义**：宽度增减跟随「分隔条相对面板的边缘位移」，而非统一取 `+dx`——
   - `splitter-inspector`（右侧面板**左缘**）：向左拖（dx<0）面板变宽，向右拖（dx>0）面板变窄，即 `width = start_w - dx`；
   - `splitter-list-tree`（左侧树列**右缘**）：向右拖（dx>0）树列变宽，向左拖变窄，即 `width = start_w + dx`。
   - 键盘方向键维持数值语义（ArrowLeft 减宽 / ArrowRight 加宽，与 `aria-valuenow` 一致），不随实例位置翻转。
3. **hover 态**：分隔条宽 6px，默认透明（叠在两列间隙上）；hover 或拖动中显示品牌色高亮（`--cdb-brand` 或 `--cdb-color-primary`，2px 内缘指示条）。
4. **双击复位**：双击分隔条恢复默认宽度（330 / 230），并写入 localStorage。
5. **键盘可达**：分隔条为 `role="separator"`，`aria-orientation="vertical"`，`aria-valuenow` 反映当前宽度；方向键 ±16px（Shift ±64px），Home/End 跳 min/max，Esc 复位默认。

### 14.3 约束

1. **宽度钳制**：Inspector 240–520px，且不超过 `视口宽度 - 200px`；树列 160–420px。
2. **生效断点**：仅 ≥1024px 视口渲染/可拖动；≤720px（Inspector 为绝对覆盖层）与 721–1023px（全屏遮罩态）下分隔条 `display:none`，宽度变量不生效。
3. **拖动期禁过渡**：`.cdb-main` 存在 `transition: grid-template-columns 160ms`，拖动期间在容器上加临时 class（如 `is-resizing`）将 `transition` 置 `none`，`pointerup` 后移除恢复过渡。
4. **Inspector 折叠态**：`inspector_open=false`（第 3 列归零）时分隔条不可见（列宽为 0 无间隙），重开后恢复持久化宽度。

### 14.4 实现约定（CSS 变量直写，绕过响应式）

1. 宽度存 `RwSignal<u32>` 仅作初始值与持久化来源；**拖动期间不更新 signal**，直接在 `pointermove` 回调中调用 `document.document_element().style().set_property("--cdb-inspector-w", format!("{}px", w))`，避免 Leptos 每帧重渲染。
2. `pointerup` 时一次性写回 signal 并持久化 localStorage。
3. ListView 树列同法：拖动期间直写树容器 `style` 的 `grid-template-columns: {w}px minmax(0,1fr)`。
4. 初始化（mount）时读 localStorage，合法值（数字且在钳制范围内）直写 CSS 变量，非法/缺失用默认值。

### 14.5 testid 与验收锚点

| 元素 | `data-testid` |
|---|---|
| Inspector 分隔条 | `splitter-inspector` |
| ListView 树列分隔条 | `splitter-list-tree` |

- 拖动期间每次 `pointermove` 后 `--cdb-inspector-w` 即时变化（无 160ms 过渡延迟）。
- 钳制：拖过 520 / 240 边界时变量停在边界值。
- 刷新后宽度恢复（见 UT-PU-23）。
- 画布无需额外处理：grid 重排 + RAF 每帧比对 `clientWidth/clientHeight`（`editor_render.rs`）已覆盖拖动期画布跟随（见 ST-PU-29）。
