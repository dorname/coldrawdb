# Delta: core-S04-test-cases.md

> 提案：fix-canvas-zoom-invite-comment-resize（问题 2：协作邀请链接无效）

## ADDED — UT-S04-16 — invite_url 使用 PUBLIC_BASE_URL 生成（含端口 / 正确 host）

- **位置**：`rooms_v1::tests::ut_s04_16_invite_url_from_public_base_url`
- **步骤**：设置 `PUBLIC_BASE_URL=http://192.168.1.10:3000` → owner 注册登录 → 创建 diagram → 创建 room → `POST /rooms/{id}/invites`
- **断言**：201；`inviteUrl == "http://192.168.1.10:3000/invite/{token}"`（含端口、host 与配置一致）；`inviteUrl` 不含 `localhost` / `127.0.0.1`

## ADDED — UT-S04-17 — 无 PUBLIC_BASE_URL 时回退 Host header 推导

- **位置**：`rooms_v1::tests::ut_s04_17_invite_url_fallback_host_header`
- **步骤**：不设置 `PUBLIC_BASE_URL` → 携带 `Host: 192.168.1.10:3000`（或含 scheme 推导头的等价反代场景）请求创建 invite
- **断言**：201；`inviteUrl` 匹配 `^{scheme}://192.168.1.10:3000/invite/{token}$`（保留非默认端口）；不含硬编码 `localhost` / `127.0.0.1` / 无端口 host

## ADDED — UT-S04-UI-17 — 前端 base_url 同源派生锚点测试

- **位置**：`frontend-rs/src/editor_panels.rs`（各 Client base_url 定义处）
- **断言**（`include_str!` 锚点口径）：
  1. `DiagramClient` / `AuthClient` / `RoomClient` / `CollabClient` 的 base_url 含 `window.location.origin`（或等价 `location().origin()`）同源派生逻辑
  2. 源码中不得将 `http://127.0.0.1:3000` 作为生产默认硬编码；仅允许经环境变量 / 构建配置在 dev 覆盖

## ADDED — ST-S04-02 — 邀请链接端到端可用（被邀请人视角）

- **位置**：`rooms_v1::tests::st_s04_02_invite_link_e2e_guest_perspective`
- **步骤**：owner 注册登录 → 创建 diagram → 创建 room → 创建 invite 得 `inviteUrl` → 解析 `inviteUrl` 的 `/invite/{token}` 路径（被邀请人视角，换用与 owner 请求不同的 Host 亦可）→ `GET /api/v1/rooms/invites/{token}` preview → guest 注册登录 → `POST /api/v1/rooms/invites/{token}/accept`
- **断言**：
  1. `inviteUrl` 匹配 `^{scheme}://{host}/invite/{token}$`，host/port 与部署对外地址一致；不含 `localhost` / `127.0.0.1`
  2. preview 200，返回 `roomName` / `diagramId` / `role=editor`
  3. accept 200；guest `GET /rooms/{roomId}` 的 `myRole == "editor"`、`memberCount == 2`

## MODIFIED — ### 变更记录

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
