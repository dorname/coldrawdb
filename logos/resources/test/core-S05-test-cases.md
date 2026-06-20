## 1. 范围

本文件覆盖场景 S05（OT 实时协作）的 UT 与 ST 用例规格。

**对应实现**：`backend/src/collab_v1.rs` + `backend/src/collab/*`

**API 契约**：`logos/resources/api/collab.yaml`

**DDL**：`logos/resources/database/coldrawdb-v2-collab.sql`

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
