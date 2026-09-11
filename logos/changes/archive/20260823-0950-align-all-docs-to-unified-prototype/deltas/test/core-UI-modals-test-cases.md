# Delta — core-UI-modals-test-cases.md（修改）

> module: core | proposal: align-all-docs-to-unified-prototype

## MODIFIED — 1. 范围

主模态：遮罩、Esc、背景点击关闭，关闭后**无残留** overlay。

状态：后端已实现；生产前端部分接入；逐项对齐待第二阶段。实现阶段须将用例结果写入 `logos/resources/verify/test-results.jsonl`（OpenLogos reporter）；本提案仅规格收口，不执行自动化。

## MODIFIED / ADDED — 关闭与残留

| ID | 变更 | 合同 |
|---|---|---|
| UT-MM-04 / UT-MM-05 | MODIFIED | 背景点击 / Esc → `modal_kind=None`；`modal-root` 与遮罩从 DOM 移除或 `hidden`；焦点回退合理 |
| ST-MM-ESC-01（ADDED） | ADDED | 连续打开 New/Share/Conflict 再 Esc：无透明拦截层；画布可点击 |
| ST-MM-CONFLICT-OT（ADDED） | ADDED | 协作 OT 成功路径**不**打开 conflict 模态（与 S01/S05 交叉） |
| UT-MM-01/06/07/08 | 保留 | 标题校验、Share URL 格式等 |

## ADDED — 层级

主模态必须高于抽屉/Popover；关闭动画结束（或 reduced-motion 即时）后不得残留 pointer-events 挡板。
