# Motion 规格（E6 — 动效与微交互）

## 1. 概述

E6 为 drawdb-web 引入动效与微交互，对齐 main `framer-motion` + Semi Design 内置 transition。**E6 不引入 framer-motion**（避免依赖膨胀），而是用 CSS `@keyframes` + `transition` + 工具类实现等价效果。

**E1 阶段**已定义动效 token（`--cdb-duration-{fast,base,slow}` + `--cdb-easing-{in,out,inOut}`），E6 填充具体 `@keyframes` 与组件级动效接线。

## 2. 动效 Token（已 E1 定义）

| Token | 值 | 用途 |
|---|---|---|
| `--cdb-duration-fast` | `120ms` | hover / focus 反馈 |
| `--cdb-duration-base` | `200ms` | 按钮过渡、tooltip |
| `--cdb-duration-slow` | `300ms` | 模态 / 抽屉 / 命令面板 |
| `--cdb-easing-in` | `cubic-bezier(0.4, 0, 1, 1)` | 退出动画 |
| `--cdb-easing-out` | `cubic-bezier(0, 0, 0.2, 1)` | 进入动画 |
| `--cdb-easing-in-out` | `cubic-bezier(0.4, 0, 0.2, 1)` | 状态切换 |

## 3. @keyframes 清单

```css
/* styles.css 末尾 */

@keyframes cdb-fade-in {
  from { opacity: 0; }
  to   { opacity: 1; }
}

@keyframes cdb-fade-out {
  from { opacity: 1; }
  to   { opacity: 0; }
}

@keyframes cdb-slide-in-right {
  from { transform: translateX(100%); }
  to   { transform: translateX(0); }
}

@keyframes cdb-slide-out-right {
  from { transform: translateX(0); }
  to   { transform: translateX(100%); }
}

@keyframes cdb-slide-down {
  from { transform: translateY(-20px); opacity: 0; }
  to   { transform: translateY(0); opacity: 1; }
}

@keyframes cdb-slide-up {
  from { transform: translateY(20px); opacity: 0; }
  to   { transform: translateY(0); opacity: 1; }
}

@keyframes cdb-pulse {
  0%, 100% { transform: scale(1); }
  50%      { transform: scale(1.08); }
}

@keyframes cdb-pulse-opacity {
  0%, 100% { opacity: 1; }
  50%      { opacity: 0.4; }
}

@keyframes cdb-spin {
  from { transform: rotate(0deg); }
  to   { transform: rotate(360deg); }
}
```

## 4. 组件级动效接线

### 4.1 Modal（E3 §3 接线）

```css
.cdb-modal {
  animation: cdb-fade-in var(--cdb-duration-slow) var(--cdb-easing-out),
             cdb-slide-down var(--cdb-duration-slow) var(--cdb-easing-out);
}

.cdb-modal--closing {
  animation: cdb-fade-out var(--cdb-duration-base) var(--cdb-easing-in) reverse;
}
```

**Leptos 接线**：
```rust
// 关闭动画需要在 visible 变 false 后延迟卸载 DOM
let closing = create_rw_signal(false);
let close_with_anim = move |_| {
    closing.set(true);
    set_timeout(move || {
        visible.set(false);
        closing.set(false);
    }, 200);
};
```

### 4.2 SideSheet（E3 §9 接线）

```css
.cdb-side-sheet {
  animation: cdb-slide-in-right var(--cdb-duration-slow) var(--cdb-easing-out);
}

.cdb-side-sheet--closing {
  animation: cdb-slide-out-right var(--cdb-duration-base) var(--cdb-easing-in) reverse;
}
```

### 4.3 Tooltip（E3 §5 接线）

```css
.cdb-tooltip {
  animation: cdb-fade-in var(--cdb-duration-base) var(--cdb-easing-out);
  animation-delay: 200ms;  /* 与 Tooltip delay_ms 同步 */
}
```

### 4.4 Dropdown / Popover（E3 §4/§6 接线）

```css
.cdb-dropdown {
  animation: cdb-slide-down var(--cdb-duration-base) var(--cdb-easing-out);
  transform-origin: top left;
}
```

### 4.5 Button hover / active

```css
.cdb-btn {
  transition: background-color var(--cdb-duration-fast) var(--cdb-easing-out),
              color var(--cdb-duration-fast) var(--cdb-easing-out),
              border-color var(--cdb-duration-fast) var(--cdb-easing-out),
              box-shadow var(--cdb-duration-fast) var(--cdb-easing-out);
}
```

### 4.6 Issues 徽章 pulse（E3 §8 接线）

```css
.cdb-tag--warning[data-count]:not([data-count="0"]) {
  animation: cdb-pulse 2s ease-in-out infinite;
}
```

### 4.7 Loading Spinner

```css
.cdb-spinner {
  animation: cdb-spin 1s linear infinite;
}
```

**Props**（E3 Button 已有 `loading`）：
```rust
<Button loading=true>
    <span class="cdb-spinner" />
    "加载中..."
</Button>
```

### 4.8 按钮 focus / active（R6）

```css
.cdb-btn:focus-visible {
  outline: none;
  box-shadow: var(--cdb-shadow-focus);
}

.cdb-btn--primary:active:not(:disabled) {
  background: var(--cdb-color-primary-active);
  border-color: var(--cdb-color-primary-active);
  transform: translateY(1px);
}

.cdb-tool-btn:focus-visible,
.cdb-tab--icon:focus-visible {
  outline: none;
  box-shadow: var(--cdb-shadow-focus);
}
```

> `.cdb-btn` transition 须包含 `transform` 与 `box-shadow`（与 E6 §4.5 合并，不重复定义块）。

### 4.9 面板 spring 入场（R6）

```css
.cdb-inspector {
  animation: cdb-slide-in-right var(--cdb-duration-slow) var(--cdb-easing-spring);
}

.cdb-main.cdb-has-io-drawer .cdb-io-drawer {
  animation: cdb-slide-in-right var(--cdb-duration-slow) var(--cdb-easing-spring);
}

.cdb-app-bar__overflow-menu {
  animation: cdb-slide-down var(--cdb-duration-base) var(--cdb-easing-spring);
}
```

> Spring 使用 CSS `cubic-bezier` 近似（`--cdb-easing-spring`），不引入 framer-motion。

> `.cdb-save-dot--saving` 使用 `animation: cdb-pulse-opacity 1s infinite`；**禁止**与 E6 `cdb-pulse` scale 混用同名 keyframes。

## 5. 关闭动画模式

为支持 Modal/SideSheet/Tooltip/Dropdown 关闭时播放退出动画，引入**延迟卸载模式**：

```rust
let actually_visible = create_rw_signal(false);
let is_closing = create_rw_signal(false);

create_effect(move |_| {
    if visible.get() && !actually_visible.get() {
        actually_visible.set(true);
    } else if !visible.get() && actually_visible.get() && !is_closing.get() {
        is_closing.set(true);
        set_timeout(move || {
            actually_visible.set(false);
            is_closing.set(false);
        }, 200);  // 200ms 退场动画
    }
});

view! {
    <Show when=move || actually_visible.get()>
        <div class={move || if is_closing.get() { "cdb-modal cdb-modal--closing" } else { "cdb-modal" }}>
            {children()}
        </div>
    </Show>
}
```

## 6. 动效减弱（accessibility）

`prefers-reduced-motion: reduce` 媒体查询下关闭所有装饰性动画：

```css
@media (prefers-reduced-motion: reduce) {
  *, *::before, *::after {
    animation-duration: 0.01ms !important;
    animation-iteration-count: 1 !important;
    transition-duration: 0.01ms !important;
  }
}
```

## 7. 验收约束

- `styles.css` 含 8 个 `@keyframes cdb-*` 动画定义
- 8 个组件（Button/Modal/Dropdown/Tooltip/Popover/Tag/Collapse/SideSheet）至少 1 个动效接线
- Modal 关闭时有 200ms 退出动画（UT-E6-01）
- SideSheet 关闭时有 200ms slide-out 动画（UT-E6-02）
- Issues 徽章在 count > 0 时 pulse（UT-E6-03）
- `prefers-reduced-motion: reduce` 时动画时长 ≤ 0.01ms（UT-E6-04）
- ST-PE-07：Playwright 断言模态动画结束后状态
- `styles.css` 仅 **一处** `@keyframes cdb-pulse`（scale）
- 存在 `@keyframes cdb-pulse-opacity` 且 `.cdb-save-dot--saving` 引用
- `.cdb-btn:focus-visible` 使用 `var(--cdb-shadow-focus)`
- `.cdb-btn--primary:active` 使用 `var(--cdb-color-primary-active)`
- Inspector / IO Drawer / 溢出菜单使用 `var(--cdb-easing-spring)`

## 8. 不在 E6 范围

- 拖拽时元素跟随（已有原生 HTML5 drag，E6 不替换）
- 复杂路径动画（贝塞尔曲线轨迹）— V2+
- Spring 物理动画（framer-motion 风格）— V2+（R6 面板 spring 入场使用 CSS `--cdb-easing-spring` 近似，见 §4.9）
- 滚动视差（parallax）— 不做

