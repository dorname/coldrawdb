## MODIFIED — ST-S04-UI-16 — 创建房间模态删除孤儿图 e2e

- **位置**：`frontend-rs/scripts/test-spec-parity-d.mjs`
- **前置**：mock `GET /diagrams/queryAll` 返回 1 张未绑定图「孤儿图」（含完整 ID、表/关系数、更新时间）；mock `DELETE /api/v1/diagrams/{id}` 204
- **步骤**：`/rooms` → 创建房间 → 在图表选择器搜索并选择「孤儿图」→ 滚动到独立「图表管理 / 危险操作」区 → 点「删除此图表」→ 模态确认
- **断言**：删除入口不在候选列表或主提交 footer 内；确认模态含图名、完整 ID 和表/关系数；发出 DELETE；候选刷新不再含「孤儿图」；选中回退「新建空白模型」；Toast「已删除图表」

## ADDED — feat-room-diagram-picker / #56 图表选择器与个人建模

### UT-S04-21 — queryAll 聚合表/关系计数

- **位置**：`backend/src/diagrams/mod.rs::tests`，独立临时 SQLite。
- **前置**：活跃图 A 含 2 表、1 关系，活跃图 B 为空图，另有软删除图及其表/关系。
- **操作**：调用 `list_active_diagrams` 并序列化结果。
- **断言**：A 的 `table_count=2`、`reference_count=1`；B 两项均为 0；软删除图及其内容不进入结果；既有 revision/updated_at/lastModified 不回归。
- **reporter**：`crate::verify_reporter::report_pass("UT-S04-21", 0)`。

### UT-S04-UI-19 — 图表候选搜索、排序与显示纯函数

- **位置**：`frontend-rs/src/editor_panels.rs`。
- **前置**：350+ 摘要，含同名空图/完整模型、相同更新时间、空名/空时间、已绑定房间图。
- **断言**：
  1. 名称、完整 ID、ID 后缀均大小写不敏感命中；已绑定图始终排除；无命中返回空集合。
  2. 最近修改倒序，`updated_at` 缺省回退 `lastModified`；同时间按名称、ID 稳定排序。
  3. 显示包含名称、至少 6 位短 ID、revision、表/关系数和时间；空名/空时间有稳定回退。
  4. 搜索函数不修改原选中 ID；350+ 输入只在内存过滤，不产生 API 调用。

### ST-S04-UI-18 — 350+ 图表选择器与个人建模真实浏览器链路

- **位置**：`frontend-rs/scripts/test-spec-parity-d.mjs`。
- **GIVEN**：mock 350+ 图表摘要，含两个同名「博客内容管理系统」（0 表空图与多表完整模型）、不同 ID/时间，另有已绑定图。
- **WHEN / THEN**：
  1. 打开创建房间：候选按最近修改排序且列表独立滚动；搜索同名后两项可通过短 ID、表/关系数、revision 和时间区分；搜索 ID 后缀只保留目标；已绑定图不可见。
  2. 选择完整模型后，选中摘要持续可见；输入无匹配关键字出现空态但不改变已选 ID；清空恢复。
  3. 危险删除按钮位于独立管理区，不在候选容器或主提交 footer；确认上下文显示完整 ID 与规模。
  4. 关闭后进入「个人建模」，搜索并打开已有图：先 GET 完整 diagram，再进入编辑器；无 `POST /rooms`、无 room-badge。
  5. 返回后新建个人图：输入图名和 PostgreSQL，提交后 POST diagram 的 name/database 正确，再 GET 并进入个人画布；仍无 room 创建请求。
  6. 快速连续进入时沿用 enter_nonce，旧响应不得覆盖后选图。

### 用例登记追加

| ID | 标题 | 对齐实现 |
|---|---|---|
| UT-S04-21 | queryAll 聚合表/关系计数 | `backend/src/diagrams/mod.rs::list_active_diagrams` |
| UT-S04-UI-19 | 图表候选搜索、排序与显示纯函数 | `frontend-rs/src/editor_panels.rs` |
| ST-S04-UI-18 | 350+ 图表选择器与个人建模 | `frontend-rs/scripts/test-spec-parity-d.mjs` |
