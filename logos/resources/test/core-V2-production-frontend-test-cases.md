# core-V2 生产前端接入测试矩阵

> module: core | proposal: align-frontend-to-prototype | type: frontend/backend alignment + frontend/prototype parity

## 1. 范围

本矩阵是 **主原型能力 → 真实 REST/WS** 的逐项对齐合同。PU 矩阵继续验静态原型；本文件验 `frontend-rs` + `backend`。

状态措辞统一：**规格合同**（上一变更已收口）/ **本提案实现**（`implement-unified-prototype-spec-parity` A～D 批落实自动化）。不得将「规格已写」标为「生产已完成」。

实现阶段须将用例结果写入 `logos/resources/verify/test-results.jsonl`（OpenLogos reporter）。

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

### 4.1 单元 / 组件辅助用例

| ID | 前置 | 操作 | 预期 |
|---|---|---|---|
| UT-FE-S05-01 | WS connected frame JSON | 解析 `connected` | serverRev、diagramId、snapshotHash、members、yourRole 正确 |
| UT-FE-S05-02 | WS ack / remote_op frame JSON | 解析 `ack` 与 `remote_op` | clientRev、serverRev、authorId、op payload 正确 |
| UT-FE-S05-03 | WS sync frame JSON | 解析 `sync` | serverRev 与 missed ops 列表正确 |
| UT-FE-S05-04 | WS error frame JSON | 解析 `READ_ONLY` | UI 状态机可识别只读错误并不递增 head |
| UT-FE-S05-05 | backend base URL | 构造 WS URL | `http://` 转 `ws://`，`https://` 转 `wss://`，路径为 `/ws/rooms/{roomId}?token=...` |
| UT-FE-S05-06 | collab REST JSON | 解析 head/ops 响应 | roomId、serverRev、checkpointRevision、op payload 正确 |

### 4.2 浏览器链路用例

| ID | 前置 | 操作 | 预期 |
|---|---|---|---|
| ST-FE-S05-01 | owner/editor 进入 room 编辑器 | 建立 WebSocket | 连接 `/ws/rooms/{roomId}?token=...`；收到 connected；显示 `ws-status` 和 `ot-rev` |
| ST-FE-S05-02 | 两个用户在线 | A 创建表 `orders` | A 收 ack；B 收 remote_op；两端 serverRev 一致；Activity 有记录 |
| ST-FE-S05-03 | 两个用户在线 | A 移动画布光标或选择对象 | B 可见远端 cursor/presence，且不遮挡本地选中态 |
| ST-FE-S05-04 | 连接中断 | 本地继续编辑后恢复连接 | 本地 op 排队；重连后 sync；队列清零且无数据丢失 |
| ST-FE-S05-05 | 重连失败 | 选择仅本地编辑 | 显示 409 风险；本地编辑可继续；不会误报 OT 已同步 |
| ST-FE-S05-06 | viewer WS 连接 | 尝试发送 op | 前端不发送 op；若后端返回 READ_ONLY，UI 显示只读提示且 head 不递增 |

### 4.3 真实 WS 传输接线用例（wire-frontend-collab-ws 增量）

> 区分说明：§4.1/§4.2（UT/ST-FE-S05-01~06）为**帧注入 / 状态机级**自动化，验证解析与 UI 状态逻辑；本子节用例针对**真实传输接线**——生产前端与真实后端建立实际 WebSocket，覆盖 wire-frontend-collab-ws 补齐的实现缺口。ST-FE-S05-07~09 使用 Playwright 双浏览器上下文 + 真实 backend，禁止 mock WS。

#### 4.3.1 单元 / 组件辅助用例

| ID | 前置 | 操作 | 预期 |
|---|---|---|---|
| UT-FE-S05-13 | 本地编辑事件 | table/field/reference/area/note 各 create/update/delete 共 15 种变更 → CollabOp 映射 | op `type`/`targetId`/`parentId`/`changes` 与 `collab.yaml` `CollabOp` 一致；未映射的变更不产生 op；字段增删改变 field_order 时随行携带 fields.reorder（fix-collab-autosave-race） |
| UT-FE-S05-14 | 构造 remote_op 帧 | 应用到 EditorStore | 画布状态按 op 变更（建表/改字段/删引用等各一例）；应用远端 op 不产生回声（不再上行 op） |
| UT-FE-S05-16 | viewer 角色连接 | 触发本地编辑 | CollabClient 不入队、不发送 op 帧；只读状态保持；仍可接收 remote_op |
| UT-FE-S05-17 | WS onclose（异常断开） | 触发重连策略 | 指数退避重试 ≤5 次；重连成功后发送 `sync {lastRev}` 并 flush 离线队列；5 次失败进入 `failed` 态且队列不丢弃 |
| UT-FE-S05-18 | room+connected | 触发 schedule_save | room 模式不产生任何全量 PUT（无 debounce 调度）；非 room 路径行为不变（fix-collab-autosave-race） |
| UT-FE-S05-19 | 本地编辑 dirty=true | 全部 op ack 且离线队列空 | dirty=false（ops 即保存，dirty 由 op 对账驱动）；存在 pending/queued 时保持 true（fix-collab-autosave-race） |
| UT-FE-S05-20 | 任意 server_rev | plan_remote_op / drain_buffered_ops | rev==cur+1→Apply；rev<=cur→SkipStale；rev>cur+1→BufferAndSync；sync 在途一律 Buffer；drain 对乱序/重复缓冲有序去重回放，缺口保留剩余并再次 sync（fix-collab-autosave-race 入房补齐） |
| UT-FE-S05-21 | 表含字段 [f1,f2] | 交换字段顺序后 diff；构造 fields.reorder remote_op 应用到 store | diff 产出 `fields.reorder`（targetId=表 id，changes.fieldIds=全量目标序），排在 field create/delete 之后；整表 create/delete 当次不重复产出 reorder；apply_op_to_store 按 fieldIds 重排且不产生回声（fix-collab-autosave-race，ST-SP-LIST-02 暴露的 op 模型缺口） |

> REMOVED（fix-collab-autosave-race）：UT-FE-S05-15（checkpoint 头构造）——checkpoint 协议废弃，
> `checkpoint_headers` / `save_room_checkpoint` 随代码移除；原断言由 UT-C-08（服务端 409 写收编）取代。

#### 4.3.2 浏览器链路用例（双上下文真实 WS）

| ID | 前置 | 操作 | 预期 |
|---|---|---|---|
| ST-FE-S05-07 | A(owner)/B(editor) 两个浏览器上下文进入同一 room | A 创建表 `orders` | B 端 500ms 内画布出现 `orders`；A 收 ack、B 收 remote_op；两端 `ot-rev` 一致；Activity 有记录；**无 S01 409 模态** |
| ST-FE-S05-08 | 双上下文在线 | B 上下文 `set_offline(true)` → B 本地编辑 → `set_offline(false)` | B 断线期间 `reconnect-banner` 可见且队列计数可见；恢复后自动 sync + flush；A 端最终可见 B 的编辑；队列清零 |
| ST-FE-S05-09 | 双上下文在线 | A 移动光标 / 选中表 | B 端画布出现 A 的远端光标/选中标识（`remote-cursor`）；B 自身光标不出现在 B 的远端层 |
| ST-FE-S05-10 | 双上下文在线 + owner 第三上下文 C | A/B 近乎同时各建一表 → C 进房 → A 刷新重进 | 双端收敛互见；C 与刷新后的 A 恒看到全部修改（物化读与 op log 一致）；全程零全量 PUT `/api/v1/diagrams/*`（网络抓包断言）（fix-collab-autosave-race） |
| ST-FE-S05-11 | 单上下文在房 + mock WS 已连接 | 建表后 <200ms（debounce 窗口内）注入远端 `note.create` 广播 | 远端帧到达前本地未上行编辑先被结算成 op：收敛后服务端物化文档同时含本地 `table_1` 与远端 note；旧实现 baseline 重采会吞掉 table.create（ST-CR-INSP-01 间歇失败根因回归，fix-collab-autosave-race） |

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

## 7. 生产前端原型页面流单元用例（align-frontend-to-prototype 增量）

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

## 8. 生产前端原型页面流浏览器用例（align-frontend-to-prototype 增量）

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

## 9. Reporter 约束（align-frontend-to-prototype 增量）

- `UT-FE-PROTO-*` 和 `ST-FE-PROTO-*` 每个用例写入一行 JSONL。
- 字段至少包含 `id`、`status`、`duration_ms`、`module`、`scenario`、`proposal`。
- 失败信息必须脱敏，不写 access token、refresh token、cookie、密码或邀请 token 全文。
- 对暂不具备浏览器 harness 的 ST 用例只能显式 `skip`，并写明缺口；最终验收前应优先把页面流主链改为可执行自动化。

## 原型能力 ↔ 生产 API/WS 对齐矩阵

| 原型能力 | 生产 API / WS | 验收 ID | 状态 |
|---|---|---|---|
| auth 登录/注册 | `POST /api/v1/auth/login` · `register` | ST-FE-S03-02/03 · ST-FE-PROTO-01/02 | 本提案 A 批实现 |
| session / refresh | `POST /api/v1/auth/refresh` · `GET /me` · logout | ST-FE-S03-04/05 · UT-FE-S03-* | 本提案 A 批实现 |
| rooms 列表/创建 | `GET/POST /api/v1/rooms` | ST-FE-S04-01/02 · ST-FE-PROTO-03/04 | 本提案 B 批实现 |
| invite 预览/接受 | `GET/POST .../invites*` · `/invite/{token}` | ST-FE-S04-03/04 · ST-FE-PROTO-05 | 本提案 B 批实现 |
| 成员与角色 | members PATCH/DELETE | ST-FE-S04-05/06 | 本提案 B 批实现 |
| Viewer 只读 | REST 403 + WS `READ_ONLY` | ST-FE-S04-06 · ST-FE-S05-06 | 本提案 B/C 批实现 |
| 单人保存 / SaveState | `PUT /api/v1/diagrams/{id}` | ST-FE-V2-02 · S01 用例 | 本提案 C 批实现 |
| 非 OT 409 | PUT 409 `revision_conflict` | ST-S01-02 · UT-S01-04 | 规格合同；协作模式禁 409 模态；C 批回归 |
| 分享只读 | `GET` + `?share=` | ST-FE-V2-01 · S02 | 本提案 A 批实现 |
| WS 连接态 | `/ws/rooms/{roomId}?token=` · `connected` | ST-FE-S05-01 · `ws-status` | 本提案 C 批实现 |
| OT rev / ack / remote_op | WS frames + collab REST head/ops | ST-FE-S05-02 · `ot-rev` | 本提案 C 批实现 |
| presence | presence 帧 | ST-FE-S05-03 · `room-presence` | 本提案 C 批实现 |
| 断线队列 / 重连 | sync + 本地 queue | ST-FE-S05-04 · `reconnect-banner` | 本提案 C 批实现 |
| 仅本地编辑 | 降级 PUT（可 409） | ST-FE-S05-05 | 本提案 C 批实现 |
| IO 抽屉 | bridge import/export | ST-FE-V2-03 · PC | 本提案 D 批实现 |
| 命令面板 / 代码视图 | 前端壳层 | ST-FE-PROTO-08 子集 · KB/PE | 本提案 D 批实现 |
| 主题 / 响应式 | tokens + layout | ST-FE-PROTO-07 · PE/SP | 本提案 D 批实现 |

## 既有 FE / PROTO 用例状态说明

对文档中全部 `UT-FE-*` / `ST-FE-*` / `UT-FE-PROTO-*` / `ST-FE-PROTO-*`：

- 保留用例 ID 与步骤作为**规格合同**。
- 实现与 e2e harness 由本提案 A～D 批落实；不得因 API client 已存在即勾选「生产已对齐主原型」。
- 浏览器 ST 若暂无 harness：显式 `skip` 并写明缺口，最终验收前优先打通页面流主链。

## 状态机与页面流回归（合同强化）

| ID | 前置 | 操作 | 预期 | 状态 |
|---|---|---|---|---|
| ST-FE-ALIGN-01 | 未登录 | 默认入口 | 仅 auth；不拉私有 rooms | 本提案 A 批实现 |
| ST-FE-ALIGN-02 | 登录成功 | — | 进入 `rooms-list-page`，不直达 editor | 本提案 A 批实现 |
| ST-FE-ALIGN-03 | room-editor | 观察协作锚点 | `ws-status`/`ot-rev`/`room-presence` 来自真实 WS 或明确降级 | 本提案 C 批实现 |
| ST-FE-ALIGN-04 | 协作已连接 | 并发 op | **禁止**弹出 S01 409 冲突模态 | 本提案 C 批实现 |
