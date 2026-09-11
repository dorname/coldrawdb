# Delta: core-V2-production-frontend-test-cases.md（fix-collab-autosave-race · verify 修复轮）

## ADDED — UT-FE-S05-21（fields.reorder diff 产出与远端应用）

| 用例 ID | 前置 | 操作 | 预期结果 |
|---|---|---|---|
| UT-FE-S05-21 | 表含字段 [f1,f2] | 交换字段顺序后 diff；构造 fields.reorder remote_op 应用到 store | diff 产出 `fields.reorder`（targetId=表 id，changes.fieldIds=全量目标序），排在 field create/delete 之后；整表 create/delete 当次不重复产出 reorder；apply_op_to_store 按 fieldIds 重排且不产生回声（fix-collab-autosave-race，ST-SP-LIST-02 暴露的 op 模型缺口） |


## ADDED — ST-FE-S05-11（远端帧不吞 debounce 窗口内本地编辑，verify 修复轮第三轮）

| 用例 ID | 前置 | 操作 | 预期结果 |
|---|---|---|---|
| ST-FE-S05-11 | 单上下文在房 + mock WS 已连接且首连 sync 完成 | 建表后 <200ms（debounce 窗口内）注入远端 `note.create` 广播 | 远端帧到达前本地未上行编辑先被结算成 op：收敛后服务端物化文档同时含本地 `table_1` 与远端 note；旧实现 baseline 重采会吞掉 table.create（ST-CR-INSP-01 间歇失败根因回归） |
