## MODIFIED — 顶部元数据剥离

> 模块：core | 提案：add-baseline-docs
> 路径：`logos/resources/prd/2-product-design/1-feature-specs/core-0c-motion.md`
> 策略：移除文件开头的 `# Delta — xxx（新文件）` 包装块与紧随的 `## ADDED — 全文` 子块及其 `>` 元数据行，保留真实一级标题以下所有内容原样。

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

## 8. 不在 E6 范围

- 拖拽时元素跟随（已有原生 HTML5 drag，E6 不替换）
- 复杂路径动画（贝塞尔曲线轨迹）— V2+
- Spring 物理动画（framer-motion 风格）— V2+
- 滚动视差（parallax）— 不做

