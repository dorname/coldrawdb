# S05：OT 实时协作 — 交互设计

> 模块：core | 场景：S05 | 版本：V2 | 优先级：P2
> 原型：`core-05-ot-collab-prototype.html`
> 前置：**S03 鉴权** + **S04 协作房间**（须在 room 内且 role ≠ viewer）
> Phase 1 输入：`core-00-scenario-overview.md` §S05 / `core-03-pain-points.md` P03
> 参考：drawdb main `CollabContext` 为 stub；coldrawdb V2 引入独立 **collab-server** + WS 网关

## 1. 产品类型与原型策略

| 项 | 结论 |
|---|---|
| 产品类型 | Web 应用（房间内编辑器 + 实时同步层） |
| 主原型 | `core-01-editor-prototype.html`（S01～S05 唯一评审入口） |
| 原型形式 | 单文件可交互 HTML（模拟双端：本地编辑 + 远端 op / 光标 / 重连） |
| 历史参考 | `core-05-ot-collab-prototype.html`（保留，不再作为验收入口） |
| 视觉基准 | 在统一协作编辑器上叠加 presence、远端光标、连接态 Banner 与演示控制台 |
| 痛点关联 | **P03**——消除「邮件传 JSON + 手动 merge」；多人同时改 schema 可收敛 |

## 2. 架构与数据流（交互视角）

```
┌─────────────┐     op/ack      ┌─────────────────┐     broadcast     ┌─────────────┐
│  Client A   │ ◄──────────────►│  collab-server  │◄─────────────────►│  Client B   │
│ (Leptos WASM)│   WS + JWT     │  transform OT   │                   │             │
└──────┬──────┘                 └────────┬────────┘                   └─────────────┘
       │ debounce snapshot              │ persist op_log
       ▼                                ▼
┌─────────────┐                 ┌─────────────────┐
│ actix-web   │◄── PUT checkpoint│ SQLite          │
│ REST API    │   (periodic)    │ operation_log   │
└─────────────┘                 └─────────────────┘
```

**与 S01 的关系**：room 模式下，**画布变更走 OT（即时）**；**快照持久化仍 debounce PUT**（如 5s 或 idle），revision 由 collab-server 统一推进，**不再弹 S01 的 409 冲突模态**（OT 已合并并发 op）。

## 3. WebSocket 会话

### 3.1 连接建立

**触发**：用户进入 `/editor/{diagramId}?room={roomId}` 且 role ∈ {owner, editor}

1. 前端持 S03 `access_token` → ``wss://host/ws/rooms/{roomId}?token=...``
2. collab-server 校验 JWT + room_member
3. **connected** 帧：`{ server_rev, members[], snapshot_hash? }`
4. StatusBar `[data-testid="ws-status"]`：`已连接 · OT 同步`
5. AppBar `[data-testid="room-presence"]`：在线成员绿点 + 头像

**viewer**：仅建立 **只读 WS**（接收 snapshot + presence，不发 op）

### 3.2 帧类型（UI 可感知）

| 帧 | 方向 | UI 反馈 |
|---|---|---|
| `op` | C→S | 本地 optimistic 渲染（已有则 skip） |
| `ack` | S→C | StatusBar rev 递增 `[data-testid="ot-rev"]` |
| `remote_op` | S→C | 远端变更动画 + Activity 条目 |
| `presence` | 双向 | 远端光标 / 选中高亮 |
| `sync` | S→C（重连） | Banner「正在同步…」→ 隐藏 |
| `error` | S→C | Toast + 可能降级只读 |

## 4. 子场景与交互流程

### S05.1 本地编辑广播

1. 用户 A 在 room 内创建表 `orders`
2. 客户端生成 op `{ type: "table.create", ... }` → WS 发送
3. 本地 **optimistic**：表立即出现在 A 的画布
4. 收到 `ack { server_rev: 43 }` → StatusBar 更新
5. 用户 B 收到 `remote_op` → B 画布动画插入表（无需刷新）

**原型锚点**：`[data-testid="editor-canvas"]` / `[data-testid="activity-feed"]`

---

### S05.2 Presence（在线与光标）

**AppBar** `[data-testid="room-presence"]`：

- 在线：头像 + 绿色角标 `[data-testid="presence-online"]`
- 离开：灰显（WS disconnect 30s 后移除）
- 最多显示 5 头像，超出 `+N`

**画布远端光标** `[data-testid="remote-cursor"]`：

- 每远端用户一条带 label 的 SVG 十字/箭头（颜色按 user_id 哈希）
- 位置随 `presence.cursor { x, y }` 更新（节流 50ms）
- 选中对象：远端选中框虚线 + 用户名 tag（避免覆盖本地选中实线）

---

### S05.3 并发编辑 OT 合并

1. A 改字段 `users.email` → NOT NULL
2. B 同时改 `users.email` 类型 → TEXT
3. collab-server `transform(opA, opB)` → 两客户端应用合并后 op
4. **无 409 模态**；Inspector 最终一致
5. 若不可合并（极少，如同时删表）：Toast「冲突已由服务器解决」+ Activity 记录

---

### S05.4 断线重连

1. WS 断开 → Banner `[data-testid="reconnect-banner"]`：「连接已断开，正在重连…（3/5）」
2. 本地编辑 **排队**（不丢失）；Banner 副文案「更改将在恢复后同步」
3. 重连成功 → 发送 `{ type: "sync", last_rev }` → 服务器补发 missed ops 或 full sync
4. Banner 关闭；Toast「已恢复协作」
5. 5 次失败 → 「无法连接协作服务」+ 按钮「仅本地编辑（会触发 409 风险）」/「刷新页面」

---

### S05.5 协作 Activity 侧栏

**可选折叠面板** `[data-testid="activity-feed"]`（Inspector 底部或 SideSheet）：

| 时间 | 条目示例 |
|---|---|
| 14:32 | Alice 创建了表 `orders` |
| 14:33 | Bob 添加了关系 `users → orders` |
| 14:34 | 你 修改了 `users.email` |

点击条目 → 画布 scrollIntoView 对应对象

## 5. UI 状态矩阵

| WS 状态 | StatusBar | 本地编辑 | 远端 op | Banner |
|---|---|---|---|---|
| connecting | 连接中… | ✅ optimistic | ❌ | — |
| connected | 已连接 · OT | ✅ | ✅ | — |
| reconnecting | 重连中… | ✅ 排队 | ❌ | 黄色 Banner |
| disconnected | 协作离线 | ✅ 排队 | ❌ | 红色 Banner |
| viewer | 只读 · 查看 | ❌ | 接收 | — |

## 6. 验收条件（交互级）

##### 正常：进入 room 建立 WS

- **GIVEN** 用户已登录且为 room editor，打开 `/editor/d-abc?room=r-001`
- **WHEN** 页面加载完成
- **THEN**
  - `[data-testid="ws-status"]` 显示「已连接 · OT 同步」
  - `[data-testid="room-presence"]` 至少包含当前用户绿点
  - 无 S01 409 冲突模态

##### 正常：收到远端创建表

- **GIVEN** A、B 同在 room，WS connected
- **WHEN** A 创建表 `orders`（B 未操作）
- **THEN**
  - B 的画布在 500ms 内出现 `orders` 表（`[data-testid="table-orders"]` 或等价）
  - B 的 `[data-testid="activity-feed"]` 新增「A 创建了表 orders」
  - B 的 `[data-testid="ot-rev"]` 递增

##### 正常：远端光标可见

- **GIVEN** A、B 在线
- **WHEN** A 移动鼠标于画布
- **THEN** B 可见 `[data-testid="remote-cursor"]` 带 A 的标签，位置随动

##### 正常：断线重连

- **GIVEN** 协作中 WS 意外断开
- **WHEN** 网络恢复，自动重连成功
- **THEN**
  - `[data-testid="reconnect-banner"]` 先显示后隐藏
  - 排队 op 全部同步，画布与 server_rev 一致
  - Toast「已恢复协作」

##### 异常：重连失败降级

- **GIVEN** 重试 5 次仍失败
- **WHEN** Banner 显示失败态
- **THEN**
  - 提供「刷新页面」与「仅本地编辑」选项
  - 选仅本地编辑时 StatusBar 警告「协作离线 · 409 风险」

##### 边界：viewer 不发送 op

- **GIVEN** 用户 role=viewer
- **WHEN** 尝试创建表
- **THEN** Tool Rail 新建 disabled；无 WS op 发出；仍可见远端 presence

## 7. 与 S01 / S04 的差异摘要

| 维度 | S01 单人 | S04 room 无 OT | S05 room + OT |
|---|---|---|---|
| 同步 | debounce PUT | 同 S01 | WS op 即时 + 周期 checkpoint PUT |
| 冲突 | 409 模态 | 409 模态 | OT merge，无模态 |
| presence | 无 | 静态头像占位 | 在线态 + 远端光标 |
| 后端 | actix-web | actix-web | + collab-server WS |

## 8. 原型操作指南

打开 `logos/resources/prd/2-product-design/2-page-design/core-05-ot-collab-prototype.html`：

| 操作 | 预期 |
|---|---|
| 默认 | 房间编辑器 + WS 已连接 |
| 「模拟 Alice 创建表」 | 画布出现 orders + Activity 条目 |
| 「模拟 Alice 光标」 | 远端光标移动 |
| 「模拟断线重连」 | Banner 流程 + rev 更新 |
| 「模拟重连失败」 | 降级 Banner |
| 「Viewer 模式」 | 只读 + 仍可见远端 op |

## 9. 反模式

- ❌ room 协作模式仍弹 S01 409 模态（OT 应已合并）
- ❌ viewer 可发送 op
- ❌ 断线期间静默丢弃本地 op
- ❌ presence 光标遮挡本地 primary 选中框且无区分样式

## 10. 单文件协作模拟器

### 10.1 模拟器定位

协作模拟器是主原型中的显式演示控制台，用确定性计时器和本地事件模拟 WebSocket 帧。它用于评审 UI 状态与业务逻辑，不建立真实网络连接，也不宣称 OT 算法已经生产接通。

### 10.2 模拟动作

| 演示动作 | 模拟帧/状态 | UI 结果 |
|---|---|---|
| Alice 移动光标 | `presence.cursor` | 紫色远端光标平滑移动并带姓名标签 |
| Alice 创建 `orders` | `remote_op: table.create` | 画布插入表、Activity 增项、server revision +1 |
| Bob 修改字段 | `remote_op: field.update` | Inspector/画布同步更新，字段短暂显示远端高亮 |
| 并发编辑 | 本地 `field.update` + remote op | Toast 提示 OT 已合并，不出现 409 模态 |
| 模拟断线 | `connected → reconnecting` | 黄色 Banner，本地写操作进入队列 |
| 恢复连接 | `sync(last_rev)` | 回放队列、补发远端操作、revision 递增、成功 Toast |
| 重连失败 | `reconnecting → failed` | 红色 Banner，显示刷新与仅本地编辑 |
| Viewer 模式 | `role=viewer` | 禁止发送 op，仍接收 presence 与 remote op |

### 10.3 状态不变量

- `connected`：本地命令立即 optimistic 应用并在短延迟后 ack；server revision 单调递增。
- `reconnecting`：本地命令可 optimistic 应用，但必须进入可见队列，StatusBar 显示待同步数量。
- `failed`：默认禁止继续协作写；选择「仅本地编辑」后允许本地命令，同时持续警告 409 风险。
- `viewer`：无论连接状态如何，均不得进入待发送队列或制造 ack；仍可接收远端状态。
- 远端操作必须同时更新画布、Activity 与 revision，避免只有动画而数据未变化。

### 10.4 操作队列与 OT 反馈

原型 store 保存 `serverRev`、`connection`、`pendingOps` 与 `activity`。本地命令经统一 dispatcher：先校验角色，再更新编辑历史；连接正常时模拟 ack，重连中则排队。恢复连接时按顺序清空队列并生成一条同步完成 Activity。

并发字段修改模拟结果必须可观察：最终字段同时保留本地约束修改与远端类型修改，并显示「服务器已合并 2 个并发操作」。协作模式不得弹出 S01 的 409 冲突模态。

### 10.5 锚点与动效

保留 `ws-status`、`ot-rev`、`remote-cursor`、`activity-feed`、`reconnect-banner`、`room-presence`。远端光标位置更新不超过 20fps；`prefers-reduced-motion: reduce` 下直接跳到目标位置并关闭脉冲动画。
