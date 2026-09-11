## 1. 范围

S05：OT 实时协作。关键可见态：`ws-status`、`ot-rev`、presence、reconnect、queue、local-only。

状态：后端已实现；生产前端已接入真实 WS（`wire-frontend-collab-ws`：连接 / op 双向 / presence / 重连 sync / checkpoint 头 / viewer 只读）。可见状态与降级用例自动化结果写入 `logos/resources/verify/test-results.jsonl`。不得将「规格已写」标为「生产已完成」。

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

### UT-C-06 — op 落库即物化（table.create 含字段）

- **位置**：`diagrams_v1::tests::ut_c06_append_op_materializes_doc`
- **前置**：room 绑定 diagram，doc_json 为 NULL
- **操作**：append_op(table.create，changes 含 fields)
- **断言**：同事务内 doc_json 初始化（自关系存储）并包含新表及其字段；`head.server_rev` 递增；再次读取 doc 与 op log 重放结果一致（fix-collab-autosave-race）

### UT-C-07 — 15 种 op 物化正确性与幂等

- **位置**：`collab::tests::ut_c07_table_lifecycle` / `ut_c07_field_lifecycle` / `ut_c07_flat_collections_and_unknown`
- **覆盖**：table/field/reference/area/note × create/update/delete
- **断言**：create=upsert 全量；table.update 合并除 fields 外键且不触碰 fields；field.* 依 parentId 定位；delete 幂等（重复 delete 不报错）；未知 type 返回 false 不改变 doc（fix-collab-autosave-race）

### UT-C-08 — room 绑定 diagram 的 PUT 写收编

- **位置**：`diagrams_v1::tests::ut_c08_room_bound_put_rejected`
- **断言**：PUT room 绑定 diagram → 409 `USE_OP_CHANNEL`（无论是否携带旧 checkpoint 头）；非 room diagram PUT 正常（legacy 回归）（fix-collab-autosave-race）

### UT-C-09 — GET 物化读与 op log 一致

- **位置**：`diagrams_v1::tests::ut_c09_get_returns_materialized_doc`
- **步骤**：room 图 append 若干 op 后 GET /diagrams/{id}
- **断言**：返回物化 doc（含全部 op 效果）；revision == 当前 head server_rev；未物化（无 op）时回退关系存储（fix-collab-autosave-race）

### UT-C-10 — fields.reorder 物化与幂等

- **位置**：`collab::tests::ut_c10_fields_reorder_materializes`
- **前置**：doc 含表 t1（fields=[f1,f2,f3]）
- **操作**：apply fields.reorder{targetId:t1, changes.fieldIds=[f3,f1,f2]}
- **断言**：fields 重排为 [f3,f1,f2]；重复 apply 幂等；fieldIds 缺失既有字段时该字段
  保持原相对序追加末尾；目标表不存在返回 false 不改 doc（fix-collab-autosave-race）

## 3. ST 用例

### ST-C-01 — 完整 OT 协作链路

- **位置**：`collab_v1::tests::st_c01_ot_collab_flow`
- **步骤**：对齐 `core-S05-ot-collab.json` 主链路（head → WS op → remote_op → REST ops → sync → viewer READ_ONLY）
- **断言**：serverRev 最终为 1；viewer op 未写入

### ST-FE-S05-10 — 并发编辑后新加入者恒一致（e2e，fix-collab-autosave-race）

- **位置**：`frontend-rs/tests/e2e/specs/s05-collab.spec.ts`（见 `core-V2-production-frontend-test-cases.md` §4.3.2）
- **步骤**：A/B 同房同时编辑（制造 op 在途交错）；随后新上下文 C 进房；A 刷新页面重进
- **断言**：C 与刷新后的 A 均看到 A+B 全部修改（物化读与 op log 恒一致）；全程无任何全量 PUT 请求（网络抓包断言）

## 可见状态与降级用例

| ID | 前置 | 操作 | 预期 | 状态 |
|---|---|---|---|---|
| ST-S05-UI-01 | Owner/Editor 进房 | 建立 WS | `ws-status` 已连接；收到 `connected`；`ot-rev` 显示 serverRev | 本提案 C 批实现 |
| ST-S05-UI-02 | 两用户在线 | A 创建表 | A ack；B remote_op；两端 `ot-rev` 一致；Activity 有记录；**无 S01 409 模态** | 本提案 C 批实现 |
| ST-S05-UI-03 | 两用户在线 | A 移动光标/选中 | B 见 `room-presence` / remote-cursor；不遮挡本地选中 | 本提案 C 批实现 |
| ST-S05-UI-04 | 连接中断 | 本地继续编辑 | 队列计数可见；`reconnect-banner`；重连 sync 后队列清零 | 本提案 C 批实现 |
| ST-S05-UI-05 | 重连失败 | 选择仅本地编辑 | 明确 409 风险；本地可编辑；不误报 OT 已同步；fix-collab-autosave-race 起不产生全量 PUT、不出现 409 模态（op 排队待重连 flush） | 本提案 C 批实现 |
| ST-S05-UI-06 | Viewer | 尝试发 op | 前端不发送；或 READ_ONLY；head/`ot-rev` 不因本地写递增 | 本提案 C 批实现 |


> 状态更新（wire-frontend-collab-ws，2026-09-09）：生产前端已接入真实 WS，原「REST head 降级形态」的 C 批 mock 自动化（ST-S05-UI-01/02/04/06、ST-PU-24、ST-FE-ALIGN-03/04、ST-S01-409-SCOPE、ST-S01-NO-409-OT）已转为 skip，由真实双上下文链路 ST-FE-S05-07~09（`frontend-rs/tests/e2e/s05-collab.spec.ts`）与单测 UT-FE-S05-13~17 覆盖；断言语义不变（见 `core-V2-production-frontend-test-cases.md` §4.3）。

> 状态更新（fix-collab-autosave-race，2026-09-10）：协作持久化改为服务端物化单写者（op 落库即物化 doc_json）。新增 UT-C-06~10（物化/写收编/物化读/fields.reorder 换序物化）、UT-FE-S05-18~21（room 不调度全量保存 / ack 清 dirty / RemoteOp 有序收编+入房补齐 / fields.reorder diff 产出与远端应用）、ST-FE-S05-10（并发后新加入者恒一致+零全量 PUT）；UT-FE-S05-15（checkpoint 头构造）随 checkpoint 协议废弃移除。

## 既有 S05 用例补充约束

保留帧级断言；补充前端必须绑定上表锚点。协作成功路径禁止 S01 409 模态（与 S01 交叉用例 `ST-S01-NO-409-OT`）。本提案 C 批负责落实上表自动化。
