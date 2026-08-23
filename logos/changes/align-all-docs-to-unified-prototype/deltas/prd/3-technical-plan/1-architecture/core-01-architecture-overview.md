# Delta — core-01-architecture-overview.md（修改）

> module: core | proposal: align-all-docs-to-unified-prototype

## ADDED — 统一原型对齐补充：文档头覆盖范围

> 覆盖：V1（S01/S02 全栈）+ V2（auth / rooms / collab REST+WS 已落地于 backend）+ V3（S06 MCP adapter，本提案仅作回归边界）
> 唯一现行 HTML 主原型：`core-01-editor-prototype.html`（只读事实输入；演示器不建立真实 REST/WS）
> 历史参考原型：`core-03/04/05-*-prototype.html`（不作验收入口）
> 本提案预期：**不新增 API 端点、不新增 DB 表/迁移**；若行为已有契约，仅补前端参与者与异常映射

## ADDED — 1.2 现行页面状态与系统边界

生产前端与规格必须以统一工作空间页面状态为准（对齐 `core-00-information-architecture.md`）：

| 状态 ID | 视图 | 技术触发 | 下一跳 |
|---|---|---|---|
| `auth` | 登录 / 注册 | 未登录访问受保护路由 | `rooms`（登录成功） |
| `rooms` | 房间与最近项目 | S03 会话有效 | `room-editor` / `invite` |
| `invite` | 邀请预览 / 失效 | `/invite/{token}` | `room-editor` 或停留失效 |
| `room-editor` | 协作 ER 编辑器 | 房间成员 | 可回 `rooms` |
| `share-readonly` | 匿名只读分享 | `?share=<id>` | **不被鉴权拦截** |

```text
Browser (frontend-rs)
  auth ──登录成功──→ rooms ──创建/打开/接受邀请──→ room-editor
                         ▲                              │
                         └──────── room-badge / 退出 ────┘
  ?share= ──────────→ share-readonly（S02 旁路）

主原型 HTML：仅本地状态机演示上述流转；禁止当作生产完成证据
```

废止默认路径：历史 Landing → 空白 `/editor` 直达。

## ADDED — 统一原型对齐补充：V2 系统上下文端点

在既有 diagrams/bridge 之外，backend 已挂载（本提案不新增）：

| 域 | 路径族 | 场景 |
|---|---|---|
| diagrams | `/api/v1/diagrams/*` | S01 / S02 / S05 checkpoint |
| bridge | `/api/v1/bridge/*` | 导入导出 |
| auth | `/api/v1/auth/*` | S03 |
| rooms | `/api/v1/rooms/*` | S04 |
| collab REST | collab head/ops（见 `collab.yaml`） | S05 |
| WS | `/ws/rooms/{roomId}` | S05 |

> 原文「V2 计划新增独立 `collab-server`」以**现行事实**覆盖：协作 WS/OT 已由 backend 实现；后续是否拆独立进程不在本提案范围。

## ADDED — 2.7 前端模块职责：生产 vs 主原型

| 层 | 职责 | 禁止 |
|---|---|---|
| `frontend-rs`（`editor_data_access` + auth/room/collab 客户端） | **唯一**真实 REST / WS 出口；Bearer、refresh、room 成员校验、OT 队列 | 绕过 data_access 直连 fetch/WS |
| `editor_core` / panels / render | 状态机、布局壳（AppBar / ToolRail / Inspector / StatusBar）、画布 | 自造持久化语义 |
| `core-01-editor-prototype.html` | 演示页面流、状态文案、锚点与交互连贯性 | 作为生产 REST/WS 完成证据；演示登录/WS 不得写入强制生产契约 |

**状态表述（强制）**：

1. 后端 auth/rooms/collab 已实现并可编排验收
2. 生产前端 API/页面流已**部分**接入
3. 相对主原型的结构 / 视觉 / 交互逐项对齐，由下一变更 `implement-unified-prototype-spec-parity` 实现与验收
4. 不得因主原型可演示而将 S03～S05 标为全栈完成

## ADDED — 5.7 页面流数据源（REST / WS）

| 页面状态 | 权威状态来源 | UI 反馈锚点（示例） |
|---|---|---|
| auth | `POST /auth/login|register`、`GET /auth/me`、`POST /auth/refresh|logout` | `login-form` / `session-indicator` |
| rooms | `GET/POST /rooms` | `rooms-list-page` / `room-list` |
| invite | `GET/POST .../invites/{token}` | `invite-accept-page` / `btn-accept-invite` |
| room-editor | rooms 成员 + S01 PUT 或 S05 WS | `room-editor-page` / `room-badge` / `save-state` / `ws-status` |
| share-readonly | `GET /diagrams/{id}`（匿名） | 只读画布；写工具禁用 |

协作房间内并发合并走 S05；**禁止**对已 OT 合并的并发弹出 S01 `modal-conflict`（409）。

## MODIFIED — 9. V1 边界

下列条目仅描述 **V1 历史边界**，不得再写为现行系统事实：

- ~~❌ OT 实时协作（V2 计划）~~ → V2 后端已实现；生产前端逐项对齐待下一变更
- ~~❌ WebSocket（V1 全 HTTP）~~ → `/ws/rooms/{roomId}` 已存在

仍有效：V1 单人非 room 编辑仍走 S01 PUT + 409；微服务拆分 / PostgreSQL 等边界不变。

## ADDED — S06 回归边界（本提案不改）

- MCP stdio adapter（`coldrawdb-mcp`）不计入统一原型 UI 对齐范围
- 本提案合并后，S06 仅作回归：不得因页面流文案变更破坏 MCP → HTTP 白名单路径与 redaction 边界
- Streamable HTTP 仍禁止在权限前置未完成前部署（见原文 MCP adapter 节）

## MODIFIED — 10. 对齐参考源

- `core-01-editor-prototype.html` — 唯一现行主原型（页面状态事实输入）
- `core-00-information-architecture.md` — auth / rooms / invite / room-editor
- `core-S03-user-auth.md` / `core-S04-room-lifecycle.md` / `core-S05-ot-collab.md` — V2 时序
- `auth.yaml` / `rooms.yaml` / `collab.yaml` — 既有契约（本提案预期无新增端点）
