# 业务场景概览（技术实现）

## ADDED — Phase 3 技术实现状态总览

> 模块：core | 提案：v1-register-scenario-docs
> 新增到 `logos/resources/prd/3-technical-plan/2-scenario-implementation/core-00-scenario-overview.md`
> 对应 `scenario-architect` SKILL §Step 5 输出规范
> 范围：V1 ✅ 场景（S01 / S02）+ V2 deferred 场景（S03 / S04 / S05）

---

# 业务场景概览（技术实现）

> 与 Phase 1 业务总览配套使用：`logos/resources/prd/1-product-requirements/core-00-scenario-overview.md`
> 本文档聚焦「技术实现状态」，按 `scenario-architect` SKILL §Step 5 输出

## 场景地图

| 编号 | 场景名称 | Phase 1 | Phase 2 | Phase 3 时序图 | API | 编排 | 状态 |
|------|---------|---------|---------|--------------|-----|------|------|
| S01  | 编辑并保存图表 | ✅ | ✅ | ✅ | ✅ | ✅ | **V1 已实现** |
| S02  | 加载分享链接图表 | ✅ | ✅ | ✅ | ✅ | ✅ | **V1 已实现** |
| S03  | 用户注册 / 登录 / Token 续期 | ✅ | 🔲 | 🔲 | 🔲 | 🔲 | **V2 deferred** |
| S04  | 创建 / 加入协作房间 | ✅ | 🔲 | 🔲 | 🔲 | 🔲 | **V2 deferred** |
| S05  | OT 实时协作 | ✅ | 🔲 | 🔲 | 🔲 | 🔲 | **V2 deferred** |

> **V1 / V2 边界声明**：
> - V1 边界（`core-01-architecture-overview.md` §9）明确不含 OT、WebSocket、用户系统
> - `core` 模块配置 `skip_phases: [api, database, scenario]`，且 `logos-project.yaml` 的 `scenarios` 字段中 S03–S05 标记为 `status: planned, version: V2`
> - 本表如实反映：S03–S05 仅完成 Phase 1 的业务场景定义，Phase 2 之后的实现路径全部 deferred 至 V2

## 场景依赖关系

```
V1 场景（无前置依赖）：
├── S01 编辑保存 ──┐
└── S02 分享加载 ──┴── 共享 backend + SQLite，无顺序约束

V2 场景（链式依赖）：
S03 鉴权 ─→ S04 房间管理 ─→ S05 OT 协作
       │              │              │
       └──────────────┴──────────────┴── 共享 collab-server + WS 网关（V2 引入）
```

- **V1 场景之间无依赖**：S01（编辑保存）和 S02（分享加载）使用同一 backend + SQLite，但 HTTP 端点独立（PUT vs GET），可独立测试
- **V2 场景存在链式依赖**：S03（鉴权）是 S04（房间）的前置，S04 是 S05（OT 协作）的前置；任一阶段未实现则后续场景无法落地
- **V1 → V2 无前置依赖**：V1 公开 share link 仍可在 V2 中作为「匿名邀请」兼容路径；具体迁移方案待 V2 提案定义

## 场景索引（V1 ✅ 技术维度）

| 场景 | 时序图 | 编排测试 | 相关功能规格 | 后端子模块 | 前端模块 |
|---|---|---|---|---|---|
| S01 | `core-S01-edit-and-save-diagram.md` | `core-S01-diagram-save.json` | `core-02-diagram-persistence.md` / `core-01a-table-and-field.md` | `diagrams_v1.rs` / `diagrams/service.rs` | `editor_data_access` / `editor_core` / `editor_panels` |
| S02 | `core-S02-load-shared-diagram.md` | `core-S02-shared-link-load.json` | `core-02-diagram-persistence.md` / `core-05-top-menu-modals.md` | `diagrams_v1.rs` / `diagrams/service.rs` | `editor_data_access` / `editor_core` / `editor_render` |

## 场景索引（V2 deferred 占位）

| 场景 | 业务定义位置 | 备注 |
|---|---|---|
| S03 | `core-04-scenario-detail.md`（仅 Phase 1 业务定义） | Phase 2+ 全部 deferred；引入时需新增 `core-S03-user-auth.md` 时序图 + 鉴权 API + 用户表迁移 |
| S04 | `core-04-scenario-detail.md`（仅 Phase 1 业务定义） | 同上；依赖 S03 完成 |
| S05 | `core-04-scenario-detail.md`（仅 Phase 1 业务定义） | 同上；依赖 S04 完成，需独立 collab-server + WS 网关 |

## 与 Phase 1 业务总览的关系

| 维度 | Phase 1 业务总览 | Phase 3 技术总览（本文件） |
|---|---|---|
| 路径 | `prd/1-product-requirements/core-00-scenario-overview.md` | `prd/3-technical-plan/2-scenario-implementation/core-00-scenario-overview.md` |
| 视角 | 用户视角（What）：场景描述、参与者、业务价值、验收 G/W/T | 技术视角（How）：时序图、API、编排、后端 / 前端模块映射 |
| 受众 | PM / 设计 / 测试 | 后端 / 前端 / DevOps |
| V2 场景 | 列出但不深入 | 明确标注 deferred，与 `core.skip_phases` 对齐 |

**使用建议**：
- 编写新功能时：从 Phase 1 总览读取业务背景 → 从本文件读取技术实现约束
- 评估 V2 范围时：以 Phase 1 总览的「场景 ↔ 文档映射」表为索引 → 在本文件确认 deferred 状态
- AI Agent 检索时：先查 Phase 1 确认「这个场景存在吗」，再查本文件确认「这个场景的时序图 / API / 编排在哪里」

## 参考源

- `logos/resources/prd/1-product-requirements/core-00-scenario-overview.md` —— Phase 1 业务总览
- `logos/resources/prd/1-product-requirements/core-04-scenario-detail.md` —— S01 / S02 GIVEN/WHEN/THEN 详述
- `logos/resources/prd/3-technical-plan/1-architecture/core-01-architecture-overview.md` —— V1 边界 §9 + 模块拆分
- `logos/resources/prd/3-technical-plan/2-scenario-implementation/core-S01-edit-and-save-diagram.md` —— S01 时序图
- `logos/resources/prd/3-technical-plan/2-scenario-implementation/core-S02-load-shared-diagram.md` —— S02 时序图
- `logos/resources/scenario/core-S01-diagram-save.json` —— S01 E2E 编排
- `logos/resources/scenario/core-S02-shared-link-load.json` —— S02 E2E 编排
- `logos/skills/scenario-architect/SKILL.md` §Step 5 —— 本文件的输出规范来源
- `logos/logos-project.yaml` —— `scenarios` 字段 + `core.skip_phases` 配置