# 实现任务

## [delta] 规格变更
- [x] 产出 delta 文件到 `deltas/api/collab.yaml` — REMOVED x-collab-checkpoint；ADDED x-collab-materialization（op 落库即物化 + GET 物化读 + PUT 409 USE_OP_CHANNEL）
- [x] 产出 delta 文件到 `deltas/api/diagrams.yaml` — PUT/GET 对 room 绑定 diagram 的语义说明
- [x] 产出 delta 文件到 `deltas/database/coldrawdb-v2-collab.sql` — room_collab_head 新增 doc_json 列
- [x] 产出 delta 文件到 `deltas/prd/3-technical-plan/2-scenario-implementation/core-S05-ot-collab.md` — S05.5 改写为「op 落库即物化」+ 生产状态行
- [x] 产出 delta 文件到 `deltas/test/core-S05-test-cases.md` — checkpoint 用例替换为物化/写收编/读切换用例

## [code] 代码实现
- [x] 后端 migration：`backend/migrations/0006_collab_doc_json.{up,down}.sql`（room_collab_head 加 doc_json 可空列）
- [x] 后端：`collab/mod.rs` 新增 `apply_op_to_doc` 纯函数（15 种 op 幂等物化）+ `append_op` 事务内物化（doc_json NULL 时从 load_diagram 初始化）+ `get_room_document` / `room_id_for_diagram` 查询
- [x] 后端：`diagrams_v1.rs` GET room 绑定 → 返回 doc_json（revision=head）；PUT room 绑定 → 409 USE_OP_CHANNEL；移除 checkpoint 头分支；非 room 路径不变
- [x] 后端测试：UT-C-06 物化（table.create 含字段）、UT-C-07 15 种 op 物化+幂等、UT-C-08 PUT 409 USE_OP_CHANNEL、UT-C-09 GET 物化读（登记 OpenLogos reporter）
- [x] 前端：`editor_panels.rs` schedule_save room+connected 直接返回不调度；移除 CollabCheckpoint/CollabRevStale/retry_save_after_conflict 协作分支（legacy 409 模态与离线重试保留）
- [x] 前端：`collab_client.rs` Ack 后 pending+queue 空 → store.dirty.set(false)；移除 checkpoint_headers（无调用方）
- [x] 前端：`editor_data_access.rs` 移除 save_room_checkpoint / CollabCheckpoint 结构
- [x] 前端测试：UT-FE-S05-18 room 模式不调度全量保存、UT-FE-S05-19 ack 清空后 dirty=false（登记 OpenLogos reporter）；ST-FE-S05-10 e2e（双上下文 + 新加入者一致性）
- [x] 前端：`collab_client.rs` 入房窗口补齐——首次 connected 若 server_rev > doc_rev 主动 sync；RemoteOp 有序收编（plan_remote_op）+ sync 在途缓冲回放（drain_buffered_ops）；UT-FE-S05-20（ST-FE-S05-10 首轮复现 C 丢 op 的根因修复）
- [x] verify 修复轮：mock 协作 WS 基建（mock-collab-ws.mjs，applyOpToDoc 镜像后端物化）+ C/D/E/F 批断言迁移到 op 物化事实源
- [x] verify 修复轮：修复 editor_panels 遗留双 op 通道（表创建/拖动直接 enqueue 无帧 pending op，ack 永不到位 → dirty 卡死）；op 唯一通道 = watcher → emit_local_ops
- [x] verify 修复轮：fields.reorder（第 16 种 op）补齐字段换序缺口（ST-SP-LIST-02 暴露）——规格 delta + UT-C-10 / UT-FE-S05-21
- [x] verify 修复轮：diff baseline 前置到入房加载完成（refresh_baseline）+ Connected 收编语义修正（保留离线队列/在途 pending，重连 sync 起点取断连前 rev）——ST-S01-SS-01 / ST-S05-UI-05 修复
- [x] verify 修复轮：在途未 ack 帧登记 inflight_frames，断连/替换回队重发 + flush_queue 失败不丢帧——ST-PC-06 间歇丢 op 修复
- [x] verify 修复轮：远端帧（remote_op/sync/全量重同步）处理前 flush_pending_local_ops 结算 debounce 窗口内本地编辑，防止 baseline 重采吞 op——ST-CR-INSP-01 间歇失败根因修复 + 回归用例 ST-FE-S05-11（含 canvas-perf/D 批失败路径 collab-dump 诊断）
- [x] verify 修复轮：账本登记修正——PV 登记表补登 UT-C-06~10 表行；openlogos_reporter.rs 移除残留的 UT-FE-S05-15（checkpoint 废弃后账本校验报「未登记 reporter ID」）
- [x] verify 修复轮：openlogos_reporter.rs 补登 UT-FE-S05-18~21 + ST-FE-S05-10（账本「未覆盖自动化用例」修正）
