# core-V2 生产前端接入测试矩阵

> module: core | proposal: align-prototype-docs-implementation | type: frontend/backend alignment

## 1. 范围

本矩阵覆盖 S03～S05 从统一原型演示能力到生产前端真实 API/WS 接入的验收用例。它不替代 `core-PU-unified-prototype-test-cases.md`；PU 矩阵继续验证静态单文件原型，本矩阵验证 `frontend-rs` 与 `backend` 的真实联调行为。

所有新增自动化用例必须写入 OpenLogos reporter，路径优先使用 `logos/resources/verify/test-results.jsonl`；如子项目沿用既有 reporter helper，最终 verify 聚合必须能读取对应结果。

## 2. S03 前端鉴权用例

### 2.1 单元 / 组件辅助用例

| ID | 前置 | 操作 | 预期 |
|---|---|---|---|
| UT-FE-S03-01 | URL 解析 | 同时存在 pathname diagram 与 `?share=` | share id 优先且标记 share_mode=true |
| UT-FE-S03-02 | auth token JSON | 解析 `{accessToken, expiresIn, tokenType}` | 生成 Bearer header，不落 localStorage |
| UT-FE-S03-03 | user profile JSON | displayName 存在 | user-menu 展示 displayName；缺失时回退 email |
| UT-FE-S03-04 | 401 body | `code=token_expired` | refresh 状态机可识别 token 过期 |
| UT-FE-S03-05 | 错误 body | 后端返回错误 JSON 或非 JSON | UI 使用脱敏错误文案，不输出 token/cookie |

### 2.2 浏览器链路用例

| ID | 前置 | 操作 | 预期 |
|---|---|---|---|
| ST-FE-S03-01 | 未登录 | 打开生产前端默认入口 | 显示登录/注册入口；不请求私有 room 数据 |
| ST-FE-S03-02 | 注册页 | 输入唯一邮箱、合法密码并提交 | 调用 `/api/v1/auth/register` 成功；进入已登录状态或自动登录后的房间入口 |
| ST-FE-S03-03 | 登录页 | 使用已注册用户登录 | 调用 `/api/v1/auth/login`；后续 API 带 Bearer；AppBar 显示 `user-menu` |
| ST-FE-S03-04 | access token 过期 | 触发任一受保护 API | 仅发起一次 `/api/v1/auth/refresh`；原请求重放成功；`session-indicator` 更新 |
| ST-FE-S03-05 | refresh 失效 | 触发受保护 API | 清空会话并跳回登录；显示“登录已过期，请重新登录”；不无限重试 |

## 3. S04 前端房间用例

### 3.1 单元 / 组件辅助用例

| ID | 前置 | 操作 | 预期 |
|---|---|---|---|
| UT-FE-S04-01 | `/invite/{token}` URL | 解析路由上下文 | invite token 不被当作 diagram id |
| UT-FE-S04-02 | rooms list JSON | 解析 `{items,total}` | room id、role、memberCount 正确 |
| UT-FE-S04-03 | create room JSON | 响应缺少 detail 字段 | 使用默认值并允许后续 GET detail 补全 |
| UT-FE-S04-04 | room detail role | owner/viewer 各一例 | owner 可邀请；viewer 标记只读且不可邀请 |
| UT-FE-S04-05 | invite preview/accept JSON | 解析邀请创建、预览和接受响应 | token、role、roomId、diagramId 正确 |
| UT-FE-S04-06 | member JSON | 解析成员条目 | userId、displayName、role 正确 |

### 3.2 浏览器链路用例

| ID | 前置 | 操作 | 预期 |
|---|---|---|---|
| ST-FE-S04-01 | 已登录且有 diagram | 打开 `/rooms` | 调用 `GET /api/v1/rooms`；展示当前用户房间列表和创建入口 |
| ST-FE-S04-02 | 已登录且 diagram 未绑定 room | 创建房间 | 调用 `POST /api/v1/rooms`；进入 `/editor/{diagramId}?room={roomId}`；显示 `room-badge` |
| ST-FE-S04-03 | owner 在 room 内 | 生成 viewer/editor 邀请 | 调用 `POST /api/v1/rooms/{roomId}/invites`；显示 `/invite/{token}` 链接 |
| ST-FE-S04-04 | 另一个用户打开有效邀请 | 接受邀请 | 先 preview，再 Bearer 调用 accept；加入后进入同一 room |
| ST-FE-S04-05 | owner 打开成员面板 | 修改成员 role 并移除成员 | 调用 member PATCH/DELETE；UI 权限标签和成员列表即时更新 |
| ST-FE-S04-06 | viewer 进入 room | 尝试新增表、改字段、邀请成员 | 写操作 disabled 或被拦截；无写 API/WS op 发出；显示只读状态 |

## 4. S05 前端协作用例

| ID | 前置 | 操作 | 预期 |
|---|---|---|---|
| ST-FE-S05-01 | owner/editor 进入 room 编辑器 | 建立 WebSocket | 连接 `/ws/rooms/{roomId}?token=...`；收到 connected；显示 `ws-status` 和 `ot-rev` |
| ST-FE-S05-02 | 两个用户在线 | A 创建表 `orders` | A 收 ack；B 收 remote_op；两端 serverRev 一致；Activity 有记录 |
| ST-FE-S05-03 | 两个用户在线 | A 移动画布光标或选择对象 | B 可见远端 cursor/presence，且不遮挡本地选中态 |
| ST-FE-S05-04 | 连接中断 | 本地继续编辑后恢复连接 | 本地 op 排队；重连后 sync；队列清零且无数据丢失 |
| ST-FE-S05-05 | 重连失败 | 选择仅本地编辑 | 显示 409 风险；本地编辑可继续；不会误报 OT 已同步 |
| ST-FE-S05-06 | viewer WS 连接 | 尝试发送 op | 前端不发送 op；若后端返回 READ_ONLY，UI 显示只读提示且 head 不递增 |

## 5. 全链路回归用例

| ID | 前置 | 操作 | 预期 |
|---|---|---|---|
| ST-FE-V2-01 | 未登录 | 打开 `?share=<id>` 分享链接 | S02 匿名只读链路不被 auth guard 阻断 |
| ST-FE-V2-02 | 已登录单人编辑 | 新建表、保存、触发 409 | S01 保存状态、revision 和冲突模态保持原行为 |
| ST-FE-V2-03 | 已登录 | 导入 SQL/DBML/JSON 后保存 | IO 抽屉、bridge API 和保存链路不回退 |
| ST-FE-V2-04 | 桌面与 720px 视口 | 登录、房间、编辑器、成员面板、IO 抽屉 | 关键操作可达，无横向溢出或不可恢复遮挡 |

## 6. Reporter 约束

- 每个用例写入一行 JSONL，至少包含 `case_id`、`status`、`duration_ms`、`scenario`、`module`、`proposal`。
- 失败用例必须记录脱敏错误、HTTP 状态、WS frame type 或可见 UI 锚点；不得写入密码、access token、refresh token 或 cookie 原文。
- e2e 截图和 trace 可以写入测试产物目录，但 reporter 只记录路径和摘要。
