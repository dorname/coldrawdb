## 1. 范围

S05：OT 实时协作。关键可见态：`ws-status`、`ot-rev`、presence、reconnect、queue、local-only。

状态：后端已实现；生产前端部分接入；逐项对齐待第二阶段。实现阶段须将用例结果写入 `logos/resources/verify/test-results.jsonl`（OpenLogos reporter）；本提案仅规格收口，不执行自动化。

## 2. UT 用例

### UT-C-01 — WS connect 收到 connected 帧

- **位置**：`collab_v1::tests::ut_c01_ws_connected_frame`
- **断言**：`type=connected`；`serverRev=0`；`yourRole=owner`

### UT-C-02 — op → ack + remote_op

- **位置**：`collab_v1::tests::ut_c02_op_ack_and_remote_op`
- **断言**：Owner 发 op 收 ack rev=1；Guest 收 remote_op

### UT-C-03 — 连续两 op 递增 serverRev

- **位置**：`collab_v1::tests::ut_c03_sequential_ops_increment_rev`
- **断言**：两次 op 后 head.serverRev=2

### UT-C-04 — sync 补发 / 空 batch

- **位置**：`collab_v1::tests::ut_c04_sync_catch_up`
- **断言**：sync lastRev=1 返回 ops 空数组；REST ops afterRev=0 返回 1 条

### UT-C-05 — viewer 发 op → READ_ONLY

- **位置**：`collab_v1::tests::ut_c05_viewer_read_only`
- **断言**：error 帧 `code=READ_ONLY`；head 不递增

## 3. ST 用例

### ST-C-01 — 完整 OT 协作链路

- **位置**：`collab_v1::tests::st_c01_ot_collab_flow`
- **步骤**：对齐 `core-S05-ot-collab.json` 主链路（head → WS op → remote_op → REST ops → sync → viewer READ_ONLY）
- **断言**：serverRev 最终为 1；viewer op 未写入

## 可见状态与降级用例

| ID | 前置 | 操作 | 预期 | 变更 |
|---|---|---|---|---|
| ST-S05-UI-01 | Owner/Editor 进房 | 建立 WS | `ws-status` 已连接；收到 `connected`；`ot-rev` 显示 serverRev | ADDED |
| ST-S05-UI-02 | 两用户在线 | A 创建表 | A ack；B remote_op；两端 `ot-rev` 一致；Activity 有记录；**无 S01 409 模态** | ADDED |
| ST-S05-UI-03 | 两用户在线 | A 移动光标/选中 | B 见 `room-presence` / remote-cursor；不遮挡本地选中 | ADDED |
| ST-S05-UI-04 | 连接中断 | 本地继续编辑 | 队列计数可见；`reconnect-banner`；重连 sync 后队列清零 | ADDED |
| ST-S05-UI-05 | 重连失败 | 选择仅本地编辑 | 明确 409 风险；本地可编辑；不误报 OT 已同步 | ADDED |
| ST-S05-UI-06 | Viewer | 尝试发 op | 前端不发送；或 READ_ONLY；head/`ot-rev` 不因本地写递增 | ADDED |

## 既有 S05 用例补充约束

保留帧级断言；补充前端必须绑定上表锚点。协作成功路径禁止 S01 409 模态（与 S01 交叉用例 `ST-S01-NO-409-OT`）。
