## ADDED — 信息架构

> 模块：core | 提案：add-baseline-docs
> 路径：`logos/resources/prd/2-product-design/1-feature-specs/core-00-information-architecture.md`
> 对齐参考源：drawdb §2.1 顶层布局 + §4 路由 + Phase 4 4 模块前端

# 信息架构（V1）

## 1. 顶层布局（Workspace）

```
+---------------------------------------------------------------+
| EditorHeader（顶部菜单：新建/打开/导入/导出/撤销/重做/分享）   |
+--------+-------------------------------------+----------------+
|        |                                     |                |
|        |                                     |                |
|        |        EditorCanvas                  |   Editor       |
| Control|   （Table / Area / Note /            |   SidePanel    |
| Panel  |    Relationship / Canvas）           |   (右)         |
| (左)   |                                     |                |
|        |                                     |                |
+--------+-------------------------------------+----------------+
| SaveState指示 + revision 状态 + 撤销/重做栈深度                |
+---------------------------------------------------------------+
```

## 2. 路由

| 路由 | 页面 | coldrawdb V1 实现 | 备注 |
|---|---|---|---|
| `/` | LandingPage | `index.html`（Vite 单页） | drawdb 有完整 LandingPage；coldrawdb V1 简化 |
| `/editor` | Editor | `frontend-rs` WASM 入口 `lib.rs::mount_to_body` | 主页面 |
| `/templates` | Templates | **V1 不实现** | drawdb 有 6 模板 |
| `/bug-report` | BugReport | **V1 不实现** | drawdb 有 |
| `/*` | NotFound | 浏览器默认 | drawdb 有 |

> coldrawdb V1 路由层由 `trunk build` 静态托管 + 前端 WASM 处理；后端仅提供 `/api/v1/*` API。

## 3. 4 模块前端（Phase 4 架构）

```
┌─────────────────────────────────────────┐
│           frontend-rs crate             │
│  (Leptos 0.x + WASM + trunk)            │
├─────────────────────────────────────────┤
│  lib.rs                                 │
│  └── mount_to_body + 模块组合           │
├─────────────────────────────────────────┤
│  editor_data_access（无依赖）           │
│  └── HTTP 客户端（diagrams/bridge）     │
│        + debounce 1s 自动保存          │
├─────────────────────────────────────────┤
│  editor_core（依赖 data_access）         │
│  └── 状态机（diagram / undo / redo）     │
├─────────────────────────────────────────┤
│  editor_panels（依赖 core）              │
│  └── 侧栏（Tables / Areas / Notes ...）  │
├─────────────────────────────────────────┤
│  editor_render（依赖 core）              │
│  └── Canvas 渲染（Table/Field/连线）    │
└─────────────────────────────────────────┘
```

**数据流**：`editor-data-access` → `editor-core` (debounce 1s) → `editor-panels` / `editor-render`（Leptos signals 细粒度更新）

**模块边界**：
- `editor_data_access` → 唯一可发起 HTTP 请求的模块
- `editor_core` → 唯一持有 diagram 主状态的模块
- `editor_panels` / `editor_render` → 只读消费 core 的 signals

## 4. 11 子模块后端

```
backend/src/
├── main.rs            # actix-web 入口
├── init.rs            # 配置加载 + DB 初始化 + migration
├── diagrams_v1.rs     # /api/v1/diagrams/* 5 端点
├── phase3_bridge.rs   # /api/v1/bridge/* 5 端点
├── areas/             # 区域实体 + repository
├── diagrams/          # diagram 实体 + repository
├── fields/            # 字段实体 + repository
├── indices/           # 索引实体 + repository
├── notes/             # 便签实体 + repository
├── references/        # 关系实体 + repository
├── tables/            # 表实体 + repository
├── todos/             # task 实体 + repository
├── common/            # 共享类型（error / id / timestamp）
├── entity/            # ORM 实体
├── error/             # 错误类型 + IntoResponse
└── repository/        # 数据访问抽象层
```

**7 个领域实体子模块**（`areas / diagrams / fields / indices / notes / references / tables`）对应 drawdb 主分支的 7 大画布对象（CAP-CANVAS-01..07）。

**4 个支撑子模块**（`todos / common / entity / error / repository`）提供横切关注点。

## 5. 11 张数据表（V1）

| 表名 | 用途 | drawdb 等价 |
|---|---|---|
| `task` | todo（待办） | drawdb `task` 模板对象 |
| `diagram` | 主表 | ✅ 1:1 |
| `diagram_link` | diagram 关联 | drawdb `diagram_link` |
| `table` | 表 | ✅ 1:1 |
| `field` | 字段 | ✅ 1:1 |
| `table_link` | 表关联 | drawdb `table_link` |
| `indice` | 索引 | ✅ 1:1 |
| `indice_link` | 索引关联 | drawdb `indice_link` |
| `reference` | 关系 | drawdb `relationships` |
| `area` | 区域 | drawdb `subjectAreas` |
| `note` | 便签 | drawdb `notes` |

> 11 张表名（`task` 用复数时为 todos，但 init.sql 中表名是单数 `task`）逐一对齐 drawdb 主分支 + `database_design.json` 样本。详细 DDL 见 `deltas/database/coldrawdb-v1.sql`。

## 6. 路由层 ↔ 实体层映射

| API 端点 | 调用 entity |
|---|---|
| `POST /api/v1/diagrams` | `diagrams` + `tables` + `fields` + `references` + `indices` + `areas` + `notes` + `diagram_link` + `table_link` + `indice_link` |
| `GET /api/v1/diagrams/{id}` | 同上（读） |
| `PUT /api/v1/diagrams/{id}` | 同上（写，含 revision 乐观锁） |
| `DELETE /api/v1/diagrams/{id}` | 同上（级联删除） |
| `POST /api/v1/diagrams/import` | 同上（从 JSON 导入） |
| `GET /api/v1/bridge/import/local/logs` | `todos`（导入日志 = task 实体） |
| `POST /api/v1/bridge/import/local/retry/{id}` | `todos`（重试 task） |
| `GET /api/v1/bridge/config` | `entity`（配置单例） |
| `PUT /api/v1/bridge/config` | `entity`（配置单例） |
| `POST /api/v1/bridge/import/local` | `todos` + 所有画布实体（导入图） |

## 7. 对齐参考源

- drawdb §2.1 顶层布局、§4 路由
- `docs/phase4/PHASE4_DONE.md`
- `docs/phase4/architecture.mmd`
- `docs/phase4/module-mapping.md`
- `backend/src/` 实际目录
- `docs/drawdb-capability-checklist.md` §3 状态管理

## 8. V2 增量：IO 抽屉（Phase C）

> 模块：core | 提案：redesign-phase-c-import-export | 最后更新：2026-06-14

### 9.1 主体栅格（含 IO 抽屉）

```css
.cdb-main {
  display: grid;
  grid-template-columns: 48px 1fr auto auto;
  /* ToolRail | Canvas | Inspector? | IoDrawer? */
}
```

| 状态 | grid-template-columns |
|------|------------------------|
| 默认（Inspector 开） | `48px 1fr 320px 0` |
| Inspector 折叠 | `48px 1fr 0 0` |
| IO 抽屉开 | `48px 1fr 0 400px`（Inspector 强制折叠） |

### 9.2 z-index

| 层级 | Phase C 内容 |
|------|--------------|
| L3 | Inspector **或** IoDrawer（互斥，同层） |
| L4 | 模态（New / 冲突）— IO 抽屉不升级至 L4 |

### 9.3 Phase 边界更新

| 能力 | Phase C |
|------|---------|
| 导入/导出侧边抽屉 | ✅ |
| SQL/DBML 全屏视图 | ❌（Phase D） |

