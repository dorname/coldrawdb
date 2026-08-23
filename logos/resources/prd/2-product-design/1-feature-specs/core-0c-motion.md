# Motion 规格（E6 — 动效与微交互）

## 0. 事实基线

唯一现行动效基线：`core-01-editor-prototype.html`。默认缓动 `--ease: cubic-bezier(.2,.8,.2,1)`。不引入 framer-motion。

## 1. 概述

E6 为 drawdb-web 引入动效与微交互，对齐 main `framer-motion` + Semi Design 内置 transition。**E6 不引入 framer-motion**（避免依赖膨胀），而是用 CSS `@keyframes` + `transition` + 工具类实现等价效果。

**E1 阶段**已定义动效 token（`--cdb-duration-{fast,base,slow}` + `--cdb-easing-{in,out,inOut}`），E6 填充具体 `@keyframes` 与组件级动效接线。

## 2. 动效 Token（已 E1 定义）

| 用途 | 主原型事实 |
|---|---|
| 默认缓动 | `--ease` = `cubic-bezier(.2,.8,.2,1)` |
| 按钮 / 工具 | `transition: .18s var(--ease)` |
| Tooltip | `.15s` 显隐 |
| Inspector | `.22s` 位移/透明度 |
| 远端光标 | `left/top .42s var(--ease)`（≤760px 关闭 transition） |
| 表远程高亮 | `remote-pulse` `1.1s` |

生产可将 `--cdb-duration-*` / `--cdb-easing-*` 映射到上述时长；冲突时以主原型观感为准。

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

主原型已包含：

```css
@media (prefers-reduced-motion: reduce) {
  *, *::before, *::after {
    animation-duration: .01ms !important;
    animation-iteration-count: 1 !important;
    transition-duration: .01ms !important;
    scroll-behavior: auto !important;
  }
}
```

要求：

- 装饰性脉冲、抽屉/Toast/Modal 位移在 reduce 下近似瞬时
- 不删除功能反馈（Toast 仍出现，只是无滑动）
- 移动端已对 `.remote-cursor` 关闭 transition，与 reduce 目标一致

## REMOVED / 调整

- 以 Semi duration 120/200/300ms 为唯一真值 → 对齐主原型 `.15/.18/.22/.24/.25s` 量级
- Issues 徽章无限 `cdb-pulse` scale 非主原型路径 → 非强制；保存态可用轻量 opacity 脉冲但须独立 keyframes 名
- Spring `--cdb-easing-spring` 可作为生产增强，不得与主原型 `--ease` 冲突到观感割裂

## 7. 验收约束

- Toast、Drawer、Modal overlay 具备入场动效（reduce 下可瞬时）
- `prefers-reduced-motion: reduce` 时动画/过渡时长 ≤ `0.01ms`
- 按钮 hover/active 微交互存在且不导致布局抖动越界
- 动效不阻塞关闭清理（无残留遮罩）

## 8. 不在 E6 范围

- 拖拽时元素跟随（已有原生 HTML5 drag，E6 不替换）
- 复杂路径动画（贝塞尔曲线轨迹）— V2+
- Spring 物理动画（framer-motion 风格）— V2+（R6 面板 spring 入场使用 CSS `--cdb-easing-spring` 近似，见 §4.9）
- 滚动视差（parallax）— 不做

## Toast / 抽屉 / 浮层关键帧

| 动画 | 时长 | 行为 |
|---|---|---|
| `toast-in` | `.25s var(--ease)` | 自右 `translateX(16px)` + fade |
| `drawer-in` | `.24s var(--ease)` | 自右 `translateX(25px)` + fade |
| `fade`（overlay） | `.18s` | 遮罩淡入 |
| `modal-in` | `.22s var(--ease)` | 上移 10px + `scale(.98→1)` + fade |
| `remote-pulse` | `1.1s` | 选中表远端编辑光晕 |

Spinner（Auth loading / 同步 Banner）保持旋转类动画，直至状态结束卸载。

## 微交互

| 元素 | 动效 |
|---|---|
| `.btn:hover` | `translateY(-1px)` |
| `.btn:active` | `translateY(0) scale(.98)` |
| `.room-card:hover` | `translateY(-4px)` + 阴影加强 |
| `.tool-button` | hover 背景；`.tool-tip` 淡入 |
| Banner / Toast 出现 | 随状态挂载播放入场；卸载即移除 |

关闭动画：原型多为即时卸载；生产若补退场动画，须遵守 `core-09`「关闭后不残留」，并在 `prefers-reduced-motion` 下可跳过。
