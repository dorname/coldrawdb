## MODIFIED — S04.1 创建协作房间

**入口**：

- `/rooms` 页「创建房间」按钮 `[data-testid="btn-create-room"]`
- 编辑器 AppBar user-menu →「为此 diagram 创建协作房间」
- `/rooms` 页另有「个人建模」按钮 `[data-testid="btn-personal-modeling"]`，打开同一图表选择器但不创建 room，见 S04.1A

**Create Room 模态** `[data-testid="modal-create-room"]`：

| 字段 | 必填 | 校验 |
|---|---|---|
| room_name | ✅（协作模式） | 1–64 字符；个人模式不显示 |
| diagram | ✅ | **可搜索图表选择器** `[data-testid="create-room-diagram"]`，替代 350+ 项原生下拉。数据源 `GET /diagrams/queryAll` 只返回未软删图，前端继续排除已绑定活跃房间的图。提供「新建空白模型」固定项 `[data-testid="create-room-diagram-new"]` |
| database | 条件必填 | **仅当 diagram 选「新建空白模型」时露出** `[data-testid="create-room-database"]`：Generic / MySQL / PostgreSQL 三项，默认 Generic；创建 diagram 时随 `POST /api/v1/diagrams` 的 `database` 入参落账，建图即确定字段类型清单口径 |
| default_role | ❌（协作模式） | 邀请默认角色 editor / viewer；个人模式不显示 |

**图表选择器（feat-room-diagram-picker / #56）**：

- 搜索框 `[data-testid="create-room-diagram-search"]` 按名称和完整/后缀 ID 做大小写不敏感包含匹配，清空恢复全部；无结果显示 `[data-testid="create-room-diagram-empty"]`，不展示陈旧候选。
- 候选按 `COALESCE(updated_at,lastModified)` 最近修改倒序，时间相同按名称、ID 稳定排序；不依赖数据库返回顺序。
- 每个候选为 `[data-testid="create-room-diagram-option-<id>"]`，显示：名称、至少 6 位 ID 后缀、revision、表数量、关系数量和最近修改时间。空名统一显示「未命名模型」，空时间显示「时间未知」，同名图必须可通过元数据区分。
- 当前选中项在搜索区外持续显示 `[data-testid="create-room-diagram-selected"]`，包含完整名称与 ID；搜索只过滤候选，不隐式改变 `diagram_choice`，避免无结果时误用不可见旧选择。
- 候选列表独立纵向滚动，350+ 项不得撑高模态；键盘可操作，选项以 `role="option"` 与 `aria-selected` 表达选中态。
- `queryAll` 响应在既有字段上新增 `table_count`、`reference_count`，由后端同一条只读 SQL 聚合计数；不得为每项额外发起 GET，也不得用 0 伪造未知计数。
- **危险操作隔离**：删除入口不再紧贴选择器或主提交按钮。仅在选中已有图时，于表单下部的独立「图表管理 / 危险操作」区 `[data-testid="create-room-diagram-management"]` 显示 `[data-testid="create-room-diagram-delete"]`。点击仍打开既有二次确认模态 `[data-testid="modal-delete-diagram"]`；确认信息必须含图名、完整 ID、表/关系数，防止同名误删。确认调 `DELETE /api/v1/diagrams/{id}`，成功后移出候选、选中回退「新建空白模型」并 Toast「已删除图表」。

**创建协作房间流程**：

1. 填写表单 → 若选「新建空白模型」：先 `POST /api/v1/diagrams` `{ name, database }`（database 透传所选引擎，缺省 Generic）→ 再 `POST /api/v1/rooms` `{ name, diagram_id }`；若选既有 diagram：直接 `POST /api/v1/rooms`（引擎随既有 diagram，不在创建流程修改）
2. **201** → Toast「房间已创建」→ 跳转 `/editor/{diagramId}?room={roomId}`
3. AppBar 出现 `[data-testid="room-badge"]` + `[data-testid="btn-invite"]`；AppBar 引擎下拉显示该 diagram 的引擎且**禁用（只读标识，见约束）**，Inspector 字段类型清单按引擎过滤（衔接 diagram-database-dialect 的 `types_for_database` 通路，清单口径见 `core-01a-table-and-field.md` §2.2 MVP 短清单）
4. **409** diagram 已绑定其他 room → 提示「该 diagram 已在房间 X 中」

### S04.1A 个人建模（feat-room-diagram-picker / #56）

- `/rooms` 页「个人建模」与「创建房间」并列但视觉上次级，复用图表搜索、排序、元数据和危险操作隔离，不强迫用户先建协作房间。
- 个人模式不显示房间名称和默认邀请角色。选择「新建空白模型」时显示图名 `[data-testid="personal-diagram-name"]`（trim 后 1–64 字符）与数据库引擎；选择已有图时直接打开。
- 新建：`POST /api/v1/diagrams` 成功后取得 ID；打开：使用已选 ID。两者均先 `GET /api/v1/diagrams/{id}` 完成加载，再进入个人编辑器；沿用 `enter_nonce` 的「先加载后切页 + 最后进入者生效」守卫，避免快速连点被旧响应覆盖。
- 进入时 `current_room=None`，不得调用 `POST /api/v1/rooms`，不得显示 room-badge；后续保存走 S01 私有 PUT 通路。已绑定活跃房间的图不出现在个人候选中，防止以私有写通路打开后遭遇 409 `USE_OP_CHANNEL`。
- 主提交按钮在个人模式显示「打开个人画布」；创建中禁用重复提交，失败保留模态并显示错误，不产生半进入状态。

**约束**：

- 引擎枚举与 AppBar 引擎下拉严格对齐（Generic / MySQL / PostgreSQL 三项；Sqlite/Mssql/Oracle 不露出）
- **引擎创建后锁定**：diagram 引擎在创建时确定（协作或个人「新建空白模型」分支，或既有 diagram 自带），进入编辑器后 AppBar `[data-testid="app-db-select"]` 禁用，仅作引擎只读标识；中途切换会使既有字段类型与新引擎清单脱节，故不开放（fix-pg-types-listview-zindex-lock-engine）
- 新建图的引擎前置选择由协作/个人共享选择器承载；如未来新增其他新建图入口，必须复用同一引擎口径
