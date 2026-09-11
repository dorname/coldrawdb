## ADDED — 4.3 真实 WS 传输接线用例（wire-frontend-collab-ws 增量）

> 区分说明：§4.1/§4.2（UT/ST-FE-S05-01~06）为**帧注入 / 状态机级**自动化，验证解析与 UI 状态逻辑；本子节用例针对**真实传输接线**——生产前端与真实后端建立实际 WebSocket，覆盖 wire-frontend-collab-ws 补齐的实现缺口。ST-FE-S05-07~09 使用 Playwright 双浏览器上下文 + 真实 backend，禁止 mock WS。

### 4.3.1 单元 / 组件辅助用例

| ID | 前置 | 操作 | 预期 |
|---|---|---|---|
| UT-FE-S05-13 | 本地编辑事件 | table/field/reference/area/note 各 create/update/delete 共 15 种变更 → CollabOp 映射 | op `type`/`targetId`/`parentId`/`changes` 与 `collab.yaml` `CollabOp` 一致；未映射的变更不产生 op |
| UT-FE-S05-14 | 构造 remote_op 帧 | 应用到 EditorStore | 画布状态按 op 变更（建表/改字段/删引用等各一例）；应用远端 op 不产生回声（不再上行 op） |
| UT-FE-S05-15 | room 模式保存 | 构造 PUT diagram 请求头 | 携带 `X-Room-Id` / `X-Collab-Server-Rev`（值为当前 head server_rev）；非 room 模式不携带 |
| UT-FE-S05-16 | viewer 角色连接 | 触发本地编辑 | CollabClient 不入队、不发送 op 帧；只读状态保持；仍可接收 remote_op |
| UT-FE-S05-17 | WS onclose（异常断开） | 触发重连策略 | 指数退避重试 ≤5 次；重连成功后发送 `sync {lastRev}` 并 flush 离线队列；5 次失败进入 `failed` 态且队列不丢弃 |

### 4.3.2 浏览器链路用例（双上下文真实 WS）

| ID | 前置 | 操作 | 预期 |
|---|---|---|---|
| ST-FE-S05-07 | A(owner)/B(editor) 两个浏览器上下文进入同一 room | A 创建表 `orders` | B 端 500ms 内画布出现 `orders`；A 收 ack、B 收 remote_op；两端 `ot-rev` 一致；Activity 有记录；**无 S01 409 模态** |
| ST-FE-S05-08 | 双上下文在线 | B 上下文 `set_offline(true)` → B 本地编辑 → `set_offline(false)` | B 断线期间 `reconnect-banner` 可见且队列计数可见；恢复后自动 sync + flush；A 端最终可见 B 的编辑；队列清零 |
| ST-FE-S05-09 | 双上下文在线 | A 移动光标 / 选中表 | B 端画布出现 A 的远端光标/选中标识（`remote-cursor`）；B 自身光标不出现在 B 的远端层 |


## ADDED — 4.3.3 C 批 mock 降级用例取代说明（实现期补充）

> 状态更新（wire-frontend-collab-ws，2026-09-09）：生产前端已接入真实 WS，原「REST head 降级形态」的 C 批 mock 自动化（ST-S05-UI-01/02/04/06、ST-PU-24、ST-FE-ALIGN-03/04、ST-S01-409-SCOPE、ST-S01-NO-409-OT）已转为 skip，由真实双上下文链路 ST-FE-S05-07~09（`frontend-rs/tests/e2e/s05-collab.spec.ts`）与单测 UT-FE-S05-13~17 覆盖；断言语义不变（见 `core-V2-production-frontend-test-cases.md` §4.3）。

同时更新 `core-S05-test-cases.md`「可见状态与降级用例」与 `core-S01-test-cases.md`「协作模式禁 409（合同）」两处状态注记。
