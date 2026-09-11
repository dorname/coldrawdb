# 变更提案：wire-frontend-collab-ws

> module: core | created: 2026-09-09

## 变更原因
用户实测反馈：两个账户进入同一协作房间后画布互不可见对方操作（2026-09-09 截图佐证：A 侧移动便签/表后 B 侧无变化）。

排查结论：S05 规格（`core-S05-ot-collab.md`）、WS 协议（`collab.yaml`）与后端实现（WS hub + OT + `operation_log`，UT-C-01~05 / ST-C-01 已通过）均已就绪，但**生产前端从未建立真实 WebSocket**：

- 全代码库无 `WebSocket::new`；`build_ws_url()` / `parse_collab_frame()` 仅被单测引用，生产零调用
- 进房间仅 `GET /collab/head` 后置状态为"已连接"（`editor_panels.rs`），UI 显示协作正常但实际无通道
- 本地编辑不生成/上行任何 op；远端 `remote_op` 无应用路径；无轮询兜底
- 前端已具备半成品基建：`CollabOtState`（连接态/离线队列/ack 对账）、presence UI 锚点（`ws-status`/`ot-rev`/`remote-cursor`/`activity-feed`/`reconnect-banner`）

属于既有规格的实现缺口补齐，非新需求。

## 变更类型
代码级（按既有 S05 规格补齐前端实现 + 测试用例补充）。规格链（场景时序图 / collab.yaml / DB DDL）不变。

## 变更范围
- 影响的需求文档：无
- 影响的功能规格：无（S05 设计不变）
- 影响的业务场景：S05（§0 生产状态行需更新："前端已有部分协作接入" → "前端已接入"）
- 影响的部署方案：无（无拓扑变化；dev 双进程下前端经 `COLDRAWDB_API_BASE` 直连后端 `:3000`，`build_ws_url` 已验证派生 `ws://127.0.0.1:3000/ws/rooms/...` 形态正确）
- 影响的 API：无（`collab.yaml` 帧契约不变；需核对后端 diagrams PUT 对 `X-Room-Id` / `X-Collab-Server-Rev` 头的处理是否已落地，缺失则补齐）
- 影响的 DB 表：无
- 影响的编排测试：无
- 影响的 smoke 测试：无
- 影响的测试文档：`core-S05-test-cases.md`（新增前端接入 UT/ST 用例）
- 影响的代码：`frontend-rs/src/`（新增 CollabClient WS 模块；`editor_panels.rs` 房间进入接线；编辑→op 映射；remote_op 应用；presence 收发与远端光标渲染；checkpoint 头；viewer 只读）；`backend/src/`（仅当 checkpoint 头处理缺失时补齐）

## 部署影响
- 是否需要部署：否
- 部署原因：纯前端功能补齐（+可能的后端小补丁），本地 dev 验证即可；无部署拓扑/配置变化
- 影响环境：本地
- 是否涉及数据迁移：否
- 是否需要回滚预案：否
- 是否需要 smoke：否

## 变更概述
按 S05 既有规格完成生产前端 WS 协作接入（完整对齐，含 presence 与 checkpoint）：

1. **连接生命周期**：room-editor 挂载后 `CollabClient.connect(roomId, token)` 建立真实 WS（`build_ws_url` 接线）；处理 `connected` 帧（server_rev / members / yourRole）；`4401/4403/4409` 关闭码映射（Toast / 只读降级 / 回 rooms）。
2. **op 双向通道**：本地编辑（table/field/reference/area/note 15 种 op 类型）→ optimistic apply + `op` 帧上行 + `ack` 对账（`CollabOtState` 队列）；`remote_op` → 应用进 EditorStore 并渲染 + activity-feed 记录。
3. **presence**：光标/选区帧上行（节流）与远端光标渲染（`remote-cursor` 锚点，排除本人）。
4. **断线重连**：onclose → Banner；离线编辑入可见队列（数量可见）；指数退避重连 ≤5 次；重连后 `sync { lastRev }` 补发（缺口大时全量 snapshot 替换）；`token_expired` → S03 refresh 后重连。
5. **checkpoint**：room 内 PUT diagram 携带 `X-Room-Id` / `X-Collab-Server-Rev`，不弹 S01 409 模态；`COLLAB_REV_STALE` 走静默重同步。
6. **viewer 只读**：不入队、不发送 op，仍收 presence/remote_op。

测试：新增前端接入 UT（op 映射、帧处理、重连状态机接线）与 ST（双浏览器真实 WS：A 建表 B 端 500ms 内可见；断线队列重连 flush；presence 光标可见），全部登记 OpenLogos reporter。
