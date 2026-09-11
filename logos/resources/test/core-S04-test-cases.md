## 1. 范围

S04：房间列表、创建、邀请、成员、Viewer。页面锚点对齐主原型 `rooms-list-page` / `invite-accept-page` / `room-editor-page`。

状态：后端已实现；生产前端部分接入。本提案 `implement-unified-prototype-spec-parity`（B 批）将 UI / 页面流用例落实为自动化，结果写入 `logos/resources/verify/test-results.jsonl`。不得将「规格已写」标为「生产已完成」。

## 2. UT 用例

### UT-S04-01 — 创建 room 201

- **位置**：`rooms_v1::tests::ut_s04_01_create_room_success`
- **步骤**：Owner 注册登录 → 创建 diagram → POST `/api/v1/rooms`
- **断言**：201；`ownerId` 匹配；`diagramId` 匹配

### UT-S04-02 — 同一 diagram 重复创建 409

- **位置**：`rooms_v1::tests::ut_s04_02_create_room_diagram_taken`
- **断言**：409；`code == ROOM_DIAGRAM_TAKEN`；`existingRoomId` 存在

### UT-S04-03 — diagram 不存在 404

- **位置**：`rooms_v1::tests::ut_s04_03_create_room_diagram_not_found`
- **断言**：404；`code == DIAGRAM_NOT_FOUND`

### UT-S04-04 — 创建邀请 201

- **位置**：`rooms_v1::tests::ut_s04_04_create_invite_success`
- **断言**：201；`token` / `inviteUrl` / `expiresAt` 存在

### UT-S04-05 — 匿名 preview 邀请 200

- **位置**：`rooms_v1::tests::ut_s04_05_preview_invite_success`
- **断言**：200；`roomName` / `diagramId` / `role` 正确

### UT-S04-06 — 接受邀请加入 200

- **位置**：`rooms_v1::tests::ut_s04_06_accept_invite_success`
- **断言**：200；Guest 成为 editor；成员数 2

### UT-S04-07 — 非成员访问 room 403

- **位置**：`rooms_v1::tests::ut_s04_07_get_room_not_a_member`
- **断言**：403；`code == NOT_A_MEMBER`

### UT-S04-08 — Owner 移除成员 204

- **位置**：`rooms_v1::tests::ut_s04_08_remove_member_success`
- **断言**：204；被移除用户 GET room → 403

### UT-S04-09 — Owner 不能离开 409

- **位置**：`rooms_v1::tests::ut_s04_09_owner_cannot_leave`
- **断言**：409；`code == OWNER_CANNOT_LEAVE`

### UT-S04-10 — Owner 归档 room 204

- **位置**：`rooms_v1::tests::ut_s04_10_archive_room_success`
- **断言**：204；GET room → 404 ROOM_NOT_FOUND

### UT-S04-11 — 恢复已归档 room 200

- **位置**：`rooms_v1::tests::ut_s04_11_restore_room_success`
- **步骤**：register+login → create diagram → create room → `DELETE /rooms/{id}`（204 归档）→ `POST /rooms/{id}/restore`
- **断言**：200；`GET /rooms`（active 列表）重新包含该房间；`GET /rooms?archived=true` 不再包含；`GET /rooms/{id}` 200 且可正常读取

### UT-S04-12 — 恢复冲突 409

- **位置**：`rooms_v1::tests::ut_s04_12_restore_room_conflict`
- **步骤**：room A（diagram d）归档 → 对同一 diagram d 创建 room B（active）→ `POST /rooms/A/restore`
- **断言**：409 `ROOM_DIAGRAM_TAKEN`，响应含 `existingRoomId=B`；A 保持归档（`GET /rooms?archived=true` 仍含 A）

### UT-S04-13 — 彻底删除硬删级联

- **位置**：`rooms_v1::tests::ut_s04_13_permanent_delete_room`
- **步骤**：create room → invite（产生 room_invite 行）→ 归档 → `DELETE /rooms/{id}/permanent`
- **断言**：204；随后 `POST /rooms/{id}/restore` → 404；`GET /rooms?archived=true` 不含该房间；room 行物理不存在（库内直查）；diagram 仍存在可 GET

### UT-S04-14 — 回收站权限与状态门控

- **位置**：`rooms_v1::tests::ut_s04_14_recycle_bin_guards`
- **断言**：
  1. 非 owner（普通成员/非成员）调 restore / permanent → 403 `FORBIDDEN`
  2. active 房间直接调 permanent → 404 `ROOM_NOT_FOUND`（防绕过回收站）
  3. `GET /rooms?archived=true` 仅返回本人为 owner 的归档房间（他人 owner 的归档房间不出现）
  4. 未带 token → 401

### UT-S04-16 — invite_url 使用 PUBLIC_BASE_URL 生成（含端口 / 正确 host）

- **位置**：`rooms_v1::tests::ut_s04_16_invite_url_from_public_base_url`
- **步骤**：设置 `PUBLIC_BASE_URL=http://192.168.1.10:3000` → owner 注册登录 → 创建 diagram → 创建 room → `POST /rooms/{id}/invites`
- **断言**：201；`inviteUrl == "http://192.168.1.10:3000/invite/{token}"`（含端口、host 与配置一致）；`inviteUrl` 不含 `localhost` / `127.0.0.1`

### UT-S04-17 — 无 PUBLIC_BASE_URL 时回退 Host header 推导

- **位置**：`rooms_v1::tests::ut_s04_17_invite_url_fallback_host_header`
- **步骤**：不设置 `PUBLIC_BASE_URL` → 携带 `Host: 192.168.1.10:3000`（或含 scheme 推导头的等价反代场景）请求创建 invite
- **断言**：201；`inviteUrl` 匹配 `^{scheme}://192.168.1.10:3000/invite/{token}$`（保留非默认端口）；不含硬编码 `localhost` / `127.0.0.1` / 无端口 host

## 3. ST 用例

### ST-S04-01 — 完整 room 生命周期

- **位置**：`rooms_v1::tests::st_s04_01_room_lifecycle_flow`
- **步骤**：register×2 → login×2 → create diagram → create room → invite → preview → accept → members → 409 duplicate → remove → 403 guest → archive
- **断言**：与 `core-S04-room-lifecycle.json` 主链路一致

### ST-S04-02 — 邀请链接端到端可用（被邀请人视角）

- **位置**：`rooms_v1::tests::st_s04_02_invite_link_e2e_guest_perspective`
- **步骤**：owner 注册登录 → 创建 diagram → 创建 room → 创建 invite 得 `inviteUrl` → 解析 `inviteUrl` 的 `/invite/{token}` 路径（被邀请人视角，换用与 owner 请求不同的 Host 亦可）→ `GET /api/v1/rooms/invites/{token}` preview → guest 注册登录 → `POST /api/v1/rooms/invites/{token}/accept`
- **断言**：
  1. `inviteUrl` 匹配 `^{scheme}://{host}/invite/{token}$`，host/port 与部署对外地址一致；不含 `localhost` / `127.0.0.1`
  2. preview 200，返回 `roomName` / `diagramId` / `role=editor`
  3. accept 200；guest `GET /rooms/{roomId}` 的 `myRole == "editor"`、`memberCount == 2`

## UI / 页面流用例

| ID | 前置 | 操作 | 预期 | 状态 |
|---|---|---|---|---|
| ST-S04-UI-01 | 已登录 | 打开 `/rooms` | `room-list` 或空状态；`btn-create-room`；用户菜单 | 本提案 B 批实现 |
| ST-S04-UI-02 | 已登录 | 创建房间 | `POST /rooms`；进入 room-editor；`room-badge` 显示房间名 | 本提案 B 批实现 |
| ST-S04-UI-03 | Owner | 生成邀请 | 显示 invite URL；preview/accept 链路可用 | 本提案 B 批实现 |
| ST-S04-UI-04 | 另一用户 | 接受邀请 | 加入后进入同一 room-editor | 本提案 B 批实现 |
| ST-S04-UI-05 | Owner | 成员面板改角色/移除 | 列表即时更新；API PATCH/DELETE | 本提案 B 批实现 |
| ST-S04-UI-06 | Viewer | 新建表/改字段/邀请 | 写操作禁用或拦截；无写 API/WS op；只读提示 | 本提案 B 批实现 |
| ST-S04-UI-07 | 邀请过期 | 打开 invite | 失效页；无加入按钮 | 本提案 B 批实现 |
| ST-S04-UI-08 | Owner | RoomPanel 点「删除房间」→ 确认模态 → 确认 | `DELETE /rooms/{id}` 204；关闭模态回 rooms 页；列表刷新后该房间消失；403 显示「无权限删除此房间」模态不关闭 | p0-fix 定点 1 实现（e2e 链路待 wasm-pack/Playwright harness） |
| ST-S04-UI-09 | 已登录 | 创建房间 → diagram 选「新建空白模型」→ 引擎选 MySQL → 创建并进入 | `create-room-database` 仅在选「新建空白模型」时可见；`POST /diagrams` 请求体 `database='mysql'`；进入后 AppBar 引擎 = MySQL；Inspector 字段类型含 `DATETIME` 且无 `SERIAL` | fix-listview-grid-and-room-dbtype |
| ST-S04-UI-11 | 已登录 | 创建房间/新建图不触碰引擎下拉（默认 Generic） | `POST /diagrams` 不带 `database` 或为 NULL；行为与现状一致（回归不破） | fix-listview-grid-and-room-dbtype |
| ST-S04-UI-12 | 已进入 room-editor | 观察 AppBar 并点击「← 空间」 | `btn-back-to-rooms` 可见且位于 room-badge 左侧；点击返回 `/rooms` 房间列表页；长房间名时 room-badge ellipsis 缩略且 `title` 属性为全文 | fix-appbar-roomname-back-and-import-merge |
| ST-S04-UI-13 | 已登录，房间列表含 owner 房间 | 房间卡片点删除按钮 → 确认模态 → 确认 | `room-card-delete-{id}` 仅 owner 卡片渲染；点击不进入房间；`modal-delete-room-list` 含房间名；确认后 `DELETE /rooms/{id}`；卡片即时从列表移除 + Toast「已删除房间」 | fix-overflow-menu-room-delete-listview-io |
| ST-S04-UI-14 | 已登录，已删除（归档）一个 owner 房间 | `/rooms` 点 `btn-room-trash` → 回收站对该房间点恢复 | 回收站视图 `room-trash-view` 展示归档房间名与归档时间；恢复调 `POST /rooms/{id}/restore`；Toast「已恢复房间」；返回列表后房间重现 | feat-room-recycle-bin-dropdown-and-db-export |
| ST-S04-UI-15 | 回收站内有归档房间 | 点 `btn-purge-room-{id}` → `modal-purge-room` 确认 | 模态正文含「不可恢复」与「Diagram 保留」；确认调 `DELETE /rooms/{id}/permanent`；Toast「已彻底删除」；列表移除 | feat-room-recycle-bin-dropdown-and-db-export |
| UT-S04-15 | DB 含 active 与 is_deleted=1 的 diagram | `GET /diagrams/queryAll` | 200；返回列表不含已软删 diagram |
| UT-S04-UI-16 | — | `include_str!` 锚点 | `editor_panels.rs` 含 `create-room-diagram-delete` 与 `modal-delete-diagram`；删除接线 `client.delete(`；仅选中既有图（非 `__new__`）时渲染 |
| ST-S04-UI-16 | 已登录，queryAll 返回 1 张未绑定孤儿图 | 打开创建房间模态 → 选中该图 → 删除此图表 → 确认 | `modal-delete-diagram` 含图名；确认调 `DELETE /api/v1/diagrams/{id}`；选项列表刷新不再含该图；选中回退「新建空白模型」；Toast「已删除图表」 |
| UT-S04-16 | 设置 `PUBLIC_BASE_URL=http://192.168.1.10:3000` | owner 创建 room → `POST /rooms/{id}/invites` | 201；`inviteUrl == "http://192.168.1.10:3000/invite/{token}"`；不含 `localhost` / `127.0.0.1` |
| UT-S04-17 | 不设置 `PUBLIC_BASE_URL` | 携带 `Host: 192.168.1.10:3000`（或 X-Forwarded-Proto 反代场景）创建 invite | 201；`inviteUrl` 由 Host + scheme 推导（非默认端口保留）；不含 `localhost` / `127.0.0.1` |

### UT-S04-UI-12 — 名称缩略锚点测试

- **位置**：`frontend-rs/src/editor_panels.rs`（AppBar 组件）
- **断言**（`include_str!` 锚点口径）：
  1. 存在 `data-testid="btn-back-to-rooms"`
  2. room-badge 渲染含 `title=` 属性绑定（悬停全文）
  3. styles.css 含 room-badge 名称缩略规则（`text-overflow:ellipsis`）

### UT-S04-UI-13 — 房间卡片删除锚点测试

- **位置**：`frontend-rs/src/editor_panels.rs`（RoomsListPage 组件）
- **断言**（`include_str!` 锚点口径）：
  1. 存在 `room-card-delete-` testid 前缀（删除按钮）
  2. 存在 `modal-delete-room-list`（列表页删除确认模态）
  3. 删除按钮渲染受 `can_delete_room` / `my_role == "owner"` 门控

### UT-S04-UI-15 — 回收站锚点测试

- **位置**：`frontend-rs/src/editor_panels.rs`（RoomsListPage 组件）
- **断言**（`include_str!` 锚点口径）：
  1. 存在 `data-testid="btn-room-trash"` 与 `room-trash-view`、`btn-trash-back`
  2. 存在 `btn-restore-room-` / `btn-purge-room-` testid 前缀与 `modal-purge-room` / `btn-confirm-purge-room`
  3. RoomClient 含 `restore_room` 与 `permanently_delete_room`（或等价命名）方法，分别指向 `/restore` 与 `/permanent`
  4. DeleteRoomModal 文案不再含「删除后不可恢复」，改为含「回收站」语义

> ~~ST-S04-UI-10（新建图模态引擎选 PostgreSQL）~~ 已删除（fix-listview-grid-and-room-dbtype 实现期修正）：新建图模态唯一触发点 TopBar/TopMenuBar 为死代码，生产端无独立新建图入口；引擎前置选择由「创建房间 → 新建空白模型」唯一活路径覆盖。

## 既有 S04 用例补充约束

后端编排保持；前端验收必须使用上表 UI 用例，不得仅以 API 200 视为「已对齐主原型」。本提案 B 批负责落实上表自动化。

### 变更记录

| TC ID | 变更 | 说明 |
|-------|------|------|
| UT-S04-11~14 | ADDED（feat-room-recycle-bin-dropdown-and-db-export） | 回收站后端：恢复 / 恢复冲突 409 / 硬删级联 / 权限与状态门控 |
| UT-S04-UI-15 | ADDED（feat-room-recycle-bin-dropdown-and-db-export） | 回收站前端锚点 + 删除文案口径修正 |
| ST-S04-UI-14/15 | ADDED（feat-room-recycle-bin-dropdown-and-db-export） | 回收站恢复 / 彻底删除 e2e |
| UT-S04-15 / UT-S04-UI-16 / ST-S04-UI-16 | ADDED（fix-select-bg-diagram-delete-and-log-format） | queryAll 过滤软删 + 创建房间模态删除孤儿图入口 |
| UT-S04-16 / UT-S04-17 | ADDED（fix-canvas-zoom-invite-comment-resize） | invite_url 生成规则：PUBLIC_BASE_URL 配置推导 + Host header 回退（禁硬编码 localhost） |
| UT-S04-UI-17 | ADDED（fix-canvas-zoom-invite-comment-resize） | 前端各 Client base_url 同源派生锚点（window.location.origin，dev 可覆盖） |
| ST-S04-02 | ADDED（fix-canvas-zoom-invite-comment-resize） | 邀请链接端到端可用：被邀请人视角 /invite/{token} → preview → accept 入房 |

### UT-S04-15 — queryAll 过滤软删 diagram

- **位置**：`diagrams::tests::ut_s04_15_query_all_excludes_deleted`
- **断言**：插入两张 diagram（一张 is_deleted=1）→ `query_all_diagrams` 返回仅含未删图

### UT-S04-UI-16 — 创建房间模态删除图入口锚点

- **位置**：`editor_panels::tests::test_create_room_diagram_delete_anchor_ut_s04_ui_16`
- **断言**（`include_str!` 口径）：
  1. `data-testid="create-room-diagram-delete"` 存在且渲染条件含 `__new__` 排除
  2. `data-testid="modal-delete-diagram"` 二次确认模态存在
  3. 删除接线 `client.delete(`（DELETE /api/v1/diagrams/{id}）

### ST-S04-UI-16 — 创建房间模态删除孤儿图 e2e

- **位置**：`frontend-rs/scripts/test-spec-parity-d.mjs`
- **前置**：mock `GET /diagrams/queryAll` 返回 1 张未绑定图「孤儿图」；mock `DELETE /api/v1/diagrams/{id}` 204
- **步骤**：`/rooms` → 创建房间 → diagram 下拉选「孤儿图」→ 点「删除此图表」→ 模态确认
- **断言**：模态含图名；发出 DELETE；选项刷新后下拉不再含「孤儿图」；选中回退 `__new__`；Toast「已删除图表」

### UT-S04-UI-17 — 前端 base_url 同源派生锚点测试

- **位置**：`frontend-rs/src/editor_panels.rs`（各 Client base_url 定义处）
- **断言**（`include_str!` 锚点口径）：
  1. `DiagramClient` / `AuthClient` / `RoomClient` / `CollabClient` 的 base_url 含 `window.location.origin`（或等价 `location().origin()`）同源派生逻辑
  2. 源码中不得将 `http://127.0.0.1:3000` 作为生产默认硬编码；仅允许经环境变量 / 构建配置在 dev 覆盖
