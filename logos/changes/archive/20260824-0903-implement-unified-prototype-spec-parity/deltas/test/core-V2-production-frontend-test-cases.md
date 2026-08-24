# Delta — core-V2-production-frontend-test-cases.md（修改）

> module: core | proposal: implement-unified-prototype-spec-parity

## MODIFIED — 1. 范围

本矩阵是 **主原型能力 → 真实 REST/WS** 的逐项对齐合同。PU 矩阵继续验静态原型；本文件验 `frontend-rs` + `backend`。

状态措辞统一：**规格合同**（上一变更已收口）/ **本提案实现**（`implement-unified-prototype-spec-parity` A～D 批落实自动化）。不得将「规格已写」标为「生产已完成」。

实现阶段须将用例结果写入 `logos/resources/verify/test-results.jsonl`（OpenLogos reporter）。

## MODIFIED — 原型能力 ↔ 生产 API/WS 对齐矩阵

| 原型能力 | 生产 API / WS | 验收 ID | 状态 |
|---|---|---|---|
| auth 登录/注册 | `POST /api/v1/auth/login` · `register` | ST-FE-S03-02/03 · ST-FE-PROTO-01/02 | 本提案 A 批实现 |
| session / refresh | `POST /api/v1/auth/refresh` · `GET /me` · logout | ST-FE-S03-04/05 · UT-FE-S03-* | 本提案 A 批实现 |
| rooms 列表/创建 | `GET/POST /api/v1/rooms` | ST-FE-S04-01/02 · ST-FE-PROTO-03/04 | 本提案 B 批实现 |
| invite 预览/接受 | `GET/POST .../invites*` · `/invite/{token}` | ST-FE-S04-03/04 · ST-FE-PROTO-05 | 本提案 B 批实现 |
| 成员与角色 | members PATCH/DELETE | ST-FE-S04-05/06 | 本提案 B 批实现 |
| Viewer 只读 | REST 403 + WS `READ_ONLY` | ST-FE-S04-06 · ST-FE-S05-06 | 本提案 B/C 批实现 |
| 单人保存 / SaveState | `PUT /api/v1/diagrams/{id}` | ST-FE-V2-02 · S01 用例 | 本提案 C 批实现 |
| 非 OT 409 | PUT 409 `revision_conflict` | ST-S01-02 · UT-S01-04 | 规格合同；协作模式禁 409 模态；C 批回归 |
| 分享只读 | `GET` + `?share=` | ST-FE-V2-01 · S02 | 本提案 A 批实现 |
| WS 连接态 | `/ws/rooms/{roomId}?token=` · `connected` | ST-FE-S05-01 · `ws-status` | 本提案 C 批实现 |
| OT rev / ack / remote_op | WS frames + collab REST head/ops | ST-FE-S05-02 · `ot-rev` | 本提案 C 批实现 |
| presence | presence 帧 | ST-FE-S05-03 · `room-presence` | 本提案 C 批实现 |
| 断线队列 / 重连 | sync + 本地 queue | ST-FE-S05-04 · `reconnect-banner` | 本提案 C 批实现 |
| 仅本地编辑 | 降级 PUT（可 409） | ST-FE-S05-05 | 本提案 C 批实现 |
| IO 抽屉 | bridge import/export | ST-FE-V2-03 · PC | 本提案 D 批实现 |
| 命令面板 / 代码视图 | 前端壳层 | ST-FE-PROTO-08 子集 · KB/PE | 本提案 D 批实现 |
| 主题 / 响应式 | tokens + layout | ST-FE-PROTO-07 · PE/SP | 本提案 D 批实现 |

## MODIFIED — 既有 FE / PROTO 用例状态说明

对文档中全部 `UT-FE-*` / `ST-FE-*` / `UT-FE-PROTO-*` / `ST-FE-PROTO-*`：

- 保留用例 ID 与步骤作为**规格合同**。
- 实现与 e2e harness 由本提案 A～D 批落实；不得因 API client 已存在即勾选「生产已对齐主原型」。
- 浏览器 ST 若暂无 harness：显式 `skip` 并写明缺口，最终验收前优先打通页面流主链。

## MODIFIED — 状态机与页面流回归（合同强化）

| ID | 前置 | 操作 | 预期 | 状态 |
|---|---|---|---|---|
| ST-FE-ALIGN-01 | 未登录 | 默认入口 | 仅 auth；不拉私有 rooms | 本提案 A 批实现 |
| ST-FE-ALIGN-02 | 登录成功 | — | 进入 `rooms-list-page`，不直达 editor | 本提案 A 批实现 |
| ST-FE-ALIGN-03 | room-editor | 观察协作锚点 | `ws-status`/`ot-rev`/`room-presence` 来自真实 WS 或明确降级 | 本提案 C 批实现 |
| ST-FE-ALIGN-04 | 协作已连接 | 并发 op | **禁止**弹出 S01 409 冲突模态 | 本提案 C 批实现 |
