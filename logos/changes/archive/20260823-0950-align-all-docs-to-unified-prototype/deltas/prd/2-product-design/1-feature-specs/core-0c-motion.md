# Delta — core-0c-motion.md（修改）

> module: core | proposal: align-all-docs-to-unified-prototype

## ADDED — 0. 事实基线

唯一现行动效基线：`core-01-editor-prototype.html`。默认缓动 `--ease: cubic-bezier(.2,.8,.2,1)`。不引入 framer-motion。

## MODIFIED — 2. 动效 Token（已 E1 定义）

| 用途 | 主原型事实 |
|---|---|
| 默认缓动 | `--ease` = `cubic-bezier(.2,.8,.2,1)` |
| 按钮 / 工具 | `transition: .18s var(--ease)` |
| Tooltip | `.15s` 显隐 |
| Inspector | `.22s` 位移/透明度 |
| 远端光标 | `left/top .42s var(--ease)`（≤760px 关闭 transition） |
| 表远程高亮 | `remote-pulse` `1.1s` |

生产可将 `--cdb-duration-*` / `--cdb-easing-*` 映射到上述时长；冲突时以主原型观感为准。

## ADDED — Toast / 抽屉 / 浮层关键帧

| 动画 | 时长 | 行为 |
|---|---|---|
| `toast-in` | `.25s var(--ease)` | 自右 `translateX(16px)` + fade |
| `drawer-in` | `.24s var(--ease)` | 自右 `translateX(25px)` + fade |
| `fade`（overlay） | `.18s` | 遮罩淡入 |
| `modal-in` | `.22s var(--ease)` | 上移 10px + `scale(.98→1)` + fade |
| `remote-pulse` | `1.1s` | 选中表远端编辑光晕 |

Spinner（Auth loading / 同步 Banner）保持旋转类动画，直至状态结束卸载。

## ADDED — 微交互

| 元素 | 动效 |
|---|---|
| `.btn:hover` | `translateY(-1px)` |
| `.btn:active` | `translateY(0) scale(.98)` |
| `.room-card:hover` | `translateY(-4px)` + 阴影加强 |
| `.tool-button` | hover 背景；`.tool-tip` 淡入 |
| Banner / Toast 出现 | 随状态挂载播放入场；卸载即移除 |

关闭动画：原型多为即时卸载；生产若补退场动画，须遵守 `core-09`「关闭后不残留」，并在 `prefers-reduced-motion` 下可跳过。

## MODIFIED — 6. 动效减弱（accessibility）

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

## MODIFIED — 7. 验收约束

- Toast、Drawer、Modal overlay 具备入场动效（reduce 下可瞬时）
- `prefers-reduced-motion: reduce` 时动画/过渡时长 ≤ `0.01ms`
- 按钮 hover/active 微交互存在且不导致布局抖动越界
- 动效不阻塞关闭清理（无残留遮罩）
