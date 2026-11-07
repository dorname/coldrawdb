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
| ST-S04-UI-16 | 已登录，queryAll 返回 1 张未绑定孤儿图 | 创建房间 → 搜索并选择该图 → 独立危险区删除 → 确认 | 确认模态含图名、完整 ID、表/关系数；调 DELETE；候选不再含该图；选中回退「新建空白模型」；Toast「已删除图表」 |
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
- **前置**：mock `GET /diagrams/queryAll` 返回 1 张未绑定图「孤儿图」（含完整 ID、表/关系数、更新时间）；mock `DELETE /api/v1/diagrams/{id}` 204
- **步骤**：`/rooms` → 创建房间 → 在图表选择器搜索并选择「孤儿图」→ 滚动到独立「图表管理 / 危险操作」区 → 点「删除此图表」→ 模态确认
- **断言**：删除入口不在候选列表或主提交 footer 内；确认模态含图名、完整 ID 和表/关系数；发出 DELETE；候选刷新不再含「孤儿图」；选中回退「新建空白模型」；Toast「已删除图表」

### UT-S04-UI-17 — 前端 base_url 同源派生锚点测试

- **位置**：`frontend-rs/src/editor_panels.rs`（各 Client base_url 定义处）
- **断言**（`include_str!` 锚点口径）：
  1. `DiagramClient` / `AuthClient` / `RoomClient` / `CollabClient` 的 base_url 含 `window.location.origin`（或等价 `location().origin()`）同源派生逻辑
  2. 源码中不得将 `http://127.0.0.1:3000` 作为生产默认硬编码；仅允许经环境变量 / 构建配置在 dev 覆盖

---

## 合并自 fix-remote-github-issues（2026-09-15）

## ADDED — UT-S04-18 — compose 默认 PUBLIC_BASE_URL 指向公开入口（非 :3000）

### UT-S04-18 — staging 默认基址语义

- **位置**：部署脚本 / `docker-compose.yml` 静态断言，或后端在 `PUBLIC_BASE_URL=http://localhost` 下创建 invite
- **步骤**：
  1. 读取 compose 文件：`coldrawdb.environment` 含 `PUBLIC_BASE_URL`
  2. 默认值**不得**为 `*:3000` 裸后端调试口
  3. 设置 `PUBLIC_BASE_URL=http://192.168.20.106` → `POST /rooms/{id}/invites`
- **断言**：
  - `inviteUrl` 前缀为 `http://192.168.20.106/invite/`
  - 不含 `:3000`（除非操作者显式把公开入口配成带 3000 的单入口）
  - reporter 登记 `UT-S04-18`

## MODIFIED — UT-S04-16 说明脚注

UT-S04-16 使用 `PUBLIC_BASE_URL=http://192.168.1.10:3000` 仅验证「env 优先于 Host」的**机制**；**不**表示生产推荐把邀请基址设为后端端口。staging/prod 推荐值见部署方案 §12.5（公开 SPA 入口）。

## MODIFIED — 用例 ID 清单（附录）

| ID | 标题 |
|---|---|
| UT-S04-18 | compose/公开入口 PUBLIC_BASE_URL；inviteUrl 非默认裸 :3000 |

## 合并自 fix-issue-45-room-rename（2026-09-28）

### UT-S04-19 — rename_room 后端校验与权限

- **类型**：后端 service/handler 测试（`backend` crate）
- **覆盖**：`PATCH /api/v1/rooms/{roomId}`（renameRoom）
- **断言**：
  1. owner 改名 → 200，返回 RoomDetail.name 为新名；GET 回读一致
  2. editor / viewer 改名 → 403 `FORBIDDEN`
  3. 非成员改名 → 403 `NOT_A_MEMBER`
  4. name trim 后为空 / 超过 64 字符 → 422 `VALIDATION_ERROR`
  5. 房间不存在或已归档 → 404 `ROOM_NOT_FOUND`
  6. 改名不触碰绑定 diagram 文档（diagram 标题不变）
- reporter 登记 `UT-S04-19`

### UT-S04-UI-18 — 重命名入口锚点与权限可见性

- **类型**：前端锚点测试（`frontend-rs` lib）
- **断言**：
  1. 房间卡片存在 `room-card-rename-{id}` 入口锚点，且仅 `my_role == "owner"` 渲染
  2. `modal-rename-room` / `input-rename-room` / `btn-confirm-rename-room` 锚点存在
  3. 前端预校验：trim 后空名或 >64 字符时确认按钮不放行并提示「名称需为 1–64 字符」
  4. `room-card-rename` 点击不触发卡片「进入房间」跳转（事件拦截，与 delete 按钮同口径）
- reporter 登记 `UT-S04-UI-18`

### ST-S04-UI-17 — 重命名协作空间 e2e

- **类型**：e2e（`frontend-rs/scripts/test-spec-parity-d.mjs`）
- **步骤**：
  1. owner 创建房间（默认名）进入房间列表
  2. 点击 `room-card-rename-{id}` → 模态输入新名「评审空间-新」→ 确认
  3. 断言卡片标题即时变为新名 + Toast「已重命名」
  4. 进入房间 → `room-badge` 显示新名；刷新页面后仍为新名
  5. 修改 AppBar diagram 标题 → 返回列表，房间卡片名不受影响（字段解耦）
- reporter 登记 `ST-S04-UI-17`

### 用例 ID 清单（附录）追加

| ID | 标题 |
|---|---|
| UT-S04-19 | rename_room 后端校验与权限（owner/403/422/404/字段解耦） |
| UT-S04-UI-18 | 重命名入口锚点与 owner 可见性 + 前端预校验 |
| ST-S04-UI-17 | 重命名协作空间 e2e：即时一致 + 刷新保持 + 字段解耦 |

## 合并自 fix-diagram-list-metadata（2026-10-31）

### UT-S04-20 — queryAll 返回真实图表元数据

- **位置**：`backend/src/diagrams/mod.rs::tests`，独立临时 SQLite。
- **前置**：迁移后的数据库含活跃图与软删除图；活跃图 `revision=8`、`updated_at='2026-10-29T12:34:56Z'`，`lastModified` 使用不同旧值以区分字段来源。
- **操作**：调用 `list_active_diagrams` 并序列化结果。
- **断言**：活跃图摘要 `revision=8`、`updated_at` 等于数据库真实值，原有 `lastModified` 保持兼容；软删除图仍不出现；不使用默认 0/null 覆盖真实元数据。
- **reporter**：登记 `UT-S04-20` 到 `logos/resources/verify/test-results.jsonl`。

### 用例登记追加

| ID | 前置 | 操作 | 预期 |
|---|---|---|---|
| UT-S04-20 | 活跃图含 revision=8 与真实 updated_at，另有软删除图 | 调用 queryAll 列表查询 | 返回真实 revision/updated_at 和兼容 lastModified；软删除图被过滤 |

## 合并自 feat-room-diagram-picker（2026-10-31）

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

## 合并自 fix-trash-restore-refresh（2026-11-04）

## MODIFIED — ST-S04-UI-14 — 回收站恢复（断言增强）

- **位置**：`frontend-rs/scripts/test-spec-parity-d.mjs`（既有用例增强，编号不变）。
- **MODIFIED 断言**：恢复成功后，主房间列表必须**重新拉取**并包含恢复的房间——
  installApi 启用动态主列表模式：初始 `roomsList` 不含 room-arch（真实恢复前
  状态），恢复请求处理后 `GET /api/v1/rooms` 动态包含 room-arch。断言
  「返回列表后房间重现」在静态 mock 下不再恒真，真正依赖 reload 刷新。
- **不变断言**：恢复调 `POST /rooms/{id}/restore`；Toast「已恢复房间」；条目移出回收站。
- **reporter**：真实运行后登记 `ST-S04-UI-14`（编号沿用，不新增 ID）。

### 用例登记（fix-trash-restore-refresh）

| ID | 层级 | 说明 |
|---|---|---|
| ST-S04-UI-14 | ST | 回收站恢复后主列表必须重拉重现（mock 时序化，断言真依赖刷新） |

## 合并自 fix-enter-diagram-fit-content（2026-11-06）

## ADDED — ST-S04-UI-19 — 进房加载后表在视口内可见（自动 fit）

- **位置**：`frontend-rs/scripts/test-spec-parity-d.mjs`
- **前置**：mock `GET /diagrams/{id}` 返回表坐标远离原点的图（如 (4000, 3000) 附近多表），rooms/协作 mock 同 ST-S04-UI-14 口径
- **步骤**：`/rooms` 点开该图所在房间进入 room-editor
- **断言**：进房后无需任何缩放/平移操作，首张表节点在画布视口内可见（fillText CTM 屏幕坐标落画布容器内）；缩放显示 ≤100%；空图（0 表）回归场景保持原点不报错
- **触发前提**（2026-11-07 verify 迭代修订，UT-CR-FOCUS-05）：自动 fit 仅在内容**完全出视口**时执行——部分可见不动视口（既有用例视口假设保持）；本用例 mock 远坐标图即全出视口形态
- **reporter**：登记 `ST-S04-UI-19`

## ADDED — ST-S04-UI-20 — 悬浮控件「适应画布」按钮

- **位置**：`frontend-rs/scripts/test-spec-parity-d.mjs`
- **步骤**：进入远坐标图房间（自动 fit）→ 空白处拖拽平移使画布呈空 → 点悬浮控件 `btn-fit-canvas`
- **断言**：按钮存在且文案「适应画布」（对齐主原型 fit-canvas）；点击后表重新出现在视口内（transform 数学断言 + 产帧视觉断言）；不影响保存链路（无 PUT/WS op 副作用）；按钮路径无条件适应（不受 UT-CR-FOCUS-05 触发前提约束）
- **reporter**：登记 `ST-S04-UI-20`

## 变更记录追加

| TC ID | 变更 | 说明 |
|-------|------|------|
| ST-S04-UI-19/20 | ADDED（fix-enter-diagram-fit-content） | 进房自动适应内容 + 适应画布按钮 e2e |
| ST-S04-UI-19/20 | MODIFIED（同批 2026-11-07 verify 迭代） | 断言定稿为探针数学层 + CTM 视觉层；登记自动 fit「全出视口才触发」前提（按钮无条件） |
