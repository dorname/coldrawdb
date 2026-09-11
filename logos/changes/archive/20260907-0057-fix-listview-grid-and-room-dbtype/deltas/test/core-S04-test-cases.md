## ADDED — UI / 页面流用例（追加行）

| 用例 | 前置 | 步骤 | 预期 | 实现批次 |
|---|---|---|---|---|
| ST-S04-UI-09 | 已登录 | 创建房间 → diagram 选「新建空白模型」→ 引擎选 MySQL → 创建并进入 | `create-room-database` 仅在选「新建空白模型」时可见；`POST /diagrams` 请求体 `database='mysql'`；进入后 AppBar 引擎 = MySQL；Inspector 字段类型含 `DATETIME` 且无 `SERIAL` | fix-listview-grid-and-room-dbtype |
| ST-S04-UI-11 | 已登录 | 创建房间/新建图不触碰引擎下拉（默认 Generic） | `POST /diagrams` 不带 `database` 或为 NULL；行为与现状一致（回归不破） | fix-listview-grid-and-room-dbtype |

## REMOVED — UI / 页面流用例（ST-S04-UI-10 行）

ST-S04-UI-10（新建图模态引擎选 PostgreSQL）**删除**。实现期勘察发现：新建图模态（`ModalKind::New`）的唯一触发点 TopBar/TopMenuBar 是死代码（生产端无独立编辑器页，图只在房间内编辑），且 `on_new_diagram` 的 set_href 导航在无 room 参数时落回房间列表页——该流程本身是断的。经用户决策（2026-09-06）：独立新建图入口撤销，引擎前置选择由「创建房间 → 新建空白模型」唯一活路径覆盖。
