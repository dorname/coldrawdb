## MODIFIED — S04.1 创建协作房间

**入口**：

- `/rooms` 页「创建房间」按钮 `[data-testid="btn-create-room"]`
- 编辑器 AppBar user-menu →「为此 diagram 创建协作房间」

**Create Room 模态** `[data-testid="modal-create-room"]`：

| 字段 | 必填 | 校验 |
|---|---|---|
| room_name | ✅ | 1–64 字符 |
| diagram | ✅ | 下拉已有 diagram 或当前 diagram；选「新建空白模型」时先创建 diagram 再绑定房间 |
| database | 条件必填 | **仅当 diagram 选「新建空白模型」时露出** `[data-testid="create-room-database"]`：Generic / MySQL / PostgreSQL 三项，默认 Generic；创建 diagram 时随 `POST /api/v1/diagrams` 的 `database` 入参落账，建图即确定字段类型清单口径 |
| default_role | ❌ | 邀请默认角色 editor / viewer |

**流程**：

1. 填写表单 → 若选「新建空白模型」：先 `POST /api/v1/diagrams` `{ name, database }`（database 透传所选引擎，缺省 Generic）→ 再 `POST /api/v1/rooms` `{ name, diagram_id }`；若选既有 diagram：直接 `POST /api/v1/rooms`（引擎随既有 diagram，不在创建流程修改）
2. **201** → Toast「房间已创建」→ 跳转 `/editor/{diagramId}?room={roomId}`
3. AppBar 出现 `[data-testid="room-badge"]` + `[data-testid="btn-invite"]`；AppBar 引擎下拉显示该 diagram 的引擎，Inspector 字段类型清单按引擎过滤（衔接 diagram-database-dialect 的 `types_for_database` 通路）
4. **409** diagram 已绑定其他 room → 提示「该 diagram 已在房间 X 中」

**约束**：

- 引擎枚举与 AppBar 引擎下拉严格对齐（Generic / MySQL / PostgreSQL 三项；Sqlite/Mssql/Oracle 不露出）
- 新建图的引擎前置选择由本流程（「新建空白模型」分支）唯一承载——生产端无独立新建图入口（TopBar/TopMenuBar 为死代码，2026-09-06 勘察确认），如未来恢复独立新建图模态需同步增加 `new-diagram-database` 下拉
