# drawDB Phase 0 - 存储迁移策略文档（冻结版）

## 1. 目标
- 将 diagram 主链路从本地持久化迁移到数据库持久化。
- 在迁移窗口内保证“可灰度、可回滚、可观测”。
- 迁移完成后，数据库成为唯一真实数据源。

## 2. 迁移范围
- 数据对象：diagram / table / field / indice / reference / note / area。
- 迁移入口：
  1) 在线 API 保存。
  2) 本地 JSON 导入（历史数据）。

## 3. 迁移策略（分阶段）

### Stage A：Schema 冻结与兼容修复
1. 冻结 v1 DDL 与索引。
2. 修复历史命名不一致：`diagram_link.reference` -> `diagram_link.reference_id`。
3. 增加 `diagram.revision`（默认 0）与各核心表 `is_deleted`（默认 false）。
4. 合并时间语义：移除 `last_modified`，统一使用 `updated_at`。
5. 为 link 表补索引。

**回滚点 A**：DDL 变更可通过 down migration 回滚；业务仍走旧存储。

### Stage B：后端能力就绪
1. 上线 `GET/PUT/POST/DELETE /api/v1/diagrams`。
2. 实现单 diagram 单事务保存。
3. 实现 `expected_revision` 乐观锁冲突（409）。
4. 增加导入接口 `/api/v1/diagrams/import`。

**回滚点 B**：关闭新 API 流量开关，恢复旧前端本地保存路径。

### Stage C：桥接迁移
1. 旧前端增加“迁移到数据库”入口。
2. 先灰度读：数据库读失败时回退本地读。
3. 再灰度写：优先写数据库，可选短期双写本地（最多 4 周）。

**回滚点 C**：关闭数据库写入开关，恢复本地写路径。

### Stage D：收敛
1. 停止双写，数据库单写。
2. 老本地格式仅保留导入能力。
3. 完成数据巡检后下线旧保存逻辑。

## 4. 数据一致性策略
- 保存事务边界：diagram 全量结构在单事务提交。
- 冲突处理：revision 不一致返回 409，客户端提示刷新或覆盖。
- 幂等性：导入接口支持客户端幂等键（建议 header: `Idempotency-Key`）。

## 5. 可观测与告警
- 指标：
  - `diagram_save_success_rate`
  - `diagram_save_p95_ms`
  - `diagram_conflict_rate`
  - `diagram_import_fail_rate`
- 日志字段：`request_id`, `diagram_id`, `expected_revision`, `current_revision`, `error_code`。
- 告警阈值：
  - 保存成功率 < 99.95%（5 分钟窗口）
  - 导入失败率 > 0.1%

## 6. 验证清单（架构评审准入）
1. 能创建并读取复杂 diagram（>= 20 tables, >= 200 fields）。
2. 并发保存冲突可稳定复现并返回 409。
3. 导入历史 JSON 成功率 >= 99%。
4. 失败请求日志可按 request_id 完整追踪。

## 7. 风险与应对
- 风险：历史 JSON 字段缺失或拼写不一致。  
  应对：导入器版本识别 + 默认值补齐 + 警告列表。
- 风险：桥接期双写导致状态漂移。  
  应对：双写限时 + 以数据库为准 + 定时比对任务。
- 风险：新增索引影响写入时延。  
  应对：压测后分批上线索引。

## 8. Phase 0 退出条件
- ERD 文档冻结。
- OpenAPI 草案冻结。
- 迁移策略文档冻结。
- 架构评审会议通过（含行动项与 owner）。
- 评审决议已固化：`is_deleted`=是，`last_modified` 合并到 `updated_at`=是，`reference` 字段 FK 增强=否。
