## MODIFIED — 顶部元数据剥离 + 功能规格索引章节补充

> 模块：core | 提案：add-baseline-docs
> 路径：`logos/resources/prd/1-product-requirements/core-00-scenario-overview.md`
> 策略：
> 1. 移除文件开头 `## ADDED — 场景总览表` + `>` 元数据块
> 2. 在原 `## 参考源` 之前新增 §3「功能规格索引」表格（16 行，覆盖 V1 基线 9 + redesign-phase-c 1 + redesign-phase-e 6）
> 3. §1 场景索引（S01/S02/V2 计划场景）保持不变

# 场景总览

> 覆盖范围：V1（已实现事实，对应 Phase 4 完成态 + drawdb 功能对齐） + V2 / SPEC-FUTURE（待实现规格）
> 场景编号全局唯一，由 `logos-project.yaml` 的 `scenario_counter.next_id` 维护

## 场景索引

| 场景 ID | 名称 | 版本 | 状态 | 主要参与者 | 关键实体 | 关键流程 |
|---|---|---|---|---|---|---|
| S01 | 编辑并保存图表 | V1 | ✅ 已实现 | 浏览器用户 / backend / SQLite | diagram / table / field / reference / indice / area / note | 编辑画布 → debounce 1s → PUT `/api/v1/diagrams/{id}` → 409 revision 冲突检测 |
| S02 | 加载分享链接图表 | V1 | ✅ 已实现 | 浏览器用户 / bridge API | diagram JSON | 解析 share 参数 → GET `/api/v1/diagrams/{id}` 或导入本地缓存 |
| S03 | 用户注册 / 登录 / Token 续期 | V2 | ❌ 未实现 | 浏览器用户 / backend / `users` + `auth_tokens` 表 | user / auth_token | 注册 → Argon2id 哈希 → 登录签发 JWT → refresh 续期 |
| S04 | 创建/加入协作房间 | V2 | ❌ 未实现 | 房间 owner / editor / viewer / `rooms` + `room_members` 表 | room / member | 创建房间 → 邀请 → 接受 → 加入 |
| S05 | OT 实时协作（本地 op → 服务端转换 → 广播） | V2 | ❌ 未实现 | 多端客户端 / collab-server / WS 网关 | operation / operation_log | 客户端 op → WS 发送 → `transform(a, b)` → ack / rev 帧广播 |

## 场景图谱

```
                            ┌──────────────────────┐
                            │      V1（事实）       │
                            └──────────────────────┘
                                       │
              ┌────────────────────────┼────────────────────────┐
              │                        │                        │
        ┌─────▼─────┐            ┌─────▼─────┐            ┌─────▼─────┐
        │    S01    │            │    S02    │            │   (未来)  │
        │ 编辑保存  │            │ 分享加载  │            │  V1 增强  │
        └───────────┘            └───────────┘            └───────────┘
              │                        │
              └────────────┬───────────┘
                           │ (共享后端 backend + SQLite)
                           ▼
                  ┌─────────────────┐
                  │  V2 / SPEC-     │
                  │     FUTURE      │
                  └─────────────────┘
                           │
              ┌────────────┼────────────┐
              │            │            │
        ┌─────▼─────┐ ┌────▼────┐ ┌─────▼─────┐
        │    S03    │ │   S04   │ │    S05    │
        │ 用户鉴权  │ │ 房间管理│ │ OT 实时   │
        └───────────┘ └─────────┘ └───────────┘
              │            │            │
              └────────────┴────────────┘
                           │
                  (共享 collab-server + WS 网关)
```

## 场景 ↔ 文档映射

| 场景 | 时序图 | 测试用例 | 编排测试 | 实现清单条目 |
|---|---|---|---|---|
| S01 | `core-S01-edit-and-save-diagram.md` | `core-S01-test-cases.md` | `core-S01-diagram-save.json` | ✅ V1 |
| S02 | `core-S02-load-shared-diagram.md` | `core-S02-test-cases.md` | `core-S02-shared-link-load.json` | ✅ V1 |
| S03 | `core-S03-user-auth.md`（V2） | `core-S03-test-cases.md`（V2） | `core-S03-user-auth.json`（V2） | ❌ V2 |
| S04 | `core-S04-room-lifecycle.md`（V2） | `core-S04-test-cases.md`（V2） | `core-S04-room-lifecycle.json`（V2） | ❌ V2 |
| S05 | `core-S05-ot-collab.md`（V2） | `core-S05-test-cases.md`（V2） | `core-S05-ot-collab.json`（V2） | ❌ V2 |

## 3. 功能规格索引（redesign phases A-E 引入 + 基线扩展）

> 覆盖范围：16 个 `core-XX-*.md` 功能规格文件（V1 基线 9 + redesign-phase-c 1 + redesign-phase-e 6）。

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

## 参考源

- V1 场景对齐：drawdb 主分支 `https://github.com/drawdb-io/drawdb` 能力集
- V1 表/字段模型：`database_design.json`（仓库根）
- V1 端点：`backend/src/diagrams_v1.rs` + `backend/src/phase3_bridge.rs`
- V1 路由拆分：`drawdb-capability-checklist.md` §1.4 / §1.5

