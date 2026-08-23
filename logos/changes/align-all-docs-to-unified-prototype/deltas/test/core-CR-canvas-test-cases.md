# Delta — core-CR-canvas-test-cases.md（修改）

> module: core | proposal: align-all-docs-to-unified-prototype

## MODIFIED — 1. 范围

画布：表拖动、pointer capture、关系线跟手。生产松手网格 **`GRID_SIZE=20`**；主原型演示 `GRID=12`，不得把 12 写成生产合同。

状态：后端已实现；生产前端部分接入；逐项对齐待第二阶段。实现阶段须将用例结果写入 `logos/resources/verify/test-results.jsonl`（OpenLogos reporter）；本提案仅规格收口，不执行自动化。

## MODIFIED — UT-CR-06 — 网格对齐仅在松手

| ID | 变更 | 合同 |
|---|---|---|
| UT-CR-06 | MODIFIED | 生产 `snap_to_grid(..., 20.0)`；拖动中不量化 |
| UT-CR-07 / ST-CR-02 | MODIFIED | pointermove 期间关系 path 使用当前视觉坐标；跟手非松手跳变 |
| UT-CR-PC-01（ADDED） | ADDED | 表头拖动 `setPointerCapture`；指针移出命中面不丢拖 |
| UT-CR-PC-02（ADDED） | ADDED | rAF 合并重绘；禁止每 move 整页重建 `#app` |
| ST-CR-GRID-20（ADDED） | ADDED | 生产 e2e：松手后 `x/y` 为 20 的倍数 |
| ST-CR-GRID-PROTO（ADDED） | ADDED | 主原型 PU：松手后为 12 的倍数（仅原型回归） |

## ADDED — 与主原型对齐说明

- 拖表过程中已有关系 SVG/`path[d]` 必须连续更新（对齐 ST-PU-05/21）。
- 生产验收以 `GRID_SIZE=20` + pointer capture + 跟线为准；视觉像素级对齐待第二阶段。
