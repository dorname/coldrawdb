# Delta — core-S01-test-cases.md（修改）

> module: core | proposal: align-all-docs-to-unified-prototype

## MODIFIED — 1. 范围

S01 覆盖编辑、自动保存、`SaveState`、非 OT 路径 409。房间协作合并见 S05。

状态：后端已实现；生产前端部分接入；逐项对齐待第二阶段。实现阶段须将用例结果写入 `logos/resources/verify/test-results.jsonl`（OpenLogos reporter）；本提案仅规格收口，不执行自动化。

## ADDED — SaveState 与页面锚点

| ID | 前置 | 操作 | 预期 | 变更 |
|---|---|---|---|---|
| UT-S01-SS-01 | dirty 编辑 | debounce 触发 PUT 成功 | `save-state`：未保存→保存中→已保存；`revision-display` +1 | ADDED |
| UT-S01-SS-02 | PUT 网络失败 | 重试耗尽 | `save-state=Error`；可手动重试；不丢本地 dirty | ADDED |
| ST-S01-SS-01 | room-editor 可写角色 | 改表后等待自动保存 | AppBar 保存态与 revision 与主原型文案阶段一致 | ADDED |

## ADDED — 统一原型对齐补充：非 OT 409

| ID | 说明 | 变更 |
|---|---|---|
| UT-S01-04 / ST-S01-02 | 过期 revision → 409 → `modal-conflict`（reload/force/cancel） | MODIFIED：仅**非 OT** 快照冲突路径 |
| ST-S01-409-SCOPE | 协作模式（S05 已连接 OT）下服务器合并成功 | **禁止**出现 `modal-conflict`；Toast/Activity 反馈 | ADDED |

## ADDED — 协作模式禁 409（合同）

| ID | 前置 | 操作 | 预期 |
|---|---|---|---|
| ST-S01-NO-409-OT | 两用户 OT 已连接 | A、B 近同时编辑并 ack | 无 S01 409 模态；`ot-rev` 前进；Activity 有记录 |
| ST-S01-409-LOCAL-ONLY | 用户选择「仅本地编辑」后 PUT 冲突 | 可走 409 模态 | 须持续显示离线/409 风险文案 |

## ADDED — 附录 A 增量：统一原型对齐用例 ID

| ID | 标题 | 对齐实现 |
|---|---|---|
| UT-S01-SS-01 | SaveState 成功路径 | `editor_data_access` + AppBar |
| UT-S01-SS-02 | SaveState 失败 | 同上 |
| ST-S01-SS-01 | 保存态 UI | room-editor |
| ST-S01-NO-409-OT | 协作禁 409 模态 | 与 S05 联测 |
| ST-S01-409-LOCAL-ONLY | 降级后允许 409 | S01+S05 |
