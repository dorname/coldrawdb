# Delta — core-KB-shortcut-test-cases.md（修改）

> module: core | proposal: align-all-docs-to-unified-prototype

## MODIFIED — 1. 范围

快捷键与主原型一致处：⌘K/Ctrl+K、Esc、T/R（建表/关系）等。

状态：后端已实现；生产前端部分接入；逐项对齐待第二阶段。实现阶段须将用例结果写入 `logos/resources/verify/test-results.jsonl`（OpenLogos reporter）；本提案仅规格收口，不执行自动化。

## ADDED / MODIFIED

| ID | 前置 | 操作 | 预期 | 变更 |
|---|---|---|---|---|
| ST-KB-CMD-01 | room-editor | ⌘K / Ctrl+K | 打开 `command-palette`（主原型入口 `tool-search`）；再 Esc 关闭无残留 | ADDED |
| ST-KB-ESC-01 | 任意浮层 | Esc | 按层级关闭最上层；不误关编辑器页 | ADDED |
| ST-KB-T-01 | 可写 | 按 `T`（无输入焦点） | 触发建表工具/新建表（与主原型 tool tip 一致） | ADDED |
| ST-KB-R-01 | 可写 | 按 `R` | 进入关系工具 | ADDED |
| UT-KB-01 / UT-MM-15/16 / ST-UI-05 | 既有 | 撤销重做 | 保留 | MODIFIED（补充与命令面板共存：输入框焦点时快捷键不抢焦点） |
| ST-KB-VIEWER（ADDED） | Viewer | T/R | 不创建；只读 | ADDED |

## ADDED — 边界

未在主原型出现的自定义快捷键不纳入本提案合同；Space 平移等 V1 占位保持边界。
