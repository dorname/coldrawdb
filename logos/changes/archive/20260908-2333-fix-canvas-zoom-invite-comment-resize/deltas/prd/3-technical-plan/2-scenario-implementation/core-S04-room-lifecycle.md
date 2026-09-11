# Delta: core-S04-room-lifecycle.md

> 提案：fix-canvas-zoom-invite-comment-resize（问题 2：协作邀请链接无效）

## MODIFIED — ## 4. 时序图 — S04.2 邀请成员

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
    Note over API,SVC: public_base_url = PUBLIC_BASE_URL 配置；缺省由请求 Host header + scheme 推导（含非默认端口）；禁止硬编码 localhost
    API->>SVC: Step 5: create_invite(inviter_id, room_id, role, public_base_url)
    SVC->>RR: Step 6: assert_member_can_invite
    SVC->>RR: Step 7: insert room_invite token expires_at plus 7d
    RR->>DB: Step 8: INSERT room_invite
    DB-->>RR: Step 9: invite_id token
    SVC-->>API: Step 10: InviteCreated(token)
    API-->>HTTP: Step 11: 201 inviteUrl = {public_base_url}/invite/{token} expiresAt
    HTTP-->>RC: Step 12: parsed
    RC-->>EI: Step 13: show modal invite-url
    EI-->>U: Step 14: 用户复制链接
```

## MODIFIED — ### 7.2 邀请（§4）

### 7.2 邀请（§4）

1. 调用方须为 room **owner 或 editor**（可配置仅 owner）。
2. 生成 `token`（32 byte url-safe），INSERT `room_invite` TTL 7 天。
3. 响应含 `inviteUrl`：`{public_base_url}/invite/{token}`（前端路由，API 路径不同）。`public_base_url` 推导规则：取 `PUBLIC_BASE_URL` 配置（环境变量）；未配置时由请求 **Host header + scheme** 推导（`Forwarded` / `X-Forwarded-Proto` 优先，保留非默认端口）；**禁止硬编码 `localhost` / `127.0.0.1` / 无端口占位 host**。
4. 前端 `DiagramClient` / `AuthClient` / `RoomClient` / `CollabClient` 的 base_url 一律由 `window.location.origin` 同源派生（单入口部署下前后端同源，无需额外配置）；仅本地 dev 双进程（trunk serve :8080 + cargo run :3000）允许经环境变量/构建配置覆盖为 `http://127.0.0.1:3000`。
