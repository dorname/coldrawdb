# Delta — core-00-scenario-overview.md（新增 S07 数据字典）

## MODIFIED — 场景索引

| 场景 ID | 名称 | 版本 | 真实状态 | 关键流程 |
|---|---|---|---|---|
| S01 | 编辑并保存图表 | V1 | ✅ 前后端已实现 | 编辑 → debounce → PUT → revision |
| S02 | 加载分享链接图表 | V1 | ✅ 前后端已实现 | share 参数 → GET → 只读加载 |
| S03 | 用户注册/登录/Token 续期 | V2 | 🟡 后端已实现；生产前端 API/页面流已部分接入；相对主原型逐项对齐待下一变更 | register/login/refresh/logout/me → 进入 rooms |
| S04 | 创建/加入协作房间 | V2 | 🟡 后端已实现；生产前端 API/页面流已部分接入；相对主原型逐项对齐待下一变更 | room/invite/member CRUD → 进入 room-editor |
| S05 | OT 实时协作 | V2 | 🟡 后端已实现；生产前端 API/页面流已部分接入；相对主原型逐项对齐待下一变更 | WS connect/op/ack/sync/presence |
| S06 | AI 客户端通过 MCP 管理数据库图表 | V3 | 🔵 规格与实现推进中 | MCP initialize → tools/list → diagram CRUD/import/export |
| S07 | 管理数据字典并绑定字段 | V3 | 🔵 规格与实现推进中 | 字典 CRUD → 字典项 value/label → 字段绑定 → 随快照保存 / Markdown 导出 |

## MODIFIED — 场景图谱

```text
S01/S02 Web 编辑与分享 ───────────────┐
                                      ├─→ diagram/bridge API → SQLite
S03 鉴权 → S04 房间 → S05 OT ───────┤
                                      │
S06 AI 客户端 → MCP stdio adapter ───┘

S07 数据字典 ── 纯前端 state，随 S01 diagram JSON 快照持久化；
              S05 下字典编辑作为模型 op 自然同步（协议零变更）；无新增 API/DB 表
```

S06 复用 diagram/bridge API，不直接依赖 SQLite，也不依赖 S03～S05 生产前端。当前 diagram API 尚未强制 S03 鉴权，因此 S06 MVP 仅限本地可信 stdio。

S07 复用 S01 持久化通路（diagram JSON 顶层新增可选 `dictionaries` 数组），不新增端点与表；与 Enum/CustomType 的「仅前端 state」口径一致（见 core-01c §0）。

## MODIFIED — 场景 ↔ 文档映射

| 场景 | 时序图 | 测试用例 | 编排测试 | 实现清单条目 |
|---|---|---|---|---|
| S01 | `core-S01-edit-and-save-diagram.md` | `core-S01-test-cases.md` | `core-S01-diagram-save.json` | ✅ V1 |
| S02 | `core-S02-load-shared-diagram.md` | `core-S02-test-cases.md` | `core-S02-shared-link-load.json` | ✅ V1 |
| S03 | `core-S03-user-auth.md`（V2） | `core-S03-test-cases.md`（V2） | `core-S03-user-auth.json` | 🟡 规格对齐中 → 下一变更实现 |
| S04 | `core-S04-room-lifecycle.md`（V2） | `core-S04-test-cases.md`（V2） | `core-S04-room-lifecycle.json` | 🟡 规格对齐中 → 下一变更实现 |
| S05 | `core-S05-ot-collab.md`（V2） | `core-S05-test-cases.md`（V2） | `core-S05-ot-collab.json` | 🟡 规格对齐中 → 下一变更实现 |
| S07 | `core-S07-data-dictionary.md`（V3） | `core-S07-test-cases.md`（V3） | 无（无新 API，前端 UT/ST 覆盖） | 🔵 本变更实现 |

### S06 补充映射

| 场景 | 需求 | 功能设计 | 时序图 | 契约 | 测试 | 编排 |
|---|---|---|---|---|---|---|
| S06 | `core-S06-mcp-service-requirements.md` | `core-S06-mcp-service-design.md` | `core-S06-ai-client-mcp.md` | `mcp-tools.yaml` | `core-S06-test-cases.md` | `core-S06-mcp-service.json` |

### S07 补充映射

| 场景 | 功能设计 | 字段绑定规格 | 时序图 | 测试 |
|---|---|---|---|---|
| S07 | `core-01e-data-dictionary.md` | `core-01a-table-and-field.md` §2.5 | `core-S07-data-dictionary.md` | `core-S07-test-cases.md` |
