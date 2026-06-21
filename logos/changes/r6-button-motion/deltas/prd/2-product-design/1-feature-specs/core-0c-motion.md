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

> `.cdb-btn` transition 须包含 `transform` 与 `box-shadow`（与 E6 §4.5 合并，不重复定义块）。

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

> Spring 使用 CSS `cubic-bezier` 近似（`--cdb-easing-spring`），不引入 framer-motion。

## MODIFIED — 3. @keyframes 清单

在 `cdb-pulse`（scale，Issues 徽章）之外 **新增** 保存态专用动画：

```css
@keyframes cdb-pulse-opacity {
  0%, 100% { opacity: 1; }
  50%      { opacity: 0.4; }
}
```

`.cdb-save-dot--saving` 使用 `animation: cdb-pulse-opacity 1s infinite`；**禁止**与 E6 `cdb-pulse` scale 混用同名 keyframes。

## MODIFIED — 7. 验收约束

在 E6 约束后追加：

- `styles.css` 仅 **一处** `@keyframes cdb-pulse`（scale）
- 存在 `@keyframes cdb-pulse-opacity` 且 `.cdb-save-dot--saving` 引用
- `.cdb-btn:focus-visible` 使用 `var(--cdb-shadow-focus)`
- `.cdb-btn--primary:active` 使用 `var(--cdb-color-primary-active)`
- Inspector / IO Drawer / 溢出菜单使用 `var(--cdb-easing-spring)`
