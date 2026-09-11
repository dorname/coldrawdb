# Delta — core-S05-test-cases.md（修改）

> module: core | proposal: implement-unified-prototype-spec-parity

## MODIFIED — 1. 范围

S05：OT 实时协作。关键可见态：`ws-status`、`ot-rev`、presence、reconnect、queue、local-only。

状态：后端已实现；生产前端部分接入。本提案 `implement-unified-prototype-spec-parity`（C 批）将可见状态与降级用例落实为自动化，结果写入 `logos/resources/verify/test-results.jsonl`。不得将「规格已写」标为「生产已完成」。

## MODIFIED — 可见状态与降级用例

| ID | 前置 | 操作 | 预期 | 状态 |
|---|---|---|---|---|
| ST-S05-UI-01 | Owner/Editor 进房 | 建立 WS | `ws-status` 已连接；收到 `connected`；`ot-rev` 显示 serverRev | 本提案 C 批实现 |
| ST-S05-UI-02 | 两用户在线 | A 创建表 | A ack；B remote_op；两端 `ot-rev` 一致；Activity 有记录；**无 S01 409 模态** | 本提案 C 批实现 |
| ST-S05-UI-03 | 两用户在线 | A 移动光标/选中 | B 见 `room-presence` / remote-cursor；不遮挡本地选中 | 本提案 C 批实现 |
| ST-S05-UI-04 | 连接中断 | 本地继续编辑 | 队列计数可见；`reconnect-banner`；重连 sync 后队列清零 | 本提案 C 批实现 |
| ST-S05-UI-05 | 重连失败 | 选择仅本地编辑 | 明确 409 风险；本地可编辑；不误报 OT 已同步 | 本提案 C 批实现 |
| ST-S05-UI-06 | Viewer | 尝试发 op | 前端不发送；或 READ_ONLY；head/`ot-rev` 不因本地写递增 | 本提案 C 批实现 |

## MODIFIED — 既有 S05 用例补充约束

保留帧级断言；补充前端必须绑定上表锚点。协作成功路径禁止 S01 409 模态（与 S01 交叉用例 `ST-S01-NO-409-OT`）。本提案 C 批负责落实上表自动化。
