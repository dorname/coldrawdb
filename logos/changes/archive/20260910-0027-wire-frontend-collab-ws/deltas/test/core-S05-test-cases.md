## MODIFIED — 1. 范围

## 1. 范围

S05：OT 实时协作。关键可见态：`ws-status`、`ot-rev`、presence、reconnect、queue、local-only。

状态：后端已实现；生产前端已接入真实 WS（`wire-frontend-collab-ws`：连接 / op 双向 / presence / 重连 sync / checkpoint 头 / viewer 只读）。可见状态与降级用例自动化结果写入 `logos/resources/verify/test-results.jsonl`。不得将「规格已写」标为「生产已完成」。
