# 变更提案：fix-collab-autosave-race

> module: core | created: 2026-09-10

## 变更原因
用户实测反馈（2026-09-10）：协作房间中，一人操作完、op 尚未同步到另一人界面时，两边都触发自动保存（1s debounce 全量快照 checkpoint），导致界面/持久化数据不一致。

代码排查确认根因：**房间内存在两个写者**——客户端全量快照 PUT（checkpoint）与 WS op 通道——二者的交错使持久化快照与 op log 互相覆盖：

1. **op 在途时旧快照合法覆盖**（主因）：A 的 op 还在 WS 途中（服务器 head 仍为 N），B 的 checkpoint 携带 `X-Collab-Server-Rev: N` 通过校验（现有校验只拒绝 `<`，不拒绝 `==`），B 的旧快照整体覆盖持久化 diagram；新加入/刷新的客户端全量加载到被覆盖的旧快照，**永久丢失 A 的修改**（`backend/src/diagrams_v1.rs:159-177`）。
2. **check-then-act 竞态**：rev 校验与写库不在同一事务，两个并发 checkpoint last-write-wins。
3. **快照采集时机错误**（前端）：`schedule_save` 调度时即采集快照，debounce 窗口内到达的远端 op 不在快照中。
4. **409 重试重发旧快照**：`retry_save_after_conflict` 用同一旧快照换 revision 重发，再次覆盖已合并的远端修改。

用户裁决（2026-09-10）：不做窗口修补，**从根上解决**——消除双写者架构。

## 变更类型
设计级（协作持久化架构调整：房间内事实源从「客户端快照」改为「服务端物化文档」；影响场景时序 + API 语义 + DB + 前后端代码 + 测试）。

## 变更范围
- 影响的需求文档：无
- 影响的功能规格：`core-S05-ot-collab-design.md`（checkpoint 段落语义改写为服务端物化）
- 影响的业务场景：S05（S05.5 由「周期 checkpoint（REST）」改为「op 落库即物化」；`core-S05-ot-collab.md` §7/§8 + 生产状态行）
- 影响的部署方案：无（无拓扑/配置变化）
- 影响的 API：`collab.yaml`（x-collab-checkpoint 标记 REMOVED，代之以 x-collab-materialization 说明）；`diagrams.yaml`（PUT 对 room 绑定 diagram 返回 409 USE_OP_CHANNEL；GET 对 room 绑定 diagram 返回物化文档）
- 影响的 DB 表：`room_collab_head`（新增 `doc_json TEXT` 列——房间 canonical 文档）
- 影响的编排测试：无
- 影响的 smoke 测试：无
- 影响的测试文档：`core-S05-test-cases.md`（checkpoint 相关用例替换为物化用例）；`core-V2-production-frontend-test-cases.md`（UT-FE-S05-15 checkpoint 头用例随代码移除）
- 影响的代码：`backend/src/collab/mod.rs`（物化）、`backend/src/diagrams_v1.rs`（GET 物化读 / PUT 拒绝）、`backend/migrations/0006_*`、`frontend-rs/src/editor_panels.rs`（room 模式禁用全量保存）、`frontend-rs/src/collab_client.rs`（dirty=未 ack op）、`frontend-rs/src/editor_data_access.rs`（移除 checkpoint 客户端）
- 关联提醒：S06 MCP（实现中）写 room 图须走 op 通道或非房间图，不能依赖全量 PUT

## 部署影响
- 是否需要部署：否
- 部署原因：本地 dev 双进程验证即可；无部署拓扑/配置变化
- 影响环境：本地
- 是否涉及数据迁移：是（`room_collab_head` 新增可空列 `doc_json`，启动时 `apply_migrations` 自动执行；存量行 NULL=未物化，首个 op 落库时从关系存储初始化——与现行 GET 行为同等保真度）
- 是否需要回滚预案：否（提供 `0006_*.down.sql` 降列即可）
- 是否需要 smoke：否

## 变更概述
**房间内唯一事实源 = 服务端维护的 JSON 文档（单写者、单方向数据流）**：

1. **op 落库即物化**：`append_op` 事务内增加物化步骤——读取 `room_collab_head.doc_json`（NULL 时从关系存储 `load_diagram` 初始化），按 op 类型做纯 JSON 幂等 apply（table/field/reference/area/note × create/update/delete，共 15 种；`table.update` 合并除 `fields` 外所有键，`table.create` 携带含字段全量），写回 `doc_json`。从此 op log 与持久化文档恒一致，不存在任何交错窗口。
2. **读路径切换**：`GET /diagrams/{id}` 若 diagram 绑定活跃 room 且已物化 → 直接返回 `doc_json`（注入当前 head 为 revision）；首连、刷新、SYNC_GAP_TOO_LARGE 全量重载永远读到与 op log 一致的状态。
3. **写路径收编**：`PUT /diagrams/{id}` 对 room 绑定 diagram 一律 409 `USE_OP_CHANNEL`（写只走 WS op 通道）；非房间 diagram 的 legacy PUT（S01/S02）完全不变。`x-collab-checkpoint` 协议与后端 checkpoint 分支移除。
4. **前端简化**：room 模式不再调度全量保存（ops 即保存）；`dirty` 语义改为「有未 ack / 未 flush 的 op」，Ack 清空后置 false（保存指示更准确）；移除 `save_room_checkpoint` / `CollabCheckpoint` / `retry_save_after_conflict` 协作分支 / `COLLAB_REV_STALE` 处理。

已知边界（不在本次范围）：`doc_json` 首次初始化自关系存储，继承其既有保真度（table 无 width/min_height 列），与现行 GET 行为一致；关系存储保真度提升为独立后续项。
