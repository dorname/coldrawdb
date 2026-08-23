# Delta — core-00-scenario-overview.md（修改）

> module: core | proposal: align-all-docs-to-unified-prototype

## ADDED — 统一原型对齐补充：文档头覆盖范围

> 覆盖范围：V1（S01/S02 全栈已实现）+ V2（S03～S05 后端已实现；生产前端 API/页面流已部分接入；相对统一主原型的逐项对齐由本提案收口规格，由下一变更 `implement-unified-prototype-spec-parity` 实现）+ V3（S06 MCP 实现中）
> 场景编号全局唯一，由 `logos-project.yaml` 的 `scenario_counter.next_id` 维护
> **唯一现行 HTML 主原型**：`prd/2-product-design/2-page-design/core-01-editor-prototype.html`（`core-03/04/05-*-prototype.html` 仅历史参考，不作验收入口）

## MODIFIED — 场景索引

| 场景 ID | 名称 | 版本 | 真实状态 | 关键流程 |
|---|---|---|---|---|
| S01 | 编辑并保存图表 | V1 | ✅ 前后端已实现 | 编辑 → debounce → PUT → revision |
| S02 | 加载分享链接图表 | V1 | ✅ 前后端已实现 | share 参数 → GET → 只读加载 |
| S03 | 用户注册/登录/Token 续期 | V2 | 🟡 后端已实现；生产前端 API/页面流已部分接入；相对主原型逐项对齐待下一变更 | register/login/refresh/logout/me → 进入 rooms |
| S04 | 创建/加入协作房间 | V2 | 🟡 后端已实现；生产前端 API/页面流已部分接入；相对主原型逐项对齐待下一变更 | room/invite/member CRUD → 进入 room-editor |
| S05 | OT 实时协作 | V2 | 🟡 后端已实现；生产前端 API/页面流已部分接入；相对主原型逐项对齐待下一变更 | WS connect/op/ack/sync/presence |
| S06 | AI 客户端通过 MCP 管理数据库图表 | V3 | 🔵 规格与实现推进中 | MCP initialize → tools/list → diagram CRUD/import/export |

## ADDED — 统一页面状态流（产品基线）

主原型与生产前端的默认登录后路径必须一致：

```text
[登录/注册]（S03）
     │ 登录成功
     ▼
[房间与最近项目]（S04）
     ├── 创建房间 ───────────────┐
     ├── 打开已有房间 ───────────┤
     └── 接受邀请 ───────────────┤
                                  ▼
                        [协作 ER 编辑器]（S01 + S04/S05）
                         ├── 编辑/关系/撤销/保存（S01）
                         ├── 导入/导出/代码/分享
                         ├── 成员/角色/邀请（S04）
                         └── OT/presence/重连（S05）

旁路：`?share=<id>` → 匿名只读加载（S02，不阻断鉴权拦截）
```

约定：

- 演示器 / 模拟错误 / 示例数据仅表达体验，**不自动成为生产需求**。
- 生产语义以真实 REST / WS 状态为准；`data-testid` 仅作测试锚点。
- 独立历史原型不得再作为新增功能或验收的事实来源。

## MODIFIED — 场景 ↔ 文档映射

| 场景 | 时序图 | 测试用例 | 编排测试 | 实现清单条目 |
|---|---|---|---|---|
| S01 | `core-S01-edit-and-save-diagram.md` | `core-S01-test-cases.md` | `core-S01-diagram-save.json` | ✅ V1 |
| S02 | `core-S02-load-shared-diagram.md` | `core-S02-test-cases.md` | `core-S02-shared-link-load.json` | ✅ V1 |
| S03 | `core-S03-user-auth.md`（V2） | `core-S03-test-cases.md`（V2） | `core-S03-user-auth.json` | 🟡 规格对齐中 → 下一变更实现 |
| S04 | `core-S04-room-lifecycle.md`（V2） | `core-S04-test-cases.md`（V2） | `core-S04-room-lifecycle.json` | 🟡 规格对齐中 → 下一变更实现 |
| S05 | `core-S05-ot-collab.md`（V2） | `core-S05-test-cases.md`（V2） | `core-S05-ot-collab.json` | 🟡 规格对齐中 → 下一变更实现 |

### S06 补充映射

| 场景 | 需求 | 功能设计 | 时序图 | 契约 | 测试 | 编排 |
|---|---|---|---|---|---|---|
| S06 | `core-S06-mcp-service-requirements.md` | `core-S06-mcp-service-design.md` | `core-S06-ai-client-mcp.md` | `mcp-tools.yaml` | `core-S06-test-cases.md` | `core-S06-mcp-service.json` |

## MODIFIED — 3. 功能规格索引（redesign phases A-E 引入 + 基线扩展）

在现有 redesign A–E 与 S06 索引之上，明确追加：

| 规格文件 | 阶段 | 核心能力 |
|---|---|---|
| `core-S03-user-auth-design.md` | V2 / S03 | 登录/注册/会话续期；成功后进入 rooms；主题与主编辑器一致 |
| `core-S04-room-lifecycle-design.md` | V2 / S04 | 房间列表/创建/邀请/成员/角色；进入 room-editor |
| `core-S05-ot-collab-design.md` | V2 / S05 | WS 连接态、presence、Activity、断线排队与本地降级 |
| `core-01-editor-prototype.html` | 产品设计主基线 | S01～S05 唯一现行 HTML 主原型（只读事实输入） |

## MODIFIED — 参考源

- 统一主原型（只读）：`logos/resources/prd/2-product-design/2-page-design/core-01-editor-prototype.html`
- 历史参考原型（非验收入口）：`core-03-auth-prototype.html`、`core-04-collab-prototype.html`、`core-05-ot-collab-prototype.html`
- V2 端点：`auth.yaml` / `rooms.yaml` / `collab.yaml` + 对应 backend 路由
