# core 前后端对齐验收说明

> module: core | proposal: align-prototype-docs-implementation + align-frontend-to-prototype

## 1. 验收边界

`core-01-editor-prototype.html` 是 S01～S05 的唯一现行视觉与交互基线，但它是静态单文件原型，不调用生产 API。本变更的生产验收必须运行 `frontend-rs` 与 `backend`，并验证真实 REST/WS 链路。

历史原型 `core-03-auth-prototype.html`、`core-04-collab-prototype.html`、`core-05-ot-collab-prototype.html` 仅用于差异追溯，不作为本变更验收入口。

## 2. 生产验收标准

- **FEALIGN-AC-01 鉴权闭环**：登录/注册/refresh/logout/me 全部通过真实 `/api/v1/auth/*`；access token 不写 localStorage；refresh 失败不无限重试。
- **FEALIGN-AC-02 分享兼容**：未登录用户打开 `?share=` 仍可走 S02 匿名只读加载，不被 S03 auth guard 阻断。
- **FEALIGN-AC-03 房间闭环**：`/rooms`、创建 room、生成 invite、preview、accept、成员 role 更新和移除全部调用真实 `/api/v1/rooms*`。
- **FEALIGN-AC-04 权限一致**：viewer 的 ToolRail、Inspector、邀请入口和 WS op 均保持只读；后端 READ_ONLY/FORBIDDEN 错误能映射到用户可见提示。
- **FEALIGN-AC-05 实时协作**：room 编辑器能建立 `/ws/rooms/{roomId}?token=...`，处理 connected/ack/remote_op/presence/sync/error 帧，并显示 ws-status、ot-rev、room-presence、reconnect-banner。
- **FEALIGN-AC-06 断线不丢编辑**：断线期间本地 op 进入队列，恢复后 sync 并清零；重连失败时明确降级为本地编辑并提示 409 风险。
- **FEALIGN-AC-07 V1 不回退**：S01 保存、409 冲突、S02 分享加载、IO 抽屉、命令面板、设计系统和移动端布局继续通过既有测试。
- **FEALIGN-AC-08 Reporter 完整**：每个新增 UT/ST/e2e 用例写入 OpenLogos reporter；失败信息脱敏。

## 5. 页面流对齐标准（align-frontend-to-prototype 增量）

上一轮 `align-prototype-docs-implementation` 已完成 S03/S04/S05 生产 API 接入。本轮验收新增“体验与页面流对齐”维度，避免仅凭 API client 和局部面板判断已经贴合主原型。

- **FEUX-AC-01 Auth 页面流**：未登录默认入口显示 `auth-gate`，登录/注册表单具备主原型的品牌区、双 tab、字段错误、loading 与安全会话提示；登录/注册成功后进入 `rooms-list-page`，不直接进入编辑器。
- **FEUX-AC-02 Share 兼容**：未登录访问 `?share=<id>` 仍绕过 auth 与 rooms，进入匿名只读编辑器；AppBar 与 `session-indicator` 明确显示只读分享状态。
- **FEUX-AC-03 Rooms 首屏**：已登录用户进入 `rooms-list-page`，可见房间卡片、空状态、新建房间入口、刷新入口与用户菜单；进入房间后才显示 `room-editor-page`。
- **FEUX-AC-04 Invite 独立页**：`/invite/{token}` 在未登录时也显示 `invite-accept-page` 和 preview 信息；未登录点击接受时提示登录，登录后可继续调用真实 accept 并进入同一 room。
- **FEUX-AC-05 Editor 协作可见状态**：房间编辑器必须可见呈现 `room-badge`、`ws-status`、`ot-rev`、`room-presence`、`activity-feed`、`reconnect-banner` 与 viewer 只读状态；状态来源必须是真实 REST/WS 或明确降级，不能把原型模拟动作标成生产同步。
- **FEUX-AC-06 响应式可达**：720px 视口下 auth、rooms、editor、members、IO 抽屉和 modal 的关键按钮可达，无横向溢出、互相遮挡或无法关闭的浮层。
- **FEUX-AC-07 回归边界**：S01 保存/409、S02 分享加载、IO 抽屉、命令面板、设计系统和 ST-PU 主原型回归不退化。
- **FEUX-AC-08 Reporter 完整**：新增 `UT-FE-PROTO-*` 与 `ST-FE-PROTO-*` 必须写入 OpenLogos reporter；跳过项必须说明 harness 限制，不能静默缺失。

## 6. 页面状态边界（align-frontend-to-prototype 增量）

生产前端应显式区分四类页面状态：

| 状态 | 入口 | 主要锚点 | 退出条件 |
|---|---|---|---|
| auth | 默认未登录入口 | `auth-gate`、`login-form`、`register-form` | 登录/注册成功进入 rooms |
| rooms | 已登录但未进入房间 | `rooms-list-page`、`room-list`、`btn-create-room` | 选择/创建房间进入 editor |
| invite | `/invite/{token}` | `invite-accept-page`、`btn-accept-invite` | 接受成功进入 editor；未登录则提示登录 |
| editor | 分享只读或房间编辑 | `room-editor-page`、`editor-ready`、`editor-canvas` | 返回 rooms、退出登录或路由跳转 |

`?share=` 是特例：它直接进入 editor 状态，并保持匿名只读，不要求 auth 或 rooms。

## 3. 源码实现限制

- 不新增与已合并 API/DDL 冲突的字段、端点或表。
- 不把 refresh token、cookie 或 access token 原文写入日志、reporter 或截图文件名。
- 不直连 SQLite 绕过后端 API 实现前端功能。
- 不把静态原型中的模拟状态直接标记为生产实现。
- 不在未完成批次中提前勾选 `core-implementation-checklist.md`。

## 4. verify 前检查

运行 `openlogos verify align-prototype-docs-implementation` 前，至少应完成：

- 后端 auth/rooms/collab Rust 测试回归。
- 前端 Rust 单元测试回归。
- 前端 Playwright/e2e 覆盖 S03/S04/S05 生产接入主链。
- 统一原型 ST-PU-01～ST-PU-19 回归，确认视觉交互基线未破坏。
- reporter 中新增用例 ID 与 `core-V2-production-frontend-test-cases.md` 一一对应。
