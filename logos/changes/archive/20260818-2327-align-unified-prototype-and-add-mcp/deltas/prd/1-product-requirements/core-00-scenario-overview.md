# Delta — core-00-scenario-overview.md（修改）

> module: core | proposal: align-unified-prototype-and-add-mcp

## MODIFIED — 场景索引

| 场景 ID | 名称 | 版本 | 真实状态 | 关键流程 |
|---|---|---|---|---|
| S01 | 编辑并保存图表 | V1 | ✅ 前后端已实现 | 编辑 → debounce → PUT → revision |
| S02 | 加载分享链接图表 | V1 | ✅ 前后端已实现 | share 参数 → GET → 只读加载 |
| S03 | 用户注册/登录/Token 续期 | V2 | 🟡 后端已实现，生产前端待接入 | register/login/refresh/logout/me |
| S04 | 创建/加入协作房间 | V2 | 🟡 后端已实现，生产前端待接入 | room/invite/member CRUD |
| S05 | OT 实时协作 | V2 | 🟡 后端已实现，生产前端待接入 | WS connect/op/ack/sync/presence |
| S06 | AI 客户端通过 MCP 管理数据库图表 | V3 | 🔵 规格变更中 | MCP initialize → tools/list → diagram CRUD/import/export |

## MODIFIED — 场景图谱

```text
S01/S02 Web 编辑与分享 ───────────────┐
                                      ├─→ diagram/bridge API → SQLite
S03 鉴权 → S04 房间 → S05 OT ───────┤
                                      │
S06 AI 客户端 → MCP stdio adapter ───┘
```

S06 复用 diagram/bridge API，不直接依赖 SQLite，也不依赖 S03～S05 生产前端。当前 diagram API 尚未强制 S03 鉴权，因此 S06 MVP 仅限本地可信 stdio。

## MODIFIED — 场景 ↔ 文档映射

追加：

| 场景 | 需求 | 功能设计 | 时序图 | 契约 | 测试 | 编排 |
|---|---|---|---|---|---|---|
| S06 | `core-S06-mcp-service-requirements.md` | `core-S06-mcp-service-design.md` | `core-S06-ai-client-mcp.md` | `mcp-tools.yaml` | `core-S06-test-cases.md` | `core-S06-mcp-service.json` |

