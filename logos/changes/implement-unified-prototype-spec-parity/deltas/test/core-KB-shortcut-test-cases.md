# Delta — core-KB-shortcut-test-cases.md（修改）

> module: core | proposal: implement-unified-prototype-spec-parity

## MODIFIED — 1. 范围

快捷键与主原型一致处：⌘K/Ctrl+K、Esc、T/R（建表/关系）等。

状态：后端已实现；生产前端部分接入。本提案 `implement-unified-prototype-spec-parity`（D 批）将 ST-KB-* 落实为自动化，结果写入 `logos/resources/verify/test-results.jsonl`。不得将「规格已写」标为「生产已完成」。

## MODIFIED — ADDED / MODIFIED

| ID | 前置 | 操作 | 预期 | 状态 |
|---|---|---|---|---|
| ST-KB-CMD-01 | room-editor | ⌘K / Ctrl+K | 打开 `command-palette`（主原型入口 `tool-search`）；再 Esc 关闭无残留 | 本提案 D 批实现 |
| ST-KB-ESC-01 | 任意浮层 | Esc | 按层级关闭最上层；不误关编辑器页 | 本提案 D 批实现 |
| ST-KB-T-01 | 可写 | 按 `T`（无输入焦点） | 触发建表工具/新建表（与主原型 tool tip 一致） | 本提案 D 批实现 |
| ST-KB-R-01 | 可写 | 按 `R` | 进入关系工具 | 本提案 D 批实现 |
| UT-KB-01 / UT-MM-15/16 / ST-UI-05 | 既有 | 撤销重做 | 保留；输入框焦点时快捷键不抢焦点 | 既有；D 批回归 |
| ST-KB-VIEWER（ADDED） | Viewer | T/R | 不创建；只读 | 本提案 D 批实现 |
