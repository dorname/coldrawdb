## ADDED — 7.5 R6 按钮与面板动效（UT-R6）

### UT-R6-01 — 保存圆点 pulse 命名拆分

**步骤**：
1. 读取 `frontend-rs/src/styles.css`
2. 断言：存在 `@keyframes cdb-pulse-opacity`
3. 断言：`.cdb-save-dot--saving` 引用 `cdb-pulse-opacity`
4. 断言：`@keyframes cdb-pulse` 仅定义 `transform: scale`（无 opacity 版重复）

**预期**：保存态与 Issues 徽章 pulse 互不覆盖

### UT-R6-02 — 按钮 focus / primary active

**步骤**：
1. 断言：`.cdb-btn:focus-visible` 含 `var(--cdb-shadow-focus)`
2. 断言：`.cdb-btn--primary:active` 含 `var(--cdb-color-primary-active)`

**预期**：键盘焦点与主按钮按压态可验收

### UT-R6-03 — 面板 spring 入场

**步骤**：
1. 断言：`.cdb-inspector` animation 含 `var(--cdb-easing-spring)`
2. 断言：`.cdb-has-io-drawer .cdb-io-drawer` animation 含 `var(--cdb-easing-spring)`
3. 断言：`.cdb-app-bar__overflow-menu` animation 含 `var(--cdb-easing-spring)`

**预期**：三处面板使用 spring easing
