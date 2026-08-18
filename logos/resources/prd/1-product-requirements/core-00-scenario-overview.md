# 场景总览

> 覆盖范围：V1（S01/S02 全栈已实现）+ V2（S03～S05 后端已实现、生产前端待接入）+ V3（S06 MCP 实现中）
> 场景编号全局唯一，由 `logos-project.yaml` 的 `scenario_counter.next_id` 维护

## 场景索引

| 场景 ID | 名称 | 版本 | 真实状态 | 关键流程 |
|---|---|---|---|---|
| S01 | 编辑并保存图表 | V1 | ✅ 前后端已实现 | 编辑 → debounce → PUT → revision |
| S02 | 加载分享链接图表 | V1 | ✅ 前后端已实现 | share 参数 → GET → 只读加载 |
| S03 | 用户注册/登录/Token 续期 | V2 | 🟡 后端已实现，生产前端待接入 | register/login/refresh/logout/me |
| S04 | 创建/加入协作房间 | V2 | 🟡 后端已实现，生产前端待接入 | room/invite/member CRUD |
| S05 | OT 实时协作 | V2 | 🟡 后端已实现，生产前端待接入 | WS connect/op/ack/sync/presence |
| S06 | AI 客户端通过 MCP 管理数据库图表 | V3 | 🔵 规格变更中 | MCP initialize → tools/list → diagram CRUD/import/export |

## 场景图谱

```text
S01/S02 Web 编辑与分享 ───────────────┐
                                      ├─→ diagram/bridge API → SQLite
S03 鉴权 → S04 房间 → S05 OT ───────┤
                                      │
S06 AI 客户端 → MCP stdio adapter ───┘
```

S06 复用 diagram/bridge API，不直接依赖 SQLite，也不依赖 S03～S05 生产前端。当前 diagram API 尚未强制 S03 鉴权，因此 S06 MVP 仅限本地可信 stdio。

## 场景 ↔ 文档映射

| 场景 | 时序图 | 测试用例 | 编排测试 | 实现清单条目 |
|---|---|---|---|---|
| S01 | `core-S01-edit-and-save-diagram.md` | `core-S01-test-cases.md` | `core-S01-diagram-save.json` | ✅ V1 |
| S02 | `core-S02-load-shared-diagram.md` | `core-S02-test-cases.md` | `core-S02-shared-link-load.json` | ✅ V1 |
| S03 | `core-S03-user-auth.md`（V2） | `core-S03-test-cases.md`（V2） | `core-S03-user-auth.json`（待补） | 🟡 后端已实现，生产前端待接入 |
| S04 | `core-S04-room-lifecycle.md`（V2） | `core-S04-test-cases.md`（V2） | `core-S04-room-lifecycle.json`（V2） | 🟡 后端已实现，生产前端待接入 |
| S05 | `core-S05-ot-collab.md`（V2） | `core-S05-test-cases.md`（V2） | `core-S05-ot-collab.json`（V2） | 🟡 后端已实现，生产前端待接入 |

### S06 补充映射

| 场景 | 需求 | 功能设计 | 时序图 | 契约 | 测试 | 编排 |
|---|---|---|---|---|---|---|
| S06 | `core-S06-mcp-service-requirements.md` | `core-S06-mcp-service-design.md` | `core-S06-ai-client-mcp.md` | `mcp-tools.yaml` | `core-S06-test-cases.md` | `core-S06-mcp-service.json` |
## 3. 功能规格索引（redesign phases A-E 引入 + 基线扩展）

> 覆盖范围：既有功能规格 + S03～S05 场景设计 + S06 MCP 功能设计。

| 规格文件 | 阶段 | 核心能力 |
|---|---|---|
| `core-00-information-architecture.md` | V1 基线 | 顶层布局（Workspace + Modal 层级）、路由拆分 |
| `core-01-editor-canvas.md` | V1 基线 | 编辑器画布总规格、平移/缩放/框选 |
| `core-01a-table-and-field.md` | V1 基线 | 表与字段编辑（CAP-CANVAS-01/02） |
| `core-01b-relationship.md` | V1 基线 | 关系编辑（CAP-CANVAS-03） |
| `core-01c-index-enum-custom-type.md` | V1 基线 | 索引 / 枚举 / 自定义类型 |
| `core-01d-import-export.md` | redesign-phase-c | 导入 / 导出 IO 抽屉（替换 V1 模态） |
| `core-02-diagram-persistence.md` | V1 基线 | diagram CRUD + revision 乐观锁 |
| `core-03-bridge-io.md` | V1 基线 | 桥接层 7 引擎 SQL + DBML + JSON |
| `core-04-side-panel-tabs.md` | V1 基线（V2 重构） | 侧栏 7 Tab + 搜索 / 筛选 |
| `core-05-top-menu-modals.md` | V1 基线（V2 重构） | AppBar + 6 模态（New/Open/Share/Rename/Settings/Confirm） |
| `core-07-design-tokens.md` | redesign-phase-e（E1） | `--cdb-*` 设计 token 体系（13 类 ~100 token） |
| `core-08-icon-library.md` | redesign-phase-e（E2） | SVG 图标库（替代 emoji） |
| `core-09-core-components.md` | redesign-phase-e（E3） | 8 类核心组件（Button / Modal / Dropdown / Tooltip / Popover / Tag / Collapse / SideSheet） |
| `core-0a-code-editor.md` | redesign-phase-e（E4） | Monaco 集成 + DBML setup + 复制按钮 |
| `core-0b-dark-mode.md` | redesign-phase-e（E5） | 暗色模式（`darkBgTheme = #16161A`） |
| `core-0c-motion.md` | redesign-phase-e（E6） | 动效 token + 过渡 / 微交互 |
| `core-S06-mcp-service-design.md` | V3 / S06 | MCP stdio、七工具、错误体验与四客户端配置 |

## 参考源

- V1 场景对齐：drawdb 主分支 `https://github.com/drawdb-io/drawdb` 能力集
- V1 表/字段模型：`database_design.json`（仓库根）
- V1 端点：`backend/src/diagrams_v1.rs` + `backend/src/phase3_bridge.rs`
- V1 路由拆分：`drawdb-capability-checklist.md` §1.4 / §1.5
