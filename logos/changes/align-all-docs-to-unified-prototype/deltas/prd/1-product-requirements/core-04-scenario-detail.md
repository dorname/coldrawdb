# Delta — core-04-scenario-detail.md（修改）

> module: core | proposal: align-all-docs-to-unified-prototype

## ADDED — 统一原型对齐补充：S01 触发条件与主路径入口

#### 触发条件

用户已进入协作编辑器（`room-editor`）或兼容的编辑器画布，对任意画布对象进行创建 / 修改 / 删除操作。

> 默认登录后路径为 `auth → rooms → room-editor`。历史「直达 `/editor` Landing」叙述仅作 V1 兼容说明，不再作为现行产品主路径。

#### 主路径

1. 用户在编辑器画布创建表 "users"，含字段 id / name / email
2. 添加关系 "users.id → posts.user_id"（一对多）；可用点击两点或拖字段出线（见关系规格）
3. 编辑过程中，1s debounce 后自动触发 `PUT /api/v1/diagrams/{id}`（协作房间内另受角色与 WS/OT 约束）
4. 服务端 `BEGIN IMMEDIATE TRANSACTION` → 写入领域表 → `COMMIT`
5. 响应 `200 OK` + `new_revision`；前端更新 revision 与 SaveState（Saved / Saving / 失败）

#### 验收条件补充（页面反馈）

- AppBar `data-testid="save-state"` 与 `revision-display` 必须反映保存结果
- 协作模式下并发合并不得误用 S01 的 409 冲突模态（见 S05）；仅非 OT 冲突路径保留 Reload / Force / Cancel

## MODIFIED — 异常：share 参数缺失

##### 异常：share 参数缺失

- **GIVEN** 用户访问站点根路径（无 `share` 参数）且未登录
- **WHEN** 页面加载
- **THEN**
  - 现行产品默认进入 **登录/注册（auth）**，而不是旧 Landing「New」直达空白编辑器
  - 不弹「分享链接无效」错误
  - 已登录用户默认进入 **房间列表（rooms）**

##### 正常：有效分享链接（补充只读边界）

- 分享链路保持匿名只读；不得被鉴权拦截阻断
- 只读态下写工具禁用；可查看表/关系；不要求建立协作 WS

## REMOVED — 场景范围外（V2 计划，不在本变更范围内）

删除将 S03/S04/S05 标为「V2 候选 / 不在本变更范围内」的整节。

## ADDED — S03：用户注册 / 登录 / Token 续期（V2 🟡）

#### 触发条件

未登录用户访问工作空间入口，或会话过期需要续期 / 重新登录。

#### 用户价值

建立身份后进入房间列表，支撑团队协作；失败反馈不泄露账号是否存在。

#### 优先级

P0（V2 工作空间必备）

#### 主路径

1. 用户打开 auth 视图（`login-form` / `register-form`）
2. 提交合法邮箱与密码（注册另含 display name / 确认密码）
3. 调用 `register` 或 `login` → 获得会话
4. 前端切换到 rooms 视图（不整页刷新）
5. Token 临近过期时 refresh；失败则回到登录

#### 验收条件（交互级）

##### 正常：登录成功进入 rooms

- **GIVEN** 用户在登录表单，输入合法邮箱与 ≥8 位密码
- **WHEN** 点击登录提交
- **THEN** 按钮 loading → Toast → 进入 `rooms-list-page`；主题保持

##### 异常：凭据失败

- **GIVEN** 服务端返回鉴权失败（或原型「模拟错误」）
- **WHEN** 提交
- **THEN** 显示通用「邮箱或密码错误」，不区分用户是否存在

##### 正常：退出

- **GIVEN** 用户已登录
- **WHEN** 用户菜单选择退出；若有未保存操作先确认
- **THEN** 清理会话，回到登录页

## ADDED — S04：创建 / 加入协作房间（V2 🟡）

#### 触发条件

用户已登录，位于 rooms；或持有邀请链接进入 invite 预览。

#### 用户价值

以房间为单位组织协作图；按 Owner/Editor/Viewer 控制写权限。

#### 优先级

P0

#### 主路径

1. 房间列表展示确定性房间卡片（关联 diagram、成员、角色、最近活动）
2. 创建房间：Modal 校验名称 / diagram / 默认邀请角色 → 进入 room-editor
3. 打开已有房间 → room-editor
4. 邀请：生成邀请 URL；接受有效邀请进入同一房间；过期邀请不提供加入
5. 成员 SideSheet：在线态、改角色、移除（Owner 约束）

#### 验收条件（交互级）

##### 正常：创建并进入编辑器

- **GIVEN** 用户在 `rooms-list-page`
- **WHEN** 创建合法房间并提交
- **THEN** 进入 `room-editor-page`；`room-badge` 显示房间名

##### 正常：Viewer 只读

- **GIVEN** 当前角色为 Viewer
- **WHEN** 尝试新建表 / 保存 / 邀请
- **THEN** 控件禁用且事件被阻断；给出原因 Toast；仍可见成员与只读画布

## ADDED — S05：OT 实时协作（V2 🟡）

#### 触发条件

用户进入 room-editor 且房间支持协作；建立 `/ws/rooms/{room_id}`（生产）或原型 WS 模拟器（仅演示）。

#### 用户价值

多人同时编辑可感知；断线不静默丢操作；Viewer 仍可观察。

#### 优先级

P0

#### 主路径

1. 连接成功：StatusBar `ws-status` 已连接；`ot-rev` 显示 server_rev
2. 本地可写操作 optimistic 应用并等待 ack；远端 op 更新画布 + Activity
3. presence 光标/头像可见且不遮挡本地主选中语义
4. 断线 → reconnecting Banner + 待同步队列；恢复 → sync/回放；失败 → 危险 Banner + 可选「仅本地编辑」
5. 协作合并成功时不得弹出 S01 的 409 冲突模态

#### 验收条件（交互级）

##### 正常：远端操作可见

- **GIVEN** 已连接
- **WHEN** 远端创建表或更新字段（生产 WS 或评审用模拟器）
- **THEN** 画布与 Activity 同步；revision 单调递增

##### 异常：重连失败

- **GIVEN** 连接进入 failed
- **WHEN** 用户未选择仅本地编辑
- **THEN** 写操作暂停；提供重新连接；选择仅本地后持续警告冲突风险

## ADDED — 原型 / 生产边界（适用于 S03～S05）

- 主原型内登录、HTTP、WS、OT、剪贴板与下载可为本地模拟，须标注演示/模拟。
- 规格中的 `data-testid` 是后续生产对齐锚点，**不得**仅因原型可演示就把场景标为全栈完成。
- 本提案合并后的文档状态：规格已对齐主原型；生产逐项实现对齐见变更 `implement-unified-prototype-spec-parity`。
