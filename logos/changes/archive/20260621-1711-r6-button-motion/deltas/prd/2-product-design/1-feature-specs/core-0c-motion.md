## ADDED — 4.8 按钮 focus / active（R6）

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

> `.cdb-btn` transition 须包含 `transform` 与 `box-shadow`（与 E6 §4.5 合并）。

## ADDED — 4.9 面板 spring 入场（R6）

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

> `.cdb-save-dot--saving` 使用 `animation: cdb-pulse-opacity 1s infinite`；禁止与 scale 版 `cdb-pulse` 混用。

## MODIFIED — 3. @keyframes 清单

在 `cdb-pulse` 后追加：

```css
@keyframes cdb-pulse-opacity {
  0%, 100% { opacity: 1; }
  50%      { opacity: 0.4; }
}
```

## MODIFIED — 7. 验收约束

在 E6 约束后追加 R6 项（见主文档 §7 末四条 UT-R6 对应约束）。
