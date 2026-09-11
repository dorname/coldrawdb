# Delta — core-V2 生产前端继续对齐主原型测试矩阵

> module: core | proposal: align-frontend-to-prototype | type: frontend/prototype parity

## ADDED — 7. 生产前端原型页面流单元用例

| ID | 前置 | 操作 | 预期 |
|---|---|---|---|
| UT-FE-PROTO-01 | pathname/search 解析 | 输入默认入口、`?share=`、`/invite/{token}` | 返回 auth / share-editor / invite 的页面状态，不把 invite token 当 diagram id |
| UT-FE-PROTO-02 | auth 状态机 | 登录/注册成功事件 | 页面状态从 auth 进入 rooms；session notice 不包含 token 原文 |
| UT-FE-PROTO-03 | rooms DTO | 解析 rooms list 空列表、有列表、缺省字段 | 生成稳定 room card view model，空状态可见 |
| UT-FE-PROTO-04 | create room 结果 | room detail 含 diagramId/name/role | 设置 current_room、diagram_id、title，并进入 editor 页面 |
| UT-FE-PROTO-05 | collab UI 状态 | Offline/Connecting/Connected/Reconnecting/ReadOnly | `ws-status`、`ot-rev`、`reconnect-banner`、只读提示文案稳定 |
| UT-FE-PROTO-06 | 响应式布局 class | 720px 视口或 inspector/io drawer 切换 | editor、members、IO 抽屉不会同时占用不可关闭层级 |
| UT-FE-PROTO-08 | styles.css 设计 token 块 | 检查裸 `:root` 选择器上的 token 块 | 无注释截断、无选择器污染、无 node_modules glob 残留 |
| UT-FE-PROTO-09 | AuthGate 输入绑定 | 检查登录/注册表单的 prop:value + on:input | 表单输入双向绑定，否则无法提交 |

## ADDED — 8. 生产前端原型页面流浏览器用例

| ID | 前置 | 操作 | 预期 |
|---|---|---|---|
| ST-FE-PROTO-01 | 未登录 | 打开生产前端默认入口 | 显示 `auth-gate`、`login-form`、`auth-tab-register`；不显示私有 room 数据 |
| ST-FE-PROTO-02 | 登录/注册成功 | 提交合法表单 | 调用真实 auth API 后进入 `rooms-list-page`；可见 `session-indicator` 和用户菜单 |
| ST-FE-PROTO-03 | 已登录 | 打开 rooms 首屏 | 调用 `GET /api/v1/rooms`；显示 `room-list`、空状态或 room card、新建房间入口 |
| ST-FE-PROTO-04 | 已登录且 diagram 可绑定 | 创建房间并进入 | 调用 `POST /api/v1/rooms`；进入 `room-editor-page`；AppBar 显示 `room-badge` |
| ST-FE-PROTO-05 | 未登录 `/invite/{token}` | 打开有效邀请链接 | 显示 `invite-accept-page` preview；接受前提示登录；登录后可继续 accept |
| ST-FE-PROTO-06 | room editor | 加载成员与协作状态 | 可见 `ws-status`、`ot-rev`、`room-presence`、`activity-feed`；viewer 写操作被禁用 |
| ST-FE-PROTO-07 | 720px 视口 | 依次打开 auth、rooms、editor、members、IO 抽屉 | 关键按钮可达；页面无横向溢出；浮层可关闭 |
| ST-FE-PROTO-08 | 完整回归 | 跑 S01/S02/IO/命令面板/ST-PU 回归集合 | 既有 V1、V2 与主原型测试不退化，reporter 覆盖完整 |

## ADDED — 9. Reporter 约束

- `UT-FE-PROTO-*` 和 `ST-FE-PROTO-*` 每个用例写入一行 JSONL。
- 字段至少包含 `id`、`status`、`duration_ms`、`module`、`scenario`、`proposal`。
- 失败信息必须脱敏，不写 access token、refresh token、cookie、密码或邀请 token 全文。
- 对暂不具备浏览器 harness 的 ST 用例只能显式 `skip`，并写明缺口；最终验收前应优先把页面流主链改为可执行自动化。
