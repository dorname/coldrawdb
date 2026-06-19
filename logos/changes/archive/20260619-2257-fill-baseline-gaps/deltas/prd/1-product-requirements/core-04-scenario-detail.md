# ADDED — core-04-scenario-detail.md（新文档，Phase 1 第四节「核心场景详述」）

> 新增到 `logos/resources/prd/1-product-requirements/core-04-scenario-detail.md`
> 对应 prd-writer SKILL §Step 4 GIVEN/WHEN/THEN 验收条件规范
> 每个 V1 P0 场景含 ≥1 正常 G/W/T + ≥1 异常 G/W/T

---

## ADDED — 四、核心场景详述

> 场景编号与 `core-00-scenario-overview.md` 一致
> 本节仅详述 V1 ✅ 场景（S01 / S02）；V2 ❌ 场景（S03 / S04 / S05）的详述见各自变更提案

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

- **GIVEN** 用户访问 `https://coldrawdb.example.com/`（无 share 参数）
- **WHEN** 页面加载
- **THEN**
  - 走默认 LandingPage 或空白编辑器（V1 行为：landing → 点击「New」创建空 diagram）
  - 不弹错误

---

## 场景范围外（V2 计划，不在本变更范围内）

- `S03`：用户注册 / 登录 / Token 续期（需要 `users` + `auth_tokens` 表）— V2 候选
- `S04`：创建/加入协作房间（需要 `rooms` + `room_members` 表）— V2 候选
- `S05`：OT 实时协作（需要 `collab-server` + WS 网关）— V2 候选