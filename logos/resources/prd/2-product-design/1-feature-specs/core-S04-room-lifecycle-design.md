# S04：创建 / 加入协作房间 — 交互设计

> 模块：core | 场景：S04 | 版本：V2 | 优先级：P2
> 现行原型：`core-01-editor-prototype.html`
> 历史参考：`core-04-collab-prototype.html`（不作为验收入口）
> 生产状态：后端已实现；生产前端 API/页面流已部分接入；相对主原型逐项对齐待下一变更 `implement-unified-prototype-spec-parity`
> 前置：**S03 鉴权**（须已登录）；后续：**S05 OT 实时协作**
> Phase 1 输入：`core-00-scenario-overview.md` §S04 / `core-03-pain-points.md` P03
> 参考：drawdb main `CollabContext` 为 **stub**，无房间 UI；本场景为 coldrawdb V2 net-new

## 1. 产品类型与原型策略

| 项 | 结论 |
|---|---|
| 生产实现 | 后端 rooms API/DB 与编排已实现；生产前端已有部分房间/邀请接入；须对齐主原型：房间列表、创建 Modal、邀请、成员 SideSheet、角色即时权限、进入 room-editor |

## 2. 信息架构（V2 增量）

### 2.1 路由

| 路由 | 页面 | 前置 | 说明 |
|---|---|---|---|
| `/rooms` | 我的协作房间 | S03 已登录 | 列表 + 创建入口 |
| `/rooms/:roomId` | 房间详情 / 设置 | 成员身份 | 成员 Tab + 进入编辑器 |
| `/editor/:diagramId?room=:roomId` | 房间内编辑 | room 成员 | AppBar 显示 room 上下文 |
| `/invite/:token` | 接受邀请 | S03 已登录 | 展示 room 预览 → 加入 |
| `/editor?share=` | 匿名分享 | 无 | **不变**（S02，与 room 无关） |

### 2.2 角色与权限

| 角色 | 画布 | 保存 PUT | 邀请成员 | 改角色 / 踢人 | 删房间 |
|---|---|---|---|---|---|
| **owner** | 读写 | ✅ | ✅ | ✅ | ✅ |
| **editor** | 读写 | ✅ | ✅（可配置关闭） | ❌ | ❌ |
| **viewer** | 只读 | ❌ | ❌ | ❌ | ❌ |

> viewer 进入编辑器时：Tool Rail 新建/关系工具 disabled；Inspector 字段表单 readonly；StatusBar 显示「只读 · 查看者」。

### 2.3 数据实体

| 实体 | 关键字段 | 说明 |
|---|---|---|
| `room` | `id`, `name`, `diagram_id`, `owner_id`, `created_at` | 1 room 绑定 1 diagram |
| `room_member` | `room_id`, `user_id`, `role`, `joined_at` | 复合唯一 `(room_id, user_id)` |
| `room_invite` | `token`, `room_id`, `role`, `expires_at`, `invited_by` | 默认 7 天过期；单次或多次可配置 |

### 2.4 房间生命周期

```
[Owner 创建 room] ──绑定 diagram──→ [room:active]
        │                                    │
        ├── 生成 invite ──→ [pending invite] ──被邀请人 accept──→ [room_member 新增]
        │                                    │
        ├── Owner 改 role / 移除成员 ────────┤
        └── Owner 删除 room ──→ [room:archived]（回收站；diagram 保留，仅解除 room 关联）
                                              │
                                              ├── Owner 恢复 ──→ [room:active]（diagram 已被其他 active room 占用时 409）
                                              └── Owner 彻底删除 ──→ [room:purged]（物理删除 room/member/invite；diagram 仍保留）
```

**房间列表页删除入口（fix-overflow-menu-room-delete-listview-io）**：

- 现状缺口：删除房间能力（`DELETE /rooms/{id}` 软删除归档）此前仅在编辑器内 RoomPanel 可达，房间列表页无入口。
- 房间卡片（`room-card-*`）追加 owner 可见的删除按钮 `room-card-delete-{id}`（卡片右上角 icon 按钮，悬停显现 + 常显焦点可达；`my_role != "owner"` 不渲染）：
  - 点击**不进入房间**，弹确认模态 `modal-delete-room-list`：标题「删除房间」、正文含房间名与「删除后进入回收站，可在回收站恢复或彻底删除」说明、确认按钮 `btn-confirm-delete-room-list`（danger）、取消按钮；
  - 确认 → 调 `DELETE /api/v1/rooms/{id}`（复用既有端点）→ 成功后卡片从列表即时移除 + Toast `notice-toast`「已删除房间」；403 显示「无权限删除此房间」模态不关闭；其他错误「删除失败，请稍后重试」；
  - 与编辑器内 RoomPanel 删除语义完全一致（软删除归档，diagram 保留）。
- 卡片删除按钮点击不得触发卡片本身的「进入房间」跳转（事件拦截）。
- **删除确认文案口径（feat-room-recycle-bin-dropdown-and-db-export 修正）**：所有删除确认入口（列表页 `modal-delete-room-list` 与编辑器内 RoomPanel `DeleteRoomModal`）统一为「删除后进入回收站」语义；旧文案「删除后不可恢复，房间及其所有协作数据将被永久删除」废止（与软删除实际行为矛盾）。

**回收站（feat-room-recycle-bin-dropdown-and-db-export）**：

- 房间列表页提供「回收站」入口 `btn-room-trash`（页头区，与「创建房间」同级），点击切换到回收站视图 `room-trash-view`；回收站视图提供「返回房间列表」`btn-trash-back`。
- 回收站数据源：`GET /api/v1/rooms?archived=true`（仅返回本人为 owner 的已归档房间，按归档时间倒序）。
- 回收站列表项 `room-trash-item-{id}` 展示：房间名、归档时间（`archivedAt`）；每行两个操作：
  - 「恢复」`btn-restore-room-{id}` → `POST /api/v1/rooms/{id}/restore` → 成功 Toast「已恢复房间」并从回收站移除；409 提示「该 diagram 已在其他协作房间中，无法恢复」；403 提示「无权限恢复此房间」。
  - 「彻底删除」`btn-purge-room-{id}` → 弹二次确认模态 `modal-purge-room`：标题「彻底删除房间」、正文含房间名与「彻底删除后不可恢复，房间的协作数据（成员/邀请）将被永久删除；Diagram 保留」、确认按钮 `btn-confirm-purge-room`（danger）→ `DELETE /api/v1/rooms/{id}/permanent` → 成功 Toast「已彻底删除」并从回收站移除。
- 空回收站显示空态文案「回收站为空」。
- 回收站视图内不提供「进入房间」跳转（归档房间不可进入）。

## 3. 子场景与交互流程

### S04.1 创建协作房间

**入口**：

- `/rooms` 页「创建房间」按钮 `[data-testid="btn-create-room"]`
- 编辑器 AppBar user-menu →「为此 diagram 创建协作房间」

**Create Room 模态** `[data-testid="modal-create-room"]`：

| 字段 | 必填 | 校验 |
|---|---|---|
| room_name | ✅ | 1–64 字符 |
| diagram | ✅ | 下拉已有 diagram 或当前 diagram；选「新建空白模型」时先创建 diagram 再绑定房间。选项数据源 `GET /diagrams/queryAll` **只返回未软删的 diagram**（`is_deleted=0`，fix-select-bg-diagram-delete-and-log-format）；选中既有图时选择器下方显示「删除此图表」按钮 `[data-testid="create-room-diagram-delete"]`，点击弹二次确认模态 `[data-testid="modal-delete-diagram"]`（文案：图表及其画布内容将被删除；选项本身不含已绑定图），确认调 `DELETE /api/v1/diagrams/{id}`，成功后刷新选项、选中态回退「新建空白模型」并 Toast「已删除图表」 |
| database | 条件必填 | **仅当 diagram 选「新建空白模型」时露出** `[data-testid="create-room-database"]`：Generic / MySQL / PostgreSQL 三项，默认 Generic；创建 diagram 时随 `POST /api/v1/diagrams` 的 `database` 入参落账，建图即确定字段类型清单口径 |
| default_role | ❌ | 邀请默认角色 editor / viewer |

**流程**：

1. 填写表单 → 若选「新建空白模型」：先 `POST /api/v1/diagrams` `{ name, database }`（database 透传所选引擎，缺省 Generic）→ 再 `POST /api/v1/rooms` `{ name, diagram_id }`；若选既有 diagram：直接 `POST /api/v1/rooms`（引擎随既有 diagram，不在创建流程修改）
2. **201** → Toast「房间已创建」→ 跳转 `/editor/{diagramId}?room={roomId}`
3. AppBar 出现 `[data-testid="room-badge"]` + `[data-testid="btn-invite"]`；AppBar 引擎下拉显示该 diagram 的引擎且**禁用（只读标识，见约束）**，Inspector 字段类型清单按引擎过滤（衔接 diagram-database-dialect 的 `types_for_database` 通路，清单口径见 `core-01a-table-and-field.md` §2.2 MVP 短清单）
4. **409** diagram 已绑定其他 room → 提示「该 diagram 已在房间 X 中」

**约束**：

- 引擎枚举与 AppBar 引擎下拉严格对齐（Generic / MySQL / PostgreSQL 三项；Sqlite/Mssql/Oracle 不露出）
- **引擎创建后锁定**：diagram 引擎在创建时确定（本流程「新建空白模型」分支或既有 diagram 自带），进入房间编辑器后 AppBar `[data-testid="app-db-select"]` 禁用，仅作引擎只读标识；中途切换会使既有字段类型与新引擎清单脱节，故不开放（fix-pg-types-listview-zindex-lock-engine）
- 新建图的引擎前置选择由本流程（「新建空白模型」分支）唯一承载——生产端无独立新建图入口（TopBar/TopMenuBar 为死代码，2026-09-06 勘察确认），如未来恢复独立新建图模态需同步增加 `new-diagram-database` 下拉

---

### S04.2 邀请成员

**入口**：编辑器 AppBar `[data-testid="btn-invite"]` 或房间设置页

**Invite 模态** `[data-testid="modal-invite"]`：

1. 选择角色：editor / viewer（owner 不可通过邀请创建）
2. 生成链接：`{public_base_url}/invite/{token}` `[data-testid="invite-url"]`。`public_base_url` 取后端 `PUBLIC_BASE_URL` 配置，缺省由请求 Host header（含 scheme 与非默认端口）推导；链接必须是**可公开访问的完整 URL**，**禁止硬编码 `localhost` / `127.0.0.1` / 无端口占位 host**——被邀请人跨机器打开该链接必须可直接访问
3. 可选：输入邮箱发送（依赖 S03 用户存在或 pending 注册）
4. 「复制链接」→ Toast；链接 7 天有效（过期后 UI 灰显 + 重新生成）

**成员列表面板** `[data-testid="room-members-panel"]`（SideSheet 或 Inspector 附加 Tab）：

- 每行：头像 + display_name + role Tag + 在线点（S05 实装 presence）
- owner 行：role 下拉 disabled
- owner 对其他成员：role 下拉（editor ↔ viewer）+ 「移除」

---

### S04.3 接受邀请并加入

**触发**：被邀请人打开 `/invite/{token}`（未登录 → redirect `/login?redirect=/invite/{token}`）

**Invite Accept 页** `[data-testid="invite-accept-page"]`：

1. 展示 room 名称、diagram 标题、邀请人、分配角色
2. 「加入房间」→ `POST /api/v1/rooms/invites/{token}/accept`
3. **200** → 跳转 `/editor/{diagramId}?room={roomId}`
4. **410** token 过期 → 「邀请已失效」+ 联系邀请人
5. **403** 已是成员 → 直接跳转编辑器

---

### S04.4 房间内编辑（与 S01 衔接）

**AppBar 增量**（room 上下文）：

```
[← 空间] [C] [room-badge: 评审周会…] [diagram 标题…] [成员头像×3 +2] [邀请] ... [user-menu]
```

- **显性返回入口（fix-appbar-roomname-back-and-import-merge）**：room-badge 左侧新增「← 空间」按钮（`data-testid="btn-back-to-rooms"`，ghost 样式，图标 + 文字「空间」），点击返回房间列表页（`on_open_rooms`，与 room-badge 原有点击行为一致）；room-badge 点击行为保留不变
- **名称缩略**：room-badge 名称与 diagram 标题超长时 `max-width` + `text-overflow: ellipsis` + `white-space: nowrap` 缩略，并带 `title` 属性悬停显示全文（对齐主原型 `.room-badge` 已有口径：badge max-width 190px、strong ellipsis）

**与 S01 差异**：

| S01 私有编辑 | S04 room 内编辑 |
|---|---|
| PUT 仅 owner 校验 | PUT 校验 `room_member.role ∈ {owner, editor}` |
| 409 冲突 modal 单人 | 409 仍弹窗；S05 将引入 OT 减少冲突 |
| 无成员 UI | `[data-testid="room-presence"]` 占位（S05 接 WS） |

**离开房间**：user-menu →「离开房间」→ Confirm → `DELETE /api/v1/rooms/{id}/members/me`（owner 需先转让或删 room）

## 4. 验收条件（交互级）

##### 正常：Owner 创建房间并进入编辑器

- **GIVEN** 用户已通过 S03 登录，当前 diagram `d-abc` 未绑定 room
- **WHEN** 用户打开 `[data-testid="modal-create-room"]`，输入 room_name=`评审周会`，提交
- **THEN**
  - 跳转 `/editor/d-abc?room=r-001`
  - AppBar 显示 `[data-testid="room-badge"]` 文案「评审周会」
  - `[data-testid="btn-invite"]` 可点击

##### 正常：复制邀请链接

- **GIVEN** 用户在 room 内且 role=owner
- **WHEN** 用户点击 `[data-testid="btn-invite"]`，选择 role=editor，点击复制
- **THEN**
  - `[data-testid="invite-url"]` 为完整 URL，匹配 `^{scheme}://{host}(:{port})?/invite/{token}$`，host/port 与当前部署实际对外地址一致（含非默认端口）
  - 链接不含硬编码 `localhost` / `127.0.0.1`；被邀请人在另一台机器上打开该链接可直接访问邀请页
  - Toast「链接已复制」

##### 正常：被邀请人加入

- **GIVEN** 用户 B 已登录，有效 invite token
- **WHEN** B 访问 `/invite/{token}` 并点击 `[data-testid="btn-accept-invite"]`
- **THEN**
  - B 成为 room_member，role=editor
  - 跳转编辑器且可编辑画布（非 viewer 限制）

##### 正常：Viewer 只读

- **GIVEN** 用户 C 的 room role=viewer
- **WHEN** C 进入 `/editor/d-abc?room=r-001`
- **THEN**
  - StatusBar 显示「只读 · 查看者」
  - Tool Rail 新建/关系工具 disabled
  - `[data-testid="btn-invite"]` hidden 或 disabled

##### 异常：未登录访问 /rooms

- **GIVEN** 匿名用户
- **WHEN** 访问 `/rooms`
- **THEN** 重定向 `/login?redirect=/rooms`

##### 异常：邀请过期

- **GIVEN** invite token 已过期
- **WHEN** 用户打开 `/invite/{token}`
- **THEN**
  - 页面显示「邀请已失效」
  - 无「加入房间」按钮；提供「返回首页」

##### 异常：Owner 移除成员

- **GIVEN** owner 在 `[data-testid="room-members-panel"]`
- **WHEN** owner 对成员 B 点击「移除」并 Confirm
- **THEN**
  - B 从列表消失
  - B 再次访问该 room URL 得 403 + 「你已不是成员」

##### 正常：删除后从回收站恢复

- **GIVEN** 用户为 room `r-001` 的 owner，且已将其删除（归档）
- **WHEN** 用户在 `/rooms` 点击 `[data-testid="btn-room-trash"]` 进入回收站，对 `r-001` 点击 `[data-testid="btn-restore-room-r-001"]`
- **THEN**
  - 调 `POST /api/v1/rooms/r-001/restore` 返回 200
  - 回收站列表移除该项；Toast「已恢复房间」
  - 返回房间列表后 `r-001` 重新出现

##### 正常：回收站彻底删除

- **GIVEN** 回收站中存在已归档 room `r-002`（owner 为当前用户）
- **WHEN** 用户点击 `[data-testid="btn-purge-room-r-002"]`，在 `modal-purge-room` 中点击 `btn-confirm-purge-room`
- **THEN**
  - 调 `DELETE /api/v1/rooms/r-002/permanent` 返回 204
  - 回收站列表移除该项；Toast「已彻底删除」
  - 再次调 `GET /api/v1/rooms?archived=true` 不含 `r-002`；`POST /rooms/r-002/restore` 返回 404

##### 异常：恢复冲突（diagram 已绑定其他 active room）

- **GIVEN** room `r-003` 已归档，其 diagram 已绑定另一 active room
- **WHEN** 用户在回收站对 `r-003` 点击「恢复」
- **THEN** 返回 409 `ROOM_DIAGRAM_TAKEN`；提示「该 diagram 已在其他协作房间中，无法恢复」；回收站列表不变

##### 异常：active 房间不可彻底删除

- **GIVEN** room `r-004` 处于 active 状态
- **WHEN** 调 `DELETE /api/v1/rooms/r-004/permanent`
- **THEN** 返回 404 `ROOM_NOT_FOUND`（防止绕过回收站）

## 页面锚点（主原型强制）

保留并作为生产对齐目标：`rooms-list-page`、`btn-create-room`、`room-list`、`room-badge`、`btn-invite`、`room-presence`、`room-members-panel`、`invite-url`、`btn-accept-invite`、`invite-accept-page`。

fix-overflow-menu-room-delete-listview-io 新增：`room-card-delete-{id}`（owner 可见删除按钮）、`modal-delete-room-list`（列表页删除确认模态）、`btn-confirm-delete-room-list`（确认删除）。

feat-room-recycle-bin-dropdown-and-db-export 新增：`btn-room-trash`（回收站入口）、`room-trash-view`（回收站视图）、`btn-trash-back`（返回房间列表）、`room-trash-item-{id}`（回收站列表项）、`btn-restore-room-{id}`（恢复）、`btn-purge-room-{id}`（彻底删除）、`modal-purge-room`（彻底删除二次确认模态）、`btn-confirm-purge-room`（确认彻底删除）。

浮层关闭后不得遗留可拦截点击的遮罩。

## 角色切换反馈

角色切换必须即时更新 ToolRail、Inspector、邀请按钮、StatusBar 与可发送写操作的能力；Viewer 禁用须同时阻断事件并给出原因 Toast。

## 5. 与 S03 / S05 的边界

| 场景 | S04 负责 | 不在 S04 |
|---|---|---|
| S03 | 登录态、JWT、user-menu | — |
| S04 | room CRUD、invite、role、成员 UI | 实时光标、OT op |
| S05 | — | WS 连接、`transform(a,b)`、presence 绿点动画 |

## 6. 原型操作指南

打开 `logos/resources/prd/2-product-design/2-page-design/core-01-editor-prototype.html`：

| 操作 | 预期 |
|---|---|
| 默认登录后 | 房间列表 |
| 「创建房间」 | 模态 → 提交 → 进入 room 编辑器视图 |
| 「邀请成员」 | 邀请模态 + 复制链接 |
| 「模拟 Viewer 视角」 | 切换只读 UI |
| 「接受邀请」视图 | 邀请预览页 → 加入 → 编辑器 |
| 「成员管理」 | SideSheet 改 role / 移除 |
| 切换 dark 主题 | `data-mode` 切换；房间卡片（room-card）、新建房间卡片（new-room-card）、用户菜单（menu-item）、标签（tag/tag--brand）与邀请预览页在暗色玻璃背景下对比度 ≥ WCAG AA 4.5:1，卡片边框使用 `--line-strong`（.30）清晰可辨；成员头像 initials（含 #aa8cff / #68aef2 / 品牌渐变填充）深色字对比度 ≥ 4.5:1；invite 页 h1 品牌色强调清晰可辨 |

`core-04-collab-prototype.html` 中成员按钮、ToolRail 和角色选择缺少完整事件绑定，只保留为历史参考，不纳入现行修复与验收。
## 7. 反模式

- ❌ 允许匿名用户创建 room（必须 S03）
- ❌ viewer 可触发 PUT 保存（权限漏洞）
- ❌ 同一 diagram 绑定多个 active room（数据不一致）
- ❌ 在 S04 原型承诺实时多人光标（属 S05）

## 8. 生产实现状态

主原型使用本地模拟房间数据。真实 room CRUD、邀请、角色权限由 backend 提供。

准确状态：

1. 后端与编排已完成
2. 生产前端已有部分调用与页面流
3. 不得仅因 `data-testid` 存在于规格就标记全栈完成
4. 逐项 UI/权限对齐合同见本提案与验收矩阵；实现见下一变更

## 9. 单文件房间生命周期演示

### 9.1 房间视图

登录成功后进入房间与最近项目视图。默认展示「评审周会」和「架构对齐」两个确定性演示房间，每张卡片显示关联 diagram、成员头像、当前角色、最近活动和连接状态。

创建房间必须在同一文件的 Modal 内完成名称、diagram 与默认邀请角色校验；提交后创建本地 room 状态并直接进入协作编辑器。不得依赖真实路由或刷新。

### 9.2 编辑器内房间管理

| 能力 | Owner | Editor | Viewer |
|---|---:|---:|---:|
| 编辑表/字段/关系 | 是 | 是 | 否 |
| 创建邀请 | 是 | 是 | 否 |
| 修改成员角色 | 是 | 否 | 否 |
| 移除成员 | 是 | 否 | 否 |
| 接收远端操作/presence | 是 | 是 | 是 |
| 删除或归档房间 | 是 | 否 | 否 |

角色切换由协作演示控制台触发，必须即时更新 ToolRail、Inspector、邀请按钮、StatusBar 和可发送操作的能力；Viewer 的禁用不能只靠视觉灰显，事件处理也必须阻止写操作并给出原因 Toast。

### 9.3 邀请与成员

- 邀请 Modal 可选择 Editor/Viewer、生成确定性邀请 URL、模拟复制与打开邀请预览。
- 邀请预览覆盖有效、已过期两个分支；有效邀请接受后进入相同房间，过期邀请不显示加入按钮。
- 成员 SideSheet 展示在线态、角色与最后活动；Owner 可修改 Editor/Viewer 并在确认后移除成员。
- Owner 自身角色不可直接修改或移除；需在说明中提示先转让房间。

### 9.4 锚点

保留 `rooms-list-page`、`btn-create-room`、`room-list`、`room-badge`、`btn-invite`、`room-presence`、`room-members-panel`、`invite-url`、`btn-accept-invite`。浮层关闭后，不得遗留可拦截点击的遮罩。
