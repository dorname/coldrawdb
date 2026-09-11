# Delta: core-S05-test-cases.md（fix-collab-autosave-race · 方案 B 服务端物化单写者）

## ADDED — 2.x UT 用例（服务端物化单写者）

### UT-C-06 — op 落库即物化（table.create 含字段）

- **位置**：`collab::tests` / `collab_v1::tests`
- **前置**：room 绑定 diagram，doc_json 为 NULL
- **操作**：append_op(table.create，changes 含 fields)
- **断言**：同事务内 doc_json 初始化（自关系存储）并包含新表及其字段；`head.server_rev` 递增；再次读取 doc 与 op log 重放结果一致

### UT-C-07 — 15 种 op 物化正确性与幂等

- **位置**：`collab::tests::ut_c07_apply_op_to_doc_*`
- **覆盖**：table/field/reference/area/note × create/update/delete
- **断言**：create=upsert 全量；table.update 合并除 fields 外键且不触碰 fields；field.* 依 parentId 定位；delete 幂等（重复 delete 不报错）；未知 type 返回 false 不改变 doc

### UT-C-08 — room 绑定 diagram 的 PUT 写收编

- **位置**：`diagrams_v1::tests` / `collab_v1::tests`
- **断言**：PUT room 绑定 diagram → 409 `USE_OP_CHANNEL`（无论是否携带旧 checkpoint 头）；非 room diagram PUT 正常（legacy 回归）

### UT-C-09 — GET 物化读与 op log 一致

- **位置**：`diagrams_v1::tests` / `collab_v1::tests`
- **步骤**：room 图 append 若干 op 后 GET /diagrams/{id}
- **断言**：返回物化 doc（含全部 op 效果）；revision == 当前 head server_rev；未物化（无 op）时回退关系存储

## ADDED — 2.x UT 用例（前端 ops 即保存）

### UT-FE-S05-18 — room 模式不调度全量保存

- **位置**：`editor_panels` 测试模块
- **断言**：room+connected 时 schedule_save 不产生任何 PUT（无 debounce 回调）；非 room 路径行为不变（legacy 回归）

### UT-FE-S05-19 — Ack 清空后 dirty=false

- **位置**：`collab_client` 测试模块
- **断言**：本地编辑 → dirty=true；全部 op ack 且离线队列空 → dirty=false；存在 pending/queued 时保持 true

### UT-FE-S05-20 — RemoteOp 有序收编与 sync 缓冲回放（入房窗口不丢 op）

- **位置**：`collab_client` 测试模块（`plan_remote_op` / `drain_buffered_ops` 纯函数）
- **背景**：入房「先 REST 加载（revision=doc_rev）后连 WS（head=server_rev）」存在提交窗口，窗口内 op 早于连接不会被广播补发；且 sync 在途与广播到达交错时旧状态会覆盖新状态（ST-FE-S05-10 首轮复现 C 丢 t_from_a 的根因）
- **断言**：rev==cur+1 → Apply；rev<=cur → SkipStale；rev>cur+1 → BufferAndSync；sync 在途一律 Buffer；drain 对乱序/重复缓冲有序去重回放，仍有缺口时保留剩余并要求再次 sync

## REMOVED — UT-FE-S05-15（checkpoint 头构造）

移除原因：checkpoint 协议废弃，`checkpoint_headers` / `save_room_checkpoint` 随代码移除；
原「协作已连接时 PUT 携带 X-Room-Id / X-Collab-Server-Rev」断言由 UT-C-08（服务端 409 写收编）取代。

## ADDED — 3.x ST 用例（端到端一致性）

### ST-FE-S05-10 — 并发编辑后新加入者恒一致（e2e）

- **位置**：`frontend-rs/tests/e2e/s05-collab.spec.ts`
- **步骤**：A/B 同房同时编辑（制造 op 在途交错）；随后新上下文 C 进房；A 刷新页面
- **断言**：C 与刷新后的 A 均看到 A+B 全部修改（`ot-rev` 一致；实体齐全）；全程无任何全量 PUT 请求（网络抓包断言）

## ADDED — UT-C-10（verify 修复轮：fields.reorder 物化）

### UT-C-10 — fields.reorder 物化与幂等

- **位置**：`collab::tests::ut_c10_fields_reorder_materializes`
- **前置**：doc 含表 t1（fields=[f1,f2,f3]）
- **操作**：apply fields.reorder{targetId:t1, changes.fieldIds=[f3,f1,f2]}
- **断言**：fields 重排为 [f3,f1,f2]；重复 apply 幂等；fieldIds 缺失既有字段时该字段
  保持原相对序追加末尾；目标表不存在返回 false 不改 doc（fix-collab-autosave-race）


## ADDED — PV 登记表补登（verify 修复轮第三轮）

- `core-PV-verify-pre-run-test-cases.md` 补登 UT-C-06~10 表行（本提案新增的后端物化/
  写收编用例此前只在 S05 文件以标题形式存在，账本校验器只认表行）。
- `frontend-rs/tests/openlogos_reporter.rs` 清单移除 UT-FE-S05-15（checkpoint 协议废弃后
  残留的上报 ID，否则账本报「未登记 reporter ID」）；补登 UT-FE-S05-18~21（同次 cargo
  test 覆盖的 src 单测）与 ST-FE-S05-10（s05-collab.spec.ts 真实链路，声明式保底覆盖，
  与 ST-FE-S05-07~09 同模式）——否则账本报「未覆盖自动化用例」。
