## ADDED — 场景总览表

> 模块：core | 提案：add-baseline-docs
> 路径：`logos/resources/prd/1-product-requirements/core-00-scenario-overview.md`

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

## 参考源

- V1 场景对齐：drawdb 主分支 `https://github.com/drawdb-io/drawdb` 能力集
- V1 表/字段模型：`database_design.json`（仓库根）
- V1 端点：`backend/src/diagrams_v1.rs` + `backend/src/phase3_bridge.rs`
- V1 路由拆分：`drawdb-capability-checklist.md` §1.4 / §1.5
