# Delta — core 前端继续对齐统一主原型验收说明

> module: core | proposal: align-frontend-to-prototype

## ADDED — 5. 生产前端页面流对齐标准

上一轮 `align-prototype-docs-implementation` 已完成 S03/S04/S05 生产 API 接入。本轮验收新增“体验与页面流对齐”维度，避免仅凭 API client 和局部面板判断已经贴合主原型。

- **FEUX-AC-01 Auth 页面流**：未登录默认入口显示 `auth-gate`，登录/注册表单具备主原型的品牌区、双 tab、字段错误、loading 与安全会话提示；登录/注册成功后进入 `rooms-list-page`，不直接进入编辑器。
- **FEUX-AC-02 Share 兼容**：未登录访问 `?share=<id>` 仍绕过 auth 与 rooms，进入匿名只读编辑器；AppBar 与 `session-indicator` 明确显示只读分享状态。
- **FEUX-AC-03 Rooms 首屏**：已登录用户进入 `rooms-list-page`，可见房间卡片、空状态、新建房间入口、刷新入口与用户菜单；进入房间后才显示 `room-editor-page`。
- **FEUX-AC-04 Invite 独立页**：`/invite/{token}` 在未登录时也显示 `invite-accept-page` 和 preview 信息；未登录点击接受时提示登录，登录后可继续调用真实 accept 并进入同一 room。
- **FEUX-AC-05 Editor 协作可见状态**：房间编辑器必须可见呈现 `room-badge`、`ws-status`、`ot-rev`、`room-presence`、`activity-feed`、`reconnect-banner` 与 viewer 只读状态；状态来源必须是真实 REST/WS 或明确降级，不能把原型模拟动作标成生产同步。
- **FEUX-AC-06 响应式可达**：720px 视口下 auth、rooms、editor、members、IO 抽屉和 modal 的关键按钮可达，无横向溢出、互相遮挡或无法关闭的浮层。
- **FEUX-AC-07 回归边界**：S01 保存/409、S02 分享加载、IO 抽屉、命令面板、设计系统和 ST-PU 主原型回归不退化。
- **FEUX-AC-08 Reporter 完整**：新增 `UT-FE-PROTO-*` 与 `ST-FE-PROTO-*` 必须写入 OpenLogos reporter；跳过项必须说明 harness 限制，不能静默缺失。

## ADDED — 6. 页面状态边界

生产前端应显式区分四类页面状态：

| 状态 | 入口 | 主要锚点 | 退出条件 |
|---|---|---|---|
| auth | 默认未登录入口 | `auth-gate`、`login-form`、`register-form` | 登录/注册成功进入 rooms |
| rooms | 已登录但未进入房间 | `rooms-list-page`、`room-list`、`btn-create-room` | 选择/创建房间进入 editor |
| invite | `/invite/{token}` | `invite-accept-page`、`btn-accept-invite` | 接受成功进入 editor；未登录则提示登录 |
| editor | 分享只读或房间编辑 | `room-editor-page`、`editor-ready`、`editor-canvas` | 返回 rooms、退出登录或路由跳转 |

`?share=` 是特例：它直接进入 editor 状态，并保持匿名只读，不要求 auth 或 rooms。
