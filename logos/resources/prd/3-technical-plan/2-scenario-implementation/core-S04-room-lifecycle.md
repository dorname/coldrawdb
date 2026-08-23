# S04 时序图：创建 / 加入协作房间（How 层 — 第 2 步：场景）

> 版本：V2 | 优先级：P2 | 前置：**S03 鉴权** | 后续：**S05 OT 协作**
> Phase 2 输入：`core-S04-room-lifecycle-design.md`
> Phase 1 输入：`core-00-scenario-overview.md` §S04

## 0. 现行文档与原型基线

> 版本：V2 | 前置：**S03** | 后续：**S05**
> Phase 2：`core-S04-room-lifecycle-design.md` | 现行原型：`core-01-editor-prototype.html`
> 历史参考：`core-04-collab-prototype.html`（非验收入口）
> 生产状态：后端已实现；生产前端 API/页面流已部分接入；逐项对齐待 `implement-unified-prototype-spec-parity`
> 跳转目标：创建 / 打开 / 接受邀请后进入 **`room-editor`**（非历史 `/editor/{id}?room=` 文案作为唯一命名）
> API/DB：预期不新增端点或表；仅补前端参与者、页面状态与权限反馈映射

## 1. 场景描述

- room 创建 / 打开 / accept 后进入 **`room-editor`**（`room-editor-page`；路由可为 `/rooms/{id}/editor` 或等价）
- 被邀请人 accept 后成为 `room_member`
- viewer 进入编辑器时为只读 UI（ToolRail / 邀请写 / PUT 禁用）

## 2. 参与者

| 角色 | 模块 | 说明 |
|---|---|---|
| RoomUI | `frontend-rs` | `rooms` / `invite` 页面；`rooms-list-page` / `invite-accept-page` |
| EditorUI | `frontend-rs` room-editor 壳 | AppBar `room-badge` / `btn-invite` / `room-members-panel` |
| RoomClient | `frontend-rs` | 真实 Bearer REST（既有 rooms 11 端点） |
| 主原型 | HTML 本地房间数据 | 仅演示；非生产完成证据 |

## 3. 时序图 — S04.1 创建协作房间

```mermaid
sequenceDiagram
    participant U as User/Browser
    participant UI as RoomUI
    participant RC as RoomClient
    participant HTTP as Browser Fetch
    participant AM as AuthMW
    participant API as RoomsAPI
    participant SVC as RoomSvc
    participant RR as RoomRepo
    participant DR as DiagramRepo
    participant DB as SQLite

    U->>UI: Step 1: 填写 create room 表单 name diagram_id
    UI->>RC: Step 2: createRoom(name, diagram_id)
    RC->>HTTP: Step 3: POST /api/v1/rooms Authorization Bearer JWT
    HTTP->>AM: Step 4: validate JWT extract user_id
    AM->>API: Step 5: authorized user_id
    API->>SVC: Step 6: create_room(owner_id, name, diagram_id)
    SVC->>DR: Step 7: find_diagram(diagram_id)
    DR->>DB: Step 8: SELECT diagram
    DB-->>DR: Step 9: diagram row
    SVC->>RR: Step 10: find_active_by_diagram(diagram_id)
    RR->>DB: Step 11: SELECT room WHERE diagram_id AND archived_at IS NULL
    alt diagram 已绑定 room
        SVC-->>API: Step 12a: DiagramAlreadyInRoom
        API-->>HTTP: Step 13a: 409 ROOM_DIAGRAM_TAKEN
    else 可创建
        SVC->>RR: Step 12b: insert room + owner member
        RR->>DB: Step 13b: INSERT room INSERT room_member role owner
        DB-->>RR: Step 14b: room_id
        SVC-->>API: Step 15b: RoomCreated
        API-->>HTTP: Step 16b: 201 room object
        HTTP-->>RC: Step 17b: parsed
        RC-->>UI: Step 18b: navigate editor diagramId room query
        UI-->>U: Step 19b: AppBar room-badge visible
    end
```

## 4. 时序图 — S04.2 邀请成员

```mermaid
sequenceDiagram
    participant U as User/Browser
    participant EI as EditorUI
    participant RC as RoomClient
    participant HTTP as Browser Fetch
    participant API as RoomsAPI
    participant SVC as RoomSvc
    participant RR as RoomRepo
    participant DB as SQLite

    U->>EI: Step 1: 点击 btn-invite 选择 role editor
    EI->>RC: Step 2: createInvite(room_id, role)
    RC->>HTTP: Step 3: POST /api/v1/rooms/{roomId}/invites Body role
    HTTP->>API: Step 4: route after AuthMW + member check owner or editor
    API->>SVC: Step 5: create_invite(inviter_id, room_id, role)
    SVC->>RR: Step 6: assert_member_can_invite
    SVC->>RR: Step 7: insert room_invite token expires_at plus 7d
    RR->>DB: Step 8: INSERT room_invite
    DB-->>RR: Step 9: invite_id token
    SVC-->>API: Step 10: InviteCreated
    API-->>HTTP: Step 11: 201 inviteUrl token expiresAt
    HTTP-->>RC: Step 12: parsed
    RC-->>EI: Step 13: show modal invite-url
    EI-->>U: Step 14: 用户复制链接
```

## 5. 时序图 — S04.3 接受邀请

```mermaid
sequenceDiagram
    participant U as User/Browser
    participant UI as RoomUI
    participant RC as RoomClient
    participant HTTP as Browser Fetch
    participant API as RoomsAPI
    participant SVC as RoomSvc
    participant RR as RoomRepo
    participant DB as SQLite

    U->>UI: Step 1: 打开 /invite/{token} 未登录则 redirect login
    UI->>RC: Step 2: previewInvite(token)
    RC->>HTTP: Step 3: GET /api/v1/rooms/invites/{token}
    HTTP->>API: Step 4: public preview no auth
    API->>SVC: Step 5: preview(token)
    SVC->>RR: Step 6: find_invite_by_token
    alt token 过期
        SVC-->>API: Step 7a: InviteExpired
        API-->>HTTP: Step 8a: 410 INVITE_EXPIRED
    else 有效
        SVC-->>API: Step 7b: InvitePreview
        API-->>HTTP: Step 8b: 200 roomName diagramTitle role
        HTTP-->>UI: Step 9b: render accept page
        U->>UI: Step 10: 点击 btn-accept-invite
        UI->>RC: Step 11: acceptInvite(token)
        RC->>HTTP: Step 12: POST /api/v1/rooms/invites/{token}/accept Bearer JWT
        HTTP->>API: Step 13: accept handler
        API->>SVC: Step 14: accept(user_id, token)
        SVC->>RR: Step 15: upsert room_member or 403 if already member redirect
        RR->>DB: Step 16: INSERT OR IGNORE room_member
        SVC-->>API: Step 17: MemberJoined
        API-->>HTTP: Step 18: 200 roomId diagramId role
        HTTP-->>UI: Step 19: redirect editor with room
        UI-->>U: Step 20: 进入 room 编辑器
    end
```

## 6. 时序图 — S04.4 成员管理与离开

```mermaid
sequenceDiagram
    participant U as User/Browser
    participant EI as EditorUI
    participant RC as RoomClient
    participant HTTP as Browser Fetch
    participant API as RoomsAPI
    participant SVC as RoomSvc
    participant RR as RoomRepo
    participant DB as SQLite

    U->>EI: Step 1: Owner 在 members panel 移除成员 B
    EI->>RC: Step 2: removeMember(room_id, user_id B)
    RC->>HTTP: Step 3: DELETE /api/v1/rooms/{roomId}/members/{userId}
    HTTP->>API: Step 4: owner only
    API->>SVC: Step 5: remove_member(actor, target)
    SVC->>RR: Step 6: delete room_member row
    RR->>DB: Step 7: DELETE FROM room_member
    SVC-->>API: Step 8: ok
    API-->>HTTP: Step 9: 204
    Note over U,DB: 成员 B 再次访问 room editor 得 403 NOT_A_MEMBER

    U->>EI: Step 10: 成员 C 点击离开房间
    EI->>RC: Step 11: leaveRoom(room_id)
    RC->>HTTP: Step 12: DELETE /api/v1/rooms/{roomId}/members/me
    HTTP->>API: Step 13: self leave owner forbidden see EX-12.1
    API->>SVC: Step 14: leave(user_id)
    SVC->>RR: Step 15: delete own membership
    API-->>HTTP: Step 16: 204 redirect /rooms
```

## 7. 步骤说明

### 7.1 创建 room（§3）

1. **User** 在 `[data-testid="modal-create-room"]` 提交 `name` + `diagram_id`。
2. **RoomClient** 携带 S03 JWT 调用 `POST /api/v1/rooms`。
3. **AuthMW** 解析 `sub` 为 `owner_id`。
4. **RoomSvc** 校验 diagram 存在且未绑定 active room → 见 EX-11.1。
5. **RoomRepo** 事务内 INSERT `room` + INSERT `room_member(role=owner)`。
6. 响应 `201` 后前端跳转 `/editor/{diagramId}?room={roomId}`。

### 7.2 邀请（§4）

1. 调用方须为 room **owner 或 editor**（可配置仅 owner）。
2. 生成 `token`（32 byte url-safe），INSERT `room_invite` TTL 7 天。
3. 响应含 `inviteUrl`：`https://host/invite/{token}`（前端路由，API 路径不同）。

### 7.3 接受（§5）

1. **GET preview** 可匿名；**POST accept** 须 JWT。
2. 已是成员 → `200` 直接返回 room 信息（幂等）或 `403` + 前端 redirect（见 EX-14.1）。

### 7.4 权限与 S01 PUT

- room 内 **editor/owner** 可对 diagram PUT（S01）；**viewer** → `403 READ_ONLY`。
- S05 引入 OT 后 PUT 频率下降，权限模型不变。

## 8. 异常用例

### EX-11.1: diagram 已绑定 room（← Phase 2 S04.1）

- **触发条件**：§3 Step 11 已存在 active room
- **期望响应**：`409 { code: "ROOM_DIAGRAM_TAKEN", existingRoomId }`
- **副作用**：不创建新 room

### EX-4.1: 未登录访问 /rooms

- **触发条件**：无 Bearer JWT
- **期望响应**：`401` + 前端 redirect `/login?redirect=/rooms`

### EX-8.1: 邀请过期

- **触发条件**：§5 preview/accept 时 `expires_at < now`
- **期望响应**：`410 { code: "INVITE_EXPIRED" }`

### EX-13.1: 非成员访问 room 编辑器 API

- **触发条件**：JWT 有效但非 room_member
- **期望响应**：`403 { code: "NOT_A_MEMBER" }`

### EX-12.1: Owner 不能 leave 只能删 room

- **触发条件**：§6 Step 13 owner 调用 `DELETE .../members/me`
- **期望响应**：`409 { code: "OWNER_CANNOT_LEAVE" }`

### EX-7.1: viewer 尝试 PUT diagram

- **触发条件**：viewer 触发 S01 save
- **期望响应**：`403 { code: "READ_ONLY" }`

## 9. API 端点摘要（由时序图推导）

| 方法 | 路径 | 子场景 | 认证 |
|---|---|---|---|
| POST | `/api/v1/rooms` | S04.1 | Bearer |
| GET | `/api/v1/rooms` | 列表 | Bearer |
| GET | `/api/v1/rooms/{id}` | 详情 | Bearer + member |
| DELETE | `/api/v1/rooms/{id}` | 删 room | Bearer + owner |
| POST | `/api/v1/rooms/{id}/invites` | S04.2 | Bearer + invite权限 |
| GET | `/api/v1/rooms/invites/{token}` | S04.3 preview | 无 |
| POST | `/api/v1/rooms/invites/{token}/accept` | S04.3 | Bearer |
| GET | `/api/v1/rooms/{id}/members` | 成员列表 | Bearer + member |
| PATCH | `/api/v1/rooms/{id}/members/{userId}` | 改 role | Bearer + owner |
| DELETE | `/api/v1/rooms/{id}/members/{userId}` | 移除 | Bearer + owner |
| DELETE | `/api/v1/rooms/{id}/members/me` | 离开 | Bearer + non-owner |

## 10. 数据表与 API 规格（V2 增量）

- DDL：`logos/resources/database/coldrawdb-v2-rooms.sql`
- OpenAPI：`logos/resources/api/rooms.yaml`（11 端点，与 §9 摘要一一对应）

## 11. 测试用例映射（规划）

| TC ID | 场景 | 对应 |
|---|---|---|
| UT-R-01 | create room 201 | §3 Step 16b |
| UT-R-02 | diagram 已占用 409 | EX-11.1 |
| UT-R-03 | create invite 201 | §4 Step 11 |
| UT-R-04 | accept invite 200 | §5 Step 18 |
| UT-R-05 | invite expired 410 | EX-8.1 |
| UT-R-06 | viewer PUT 403 | EX-7.1 |
| ST-R-01 | 创建→邀请→B accept→同 room 编辑 | S04 E2E |

## 房间生命周期跳转语义

凡「navigate editor …」「redirect editor with room」「`/editor/{diagramId}?room={roomId}`」统一语义为：

→ 进入 **`room-editor`**，并携带 room（及 diagram）上下文；AppBar 显示 `room-badge`。

对应：§3 Step 18b–19b、§5 Step 19–20、§7.1 Step 6、§13 与 S05 衔接中的编辑器 URL。

兼容：生产可用 query 或 path 实现同一页面状态，**信息架构状态 ID 以 `room-editor` 为准**。

## 页面流（技术）

```text
rooms（列表/创建）
  ├── POST /rooms 201 ──────────────→ room-editor
  ├── 打开已有房间 ─────────────────→ room-editor
  └── /invite/{token}
        ├── preview 410 ────────────→ 失效页（无加入按钮）
        └── accept 200 ─────────────→ room-editor
```

未登录访问 `/rooms` → `401` + 前端 redirect `/login?redirect=/rooms`（EX-4.1）。

## 权限反馈映射（前端）

| 条件 | HTTP / 业务码 | 前端 |
|---|---|---|
| 非成员进编辑器 | 403 NOT_A_MEMBER | Toast + 回 rooms |
| viewer 写操作 | 403 READ_ONLY | 禁用工具 + Toast 原因 |
| diagram 已绑 room | 409 ROOM_DIAGRAM_TAKEN | 创建失败提示，可引导打开已有房间 |
| 邀请过期 | 410 INVITE_EXPIRED | 失效页，无 `btn-accept-invite` |
| Owner leave | 409 OWNER_CANNOT_LEAVE | Toast；须删 room 或转让（若规格有） |
| 角色切换 | PATCH members | **即时**更新 ToolRail / Inspector / 邀请 / StatusBar |

锚点保留：`rooms-list-page`、`btn-create-room`、`room-list`、`room-badge`、`btn-invite`、`room-presence`、`room-members-panel`、`invite-url`、`btn-accept-invite`、`invite-accept-page`。

## 12. V2 边界

- ✅ 依赖 S03 JWT；不重复实现登录
- ❌ WebSocket / OT（S05）
- ❌ 实时 presence（S05）
- ✅ S02 `?share=` 与 room 独立

## 13. 与 S05 的衔接

- S04 完成后，用户在 **`room-editor`** 上下文建立 WS（S05）
- 离开编辑器经 `room-badge` 回 **rooms**，不断开鉴权会话（除非 logout）

## 14. 对齐参考源

- 现行主原型：`core-01-editor-prototype.html`
- `core-00-information-architecture.md` — rooms / invite / room-editor
