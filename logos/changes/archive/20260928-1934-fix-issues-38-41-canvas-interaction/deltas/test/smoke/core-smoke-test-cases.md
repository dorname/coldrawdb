# Delta — core-smoke-test-cases.md（fix-issues-38-41-canvas-interaction）

> 模块：core | 提案：fix-issues-38-41-canvas-interaction
> 关联 issue：#41

## ADDED — 8.8 SMOKE-core-09 — 区域锁定持久化（area.locked）

验证 migration 0012_area_lock 生效 + locked 经 PUT 快照通路落库（#41 R-AREALOCK-01/06）。**diagram-api-auth：写操作携带 `Authorization: Bearer <smoke_token>`。**

**步骤**：
1. 创建 diagram，PUT 快照含 `areas: [{id, x, y, width, height, name, locked: true}]`
2. GET 读回该 diagram
3. 断言 `areas[0].locked == true`（migration 后列存在且值透传）
4. 再 PUT `locked: false` → GET 断言 `locked == false`（往返可写）

**通过条件**：两次 GET 的 locked 值与写入一致；无 500/422；旧图（无 locked 键）读取不报错且等价 false。

## MODIFIED — 附录 A：用例 ID 清单（追加行）

| SMOKE-core-09 | 区域锁定持久化（migration 0012 + PUT 快照 locked 往返） |
