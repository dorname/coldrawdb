# 业务场景概览（技术实现）

> 与 Phase 1 业务总览配套使用：`logos/resources/prd/1-product-requirements/core-00-scenario-overview.md`
> 本文档聚焦「技术实现状态」，按 `scenario-architect` SKILL §Step 5 输出

## 场景地图

| 编号 | 场景名称 | Phase 1 | Phase 2 | Phase 3 时序图 | API | 编排 | 状态 |
|------|---------|---------|---------|--------------|-----|------|------|
| S01 | 编辑并保存图表 | ✅ | ✅ | ✅ | ✅ | ✅ | **V1 已实现**；默认落在 `room-editor` 上下文 |
| S02 | 加载分享链接图表 | ✅ | ✅ | ✅ | ✅ | ✅ | **V1 已实现**；`?share=` 鉴权旁路 |
| S03 | 用户注册 / 登录 / Token 续期 | ✅ | ✅ | ✅ | ✅ | ✅ | **后端已实现**；生产前端 API/页面流已部分接入；相对主原型逐项对齐待 `implement-unified-prototype-spec-parity` |
| S04 | 创建 / 加入协作房间 | ✅ | ✅ | ✅ | ✅ | ✅ | **后端已实现**；生产前端已部分接入；逐项对齐待下一变更 |
| S05 | OT 实时协作 | ✅ | ✅ | ✅ | ✅ | ✅ | **后端已实现**；生产前端已部分接入；逐项对齐待下一变更 |
| S06 | AI 客户端通过 MCP 管理数据库图表 | ✅ | ✅ | ✅ | ✅ MCP | ✅ | **规格与实现推进中**；本提案仅回归边界，不改 MCP 契约 |

## S06 技术索引

| 场景 | 时序图 | Phase 2 | 工具契约 | 编排 | 实现目标 |
|---|---|---|---|---|---|
| S06 | `core-S06-ai-client-mcp.md` | `core-S06-mcp-service-design.md` | `mcp-tools.yaml` | `core-S06-mcp-service.json` | 独立 `coldrawdb-mcp` Rust 服务 |

## 场景依赖关系

```
V1：
├── S01 编辑保存（非 room：PUT + 409；room 内：见 S05，禁止 409 模态）
└── S02 分享加载（?share= 旁路）

V2 链式：
S03 鉴权（成功 → rooms）
  └─→ S04 房间（创建/打开/接受邀请 → room-editor）
        └─→ S05 OT（WS 连接态 / 排队 / 重连 / 本地降级）

回归边界：
S06 MCP ──→ 既有 HTTP diagram 白名单（本提案不改路径）
```

- **废止**：依赖图中「共享独立 collab-server + WS 网关（V2 引入）」作为未落地计划表述 → 改为 **backend 已实现 WS/OT**；是否再拆进程不在本提案范围。
- **V1 → V2**：`?share=` 仍为匿名只读兼容路径，与 room 成员写权限独立。

## 场景索引（V1 ✅ 技术维度）

| 场景 | 时序图 | Phase 2 设计 | 编排测试 | 相关功能规格 | 后端子模块 | 前端模块 |
|---|---|---|---|---|---|---|
| S01 | `core-S01-edit-and-save-diagram.md` | `core-S01-edit-and-save-design.md` | `core-S01-diagram-save.json` | `core-02-diagram-persistence.md` / `core-01a-table-and-field.md` / `core-0a-code-editor.md` | `diagrams_v1.rs` / `diagrams/service.rs` | `editor_data_access` / `editor_core` / `editor_panels` / `command_palette` / `code_view` |
| S02 | `core-S02-load-shared-diagram.md` | `core-S02-load-shared-diagram-design.md` | `core-S02-shared-link-load.json` | `core-02-diagram-persistence.md` / `core-05-top-menu-modals.md` | `diagrams_v1.rs` / `diagrams/service.rs` | `editor_data_access` / `editor_core` / `editor_render` / `editor_panels`（Share 模态） |

## 场景索引（V2 后端已实现，生产前端部分接入）

| 场景 | 时序图 | Phase 2 设计 | 编排测试 | API / DB | 备注 |
|---|---|---|---|---|---|
| S03 | `core-S03-user-auth.md` ✅ | `core-S03-user-auth-design.md` ✅ | `core-S03-user-auth.json` ✅ | `auth.yaml` + `coldrawdb-v2-auth.sql` | 成功默认进入 **rooms**（非 `/editor`） |
| S04 | `core-S04-room-lifecycle.md` ✅ | `core-S04-room-lifecycle-design.md` ✅ | `core-S04-room-lifecycle.json` ✅ | `rooms.yaml` + `coldrawdb-v2-rooms.sql` | 进入 **room-editor** |
| S05 | `core-S05-ot-collab.md` ✅ | `core-S05-ot-collab-design.md` ✅ | `core-S05-ot-collab.json` ✅ | `collab.yaml` + `coldrawdb-v2-collab.sql` | 连接态/排队/重连/本地降级；协作不弹 S01 409 |

## 与 Phase 1 业务总览的关系

| 维度 | Phase 1 业务总览 | Phase 3 技术总览（本文件） |
|---|---|---|
| V2 场景 | 列出真实状态 + 统一页面流 | 标注「后端已实现、生产前端部分接入、逐项对齐待下一变更」；规格映射含 rooms / room-editor / share 旁路 |

## 参考源

- `core-01-editor-prototype.html` — 唯一现行主原型
- `core-01-architecture-overview.md` — 页面状态 / 生产 vs 原型边界（本提案 delta）
- 下一代码变更合同：`implement-unified-prototype-spec-parity`

## 主原型与生产模块映射（摘要）

| 页面状态 | 前端（生产） | 后端 | 主原型 |
|---|---|---|---|
| auth | AuthUI + AuthClient | `auth_v1` | 同壳演示登录 |
| rooms / invite | RoomUI + RoomClient | `rooms_v1` | 本地房间数据 |
| room-editor | editor_* + CollabClient | diagrams + WS | 本地 OT 模拟 |
| share-readonly | EditorEntry + DataAccess | diagrams GET | Share 旁路演示 |

## 实现边界声明

> **实现边界声明**：
> - S01/S02 前后端已实现。
> - S03～S05 的 auth/rooms/collab REST、DB、WS 与测试已实现；生产前端已有**部分**接入，但不足以证明相对 `core-01-editor-prototype.html` 逐项完成。
> - 统一 HTML 主原型仅模拟 S03～S05 状态，**不建立**真实 auth/room/WS 连接，不能作为生产前端完成证据。
> - 默认登录后技术路径：`auth → rooms → room-editor`；`?share=` 旁路不阻断。
> - 本提案预期不新增 API/DB；仅补齐前端参与者、页面状态与异常映射。
> - S06 的工具设计源自 `core-S06-ai-client-mcp.md`；MVP 仍是独立 Rust stdio adapter，不受主原型 UI 对齐影响。
