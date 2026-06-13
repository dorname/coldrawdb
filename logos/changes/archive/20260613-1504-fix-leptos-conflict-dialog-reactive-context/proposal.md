# 变更提案：fix-leptos-conflict-dialog-reactive-context

> module: core | created: 2026-06-13

## 变更原因

`./scripts/start-local.sh` 启动后，访问 `http://localhost:8080/editor` 时前端界面无法正常渲染：画布不显示、按钮与文本错乱、整体无样式。浏览器控制台抛出 Leptos 响应式上下文警告：

```
At src/editor_panels.rs:245:24, you access a signal or memo
  (defined at src/editor_panels.rs:1121:52) outside a reactive tracking context.
At src/editor_panels.rs:290:21, you access a signal or memo
  (defined at src/editor_panels.rs:1122:43) outside a reactive tracking context.
```

WASM 栈顶端指向 `__ConflictDialog::render → AppRoot → mount_to_body`，因此整个 `AppRoot` 在挂载阶段就处于未追踪状态，导致画布、按钮、样式等关键 DOM 未正确挂载。

**根因**：`ConflictDialog` 与 `ErrorToast` 组件内部使用内嵌 `fn render(...) -> impl IntoView`，并在 `view!` 模板中以 `{render(...)}` 普通函数方式调用。这种写法在 Leptos 0.5 中**不建立响应式订阅**，signal 的 `get()` 处于 untracked 上下文，且组件树未正确挂载。

## 变更类型

代码级修复

## 变更范围

- 影响的需求文档：无
- 影响的功能规格：无
- 影响的业务场景：S01（编辑保存）、S02（分享加载）—— `AppRoot` 渲染修复后全场景入口恢复
- 影响的 API：无
- 影响的 DB 表：无
- 影响的编排测试：无
- 影响的 smoke 测试：`SMOKE-core-04` 静态资源加载验证（重新可观察到 `data-testid` 出现）
- 影响的源文件：
  - `frontend-rs/src/editor_panels.rs:240-283`（`ConflictDialog` 的内嵌 render 函数）
  - `frontend-rs/src/editor_panels.rs:288-301`（`ErrorToast` 的内嵌 render 函数）

## 部署影响

- 是否需要部署：否
- 部署原因：仅本地前端代码修复
- 影响环境：本地
- 是否涉及数据迁移：否
- 是否需要回滚预案：否
- 是否需要 smoke：是（需重新验证本地启停 + UI 渲染）

## 变更概述

将 `ConflictDialog` 与 `ErrorToast` 组件内的 `fn render` 改为 `move ||` 响应式闭包，确保 `signal.get()` 在 reactive context 中被订阅、UI 能正确响应变更。`view!` 中调用从 `{render(...)}` 改为直接传闭包。
