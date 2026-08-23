# Delta — core-S04-room-lifecycle.md（修改）

> module: core | proposal: align-all-docs-to-unified-prototype

## ADDED — 统一原型对齐补充：文档头

> 版本：V2 | 前置：**S03** | 后续：**S05**
> Phase 2：`core-S04-room-lifecycle-design.md` | 现行原型：`core-01-editor-prototype.html`
> 历史参考：`core-04-collab-prototype.html`（非验收入口）
> 生产状态：后端已实现；生产前端 API/页面流已部分接入；逐项对齐待 `implement-unified-prototype-spec-parity`
> 跳转目标：创建 / 打开 / 接受邀请后进入 **`room-editor`**（非历史 `/editor/{id}?room=` 文案作为唯一命名）
> API/DB：预期不新增端点或表；仅补前端参与者、页面状态与权限反馈映射

## MODIFIED — 1. 场景描述

- room 创建 / 打开 / accept 后进入 **`room-editor`**（`room-editor-page`；路由可为 `/rooms/{id}/editor` 或等价）
- 被邀请人 accept 后成为 `room_member`
- viewer 进入编辑器时为只读 UI（ToolRail / 邀请写 / PUT 禁用）

## MODIFIED — 2. 参与者

| 角色 | 模块 | 说明 |
|---|---|---|
| RoomUI | `frontend-rs` | `rooms` / `invite` 页面；`rooms-list-page` / `invite-accept-page` |
| EditorUI | `frontend-rs` room-editor 壳 | AppBar `room-badge` / `btn-invite` / `room-members-panel` |
| RoomClient | `frontend-rs` | 真实 Bearer REST（既有 rooms 11 端点） |
| 主原型 | HTML 本地房间数据 | 仅演示；非生产完成证据 |

## ADDED — 统一原型对齐补充：时序跳转文案

凡「navigate editor …」「redirect editor with room」「`/editor/{diagramId}?room={roomId}`」统一语义为：

→ 进入 **`room-editor`**，并携带 room（及 diagram）上下文；AppBar 显示 `room-badge`。

对应：§3 Step 18b–19b、§5 Step 19–20、§7.1 Step 6、§13 与 S05 衔接中的编辑器 URL。

兼容：生产可用 query 或 path 实现同一页面状态，**信息架构状态 ID 以 `room-editor` 为准**。

## ADDED — 页面流（技术）

```text
rooms（列表/创建）
  ├── POST /rooms 201 ──────────────→ room-editor
  ├── 打开已有房间 ─────────────────→ room-editor
  └── /invite/{token}
        ├── preview 410 ────────────→ 失效页（无加入按钮）
        └── accept 200 ─────────────→ room-editor
```

未登录访问 `/rooms` → `401` + 前端 redirect `/login?redirect=/rooms`（EX-4.1）。

## ADDED — 权限反馈映射（前端）

| 条件 | HTTP / 业务码 | 前端 |
|---|---|---|
| 非成员进编辑器 | 403 NOT_A_MEMBER | Toast + 回 rooms |
| viewer 写操作 | 403 READ_ONLY | 禁用工具 + Toast 原因 |
| diagram 已绑 room | 409 ROOM_DIAGRAM_TAKEN | 创建失败提示，可引导打开已有房间 |
| 邀请过期 | 410 INVITE_EXPIRED | 失效页，无 `btn-accept-invite` |
| Owner leave | 409 OWNER_CANNOT_LEAVE | Toast；须删 room 或转让（若规格有） |
| 角色切换 | PATCH members | **即时**更新 ToolRail / Inspector / 邀请 / StatusBar |

锚点保留：`rooms-list-page`、`btn-create-room`、`room-list`、`room-badge`、`btn-invite`、`room-presence`、`room-members-panel`、`invite-url`、`btn-accept-invite`、`invite-accept-page`。

## MODIFIED — 13. 与 S05 的衔接

- S04 完成后，用户在 **`room-editor`** 上下文建立 WS（S05）
- 离开编辑器经 `room-badge` 回 **rooms**，不断开鉴权会话（除非 logout）

## MODIFIED — 14. 对齐参考源

- 现行主原型：`core-01-editor-prototype.html`
- `core-00-information-architecture.md` — rooms / invite / room-editor
