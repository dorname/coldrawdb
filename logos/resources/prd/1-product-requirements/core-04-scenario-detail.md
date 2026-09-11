### S01：编辑并保存图表（V1 ✅ P0）

#### 触发条件

用户在 `/editor` 页面进行任意画布对象的创建 / 修改 / 删除操作。

#### 用户价值

实时将脑中的数据库 schema 落到画布，并自动持久化到服务端，永不丢失。关联痛点 **P02**（持久化）、**P03**（冲突感知）。

#### 优先级

P0（V1 launch 必备）

#### 主路径

1. 用户在编辑器画布创建表 "users"，含字段 id / name / email
2. 添加关系 "users.id → posts.user_id"（一对多）
3. 编辑过程中，1s debounce 后自动触发 `PUT /api/v1/diagrams/{id}`
4. 服务端 `BEGIN IMMEDIATE TRANSACTION` → 写入 11 张表 → `COMMIT`
5. 响应 `200 OK` + `new_revision`；前端 `editor_core::update_revision`

#### 验收条件（交互级）

##### 正常：完整编辑保存

- **GIVEN** 用户在 `/editor`，当前 diagram 已加载，本地 `revision=5`
- **WHEN** 用户创建表 "users"（3 字段：id BIGINT PK / name VARCHAR(255) NOT NULL / email VARCHAR(255) UNIQUE），1s 内不操作
- **THEN**
  - 表出现在画布指定坐标（默认 `(100, 100)`）
  - 1s debounce 触发 PUT，body 含 `expected_revision=5`
  - 服务端返回 `200 OK` + body `{ "new_revision": 6 }`
  - 前端 UI 右上角 revision 指示器从 "5" → "6"
  - 顶部 SaveState 指示从 "Saving..." → "Saved"

##### 正常：字段编辑持久化

- **GIVEN** 表 "users" 已存在，含字段 id / name
- **WHEN** 用户在 inspector 中新增字段 "email"（类型 `VARCHAR(255)`，勾选 `NOT NULL`）
- **THEN**
  - 字段列表立即渲染该行（Leptos signal 细粒度更新）
  - 1s debounce 触发 PUT
  - 服务端 `field` 表插入新行
  - 刷新页面后该字段仍在

##### 异常：409 revision 冲突

- **GIVEN** 客户端 `revision=5`，服务端最新 `revision=7`（被其他客户端更新过）
- **WHEN** 用户点击保存（触发 PUT）
- **THEN**
  - 服务端返回 `409 Conflict` + body `{ "current_revision": 7, "your_expected": 5 }`
  - 前端捕获错误，弹冲突对话框，列出 3 选项：
    - **Reload**：GET `/api/v1/diagrams/{id}` 拉取服务端最新版本覆盖本地
    - **Force**：PUT 携带 `expected_revision=7+1=8` 覆盖服务端
    - **Cancel**：保留本地与远端两份 JSON 供用户手动 merge

##### 异常：网络中断

- **GIVEN** 编辑过程中网络断开
- **WHEN** PUT 触发
- **THEN**
  - 前端捕获 `fetch` error
  - UI 顶部 SaveState 指示变红，显示「保存失败（离线）」
  - 3s 后自动重试（指数退避：3s / 6s / 12s，封顶 30s）
  - 网络恢复后 PUT 成功，revision 推进，UI 恢复 "Saved"

---

### S01 统一工作空间入口补充

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


### S02：加载分享链接图表（V1 ✅ P1）

#### 触发条件

用户访问 URL `https://coldrawdb.example.com/?share=<diagram_id>`。

#### 用户价值

无需登录、无需知道服务端地址，通过分享链接直接打开任意 diagram。关联痛点 **P01**（自托管 + 跨设备访问）。

#### 优先级

P1（V1 launch 必备，但相比 S01 略次）

#### 主路径

1. 浏览器加载 `index.html`，`lib.rs::mount_to_body` 启动 WASM
2. `editor_data_access` 解析 URL 参数 `share=<diagram_id>`
3. `GET /api/v1/diagrams/{id}` 拉取 diagram JSON
4. `200` → `editor_core::set_diagram` → `editor_render` 渲染到画布
5. `404` / `5xx` → 弹错误提示，保持空白画布，提供「创建新图表」入口

#### 验收条件（交互级）

##### 正常：有效分享链接

- **GIVEN** 服务端存在 diagram id=`abc-123-def`，对应一张包含 3 张表的 diagram
- **WHEN** 用户访问 `https://coldrawdb.example.com/?share=abc-123-def`
- **THEN**
  - 浏览器加载页面，loading 状态出现 < 200ms
  - GET 返回 200 + diagram JSON
  - 3 张表渲染到画布
  - URL 保持 share 参数（不污染浏览器历史）

##### 异常：无效分享链接

- **GIVEN** 服务端不存在 diagram id=`xxx-nonexistent`
- **WHEN** 用户访问 `https://coldrawdb.example.com/?share=xxx-nonexistent`
- **THEN**
  - GET 返回 `404`
  - 前端捕获错误，弹错误提示「分享链接无效或图表已删除」
  - 提供「创建新图表」按钮跳转 `/editor`

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

### S03：用户注册 / 登录 / Token 续期（V2 🟡）

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

### S04：创建 / 加入协作房间（V2 🟡）

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

### S05：OT 实时协作（V2 🟡）

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

## 原型 / 生产边界（适用于 S03～S05）

- 主原型内登录、HTTP、WS、OT、剪贴板与下载可为本地模拟，须标注演示/模拟。
- 规格中的 `data-testid` 是后续生产对齐锚点，**不得**仅因原型可演示就把场景标为全栈完成。
- 本提案合并后的文档状态：规格已对齐主原型；生产逐项实现对齐见变更 `implement-unified-prototype-spec-parity`。
