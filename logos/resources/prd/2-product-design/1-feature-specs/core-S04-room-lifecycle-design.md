# S04：创建 / 加入协作房间 — 交互设计

> 模块：core | 场景：S04 | 版本：V2 | 优先级：P2
> 原型：`core-04-collab-prototype.html`
> 前置：**S03 鉴权**（须已登录）；后续：**S05 OT 实时协作**
> Phase 1 输入：`core-00-scenario-overview.md` §S04 / `core-03-pain-points.md` P03
> 参考：drawdb main `CollabContext` 为 **stub**，无房间 UI；本场景为 coldrawdb V2 net-new

## 1. 产品类型与原型策略

| 项 | 结论 |
|---|---|
| 产品类型 | Web 应用（编辑器 + 房间管理 SideSheet / 模态） |
| 原型形式 | 可交互 HTML（房间列表 → 创建 → 编辑器内协作 → 邀请 / 加入） |
| 视觉基准 | 延续 `core-07` token + 与 `core-01-editor-prototype.html` 一致的 AppBar 栅格 |
| 痛点关联 | **P03** 团队评审——将「导出 JSON 邮件来回」替换为「同一 diagram 房间内协作入口」 |

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
        └── Owner 删除 room ──→ [room:archived]（diagram 保留，仅解除 room 关联）
```

## 3. 子场景与交互流程

### S04.1 创建协作房间

**入口**：

- `/rooms` 页「创建房间」按钮 `[data-testid="btn-create-room"]`
- 编辑器 AppBar user-menu →「为此 diagram 创建协作房间」

**Create Room 模态** `[data-testid="modal-create-room"]`：

| 字段 | 必填 | 校验 |
|---|---|---|
| room_name | ✅ | 1–64 字符 |
| diagram | ✅ | 下拉已有 diagram 或当前 diagram |
| default_role | ❌ | 邀请默认角色 editor / viewer |

**流程**：

1. 填写表单 → `POST /api/v1/rooms` `{ name, diagram_id }`
2. **201** → Toast「房间已创建」→ 跳转 `/editor/{diagramId}?room={roomId}`
3. AppBar 出现 `[data-testid="room-badge"]` + `[data-testid="btn-invite"]`
4. **409** diagram 已绑定其他 room → 提示「该 diagram 已在房间 X 中」

---

### S04.2 邀请成员

**入口**：编辑器 AppBar `[data-testid="btn-invite"]` 或房间设置页

**Invite 模态** `[data-testid="modal-invite"]`：

1. 选择角色：editor / viewer（owner 不可通过邀请创建）
2. 生成链接：`https://coldrawdb.example.com/invite/{token}` `[data-testid="invite-url"]`
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
[C] [room-badge: 评审周会] [diagram 标题] [成员头像×3 +2] [邀请] ... [user-menu]
```

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
  - `[data-testid="invite-url"]` 含 `/invite/` token
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

## 5. 与 S03 / S05 的边界

| 场景 | S04 负责 | 不在 S04 |
|---|---|---|
| S03 | 登录态、JWT、user-menu | — |
| S04 | room CRUD、invite、role、成员 UI | 实时光标、OT op |
| S05 | — | WS 连接、`transform(a,b)`、presence 绿点动画 |

## 6. 原型操作指南

打开 `logos/resources/prd/2-product-design/2-page-design/core-04-collab-prototype.html`：

| 操作 | 预期 |
|---|---|
| 默认 | 房间列表（模拟已登录） |
| 「创建房间」 | 模态 → 提交 → 进入 room 编辑器视图 |
| 「邀请成员」 | 邀请模态 + 复制链接 |
| 「模拟 Viewer 视角」 | 切换只读 UI |
| 「接受邀请」视图 | 邀请预览页 → 加入 → 编辑器 |
| 「成员管理」 | SideSheet 改 role / 移除 |

## 7. 反模式

- ❌ 允许匿名用户创建 room（必须 S03）
- ❌ viewer 可触发 PUT 保存（权限漏洞）
- ❌ 同一 diagram 绑定多个 active room（数据不一致）
- ❌ 在 S04 原型承诺实时多人光标（属 S05）
