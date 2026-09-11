# Delta — core-S01-test-cases.md（修改）

> module: core | proposal: implement-unified-prototype-spec-parity

## MODIFIED — 1. 范围

S01 覆盖编辑、自动保存、`SaveState`、非 OT 路径 409。房间协作合并见 S05。

状态：后端已实现；生产前端部分接入。本提案 `implement-unified-prototype-spec-parity`（C 批）将 SaveState / 非 OT 409 / 协作禁 409 用例落实为自动化，结果写入 `logos/resources/verify/test-results.jsonl`。不得将「规格已写」标为「生产已完成」。

## MODIFIED — SaveState 与页面锚点

| ID | 前置 | 操作 | 预期 | 状态 |
|---|---|---|---|---|
| UT-S01-SS-01 | dirty 编辑 | debounce 触发 PUT 成功 | `save-state`：未保存→保存中→已保存；`revision-display` +1 | 本提案 C 批实现 |
| UT-S01-SS-02 | PUT 网络失败 | 重试耗尽 | `save-state=Error`；可手动重试；不丢本地 dirty | 本提案 C 批实现 |
| ST-S01-SS-01 | room-editor 可写角色 | 改表后等待自动保存 | AppBar 保存态与 revision 与主原型文案阶段一致 | 本提案 C 批实现 |

## MODIFIED — 非 OT 409

| ID | 说明 | 状态 |
|---|---|---|
| UT-S01-04 / ST-S01-02 | 过期 revision → 409 → `modal-conflict`（reload/force/cancel）；仅**非 OT** 快照冲突路径 | 既有；本提案回归 |
| ST-S01-409-SCOPE | 协作模式（S05 已连接 OT）下服务器合并成功；**禁止**出现 `modal-conflict`；Toast/Activity 反馈 | 本提案 C 批实现 |

## MODIFIED — 协作模式禁 409（合同）

| ID | 前置 | 操作 | 预期 | 状态 |
|---|---|---|---|---|
| ST-S01-NO-409-OT | 两用户 OT 已连接 | A、B 近同时编辑并 ack | 无 S01 409 模态；`ot-rev` 前进；Activity 有记录 | 本提案 C 批实现 |
| ST-S01-409-LOCAL-ONLY | 用户选择「仅本地编辑」后 PUT 冲突 | 可走 409 模态 | 须持续显示离线/409 风险文案 | 本提案 C 批实现 |

## MODIFIED — 附录 A 增量：统一原型对齐用例 ID

| ID | 标题 | 对齐实现 | 状态 |
|---|---|---|---|
| UT-S01-SS-01 | SaveState 成功路径 | `editor_data_access` + AppBar | 本提案 C 批实现 |
| UT-S01-SS-02 | SaveState 失败 | 同上 | 本提案 C 批实现 |
| ST-S01-SS-01 | 保存态 UI | room-editor | 本提案 C 批实现 |
| ST-S01-NO-409-OT | 协作禁 409 模态 | 与 S05 联测 | 本提案 C 批实现 |
| ST-S01-409-LOCAL-ONLY | 降级后允许 409 | S01+S05 | 本提案 C 批实现 |
