# V1 技术架构（How 层 — 第 1 步：架构）

## 1. 系统上下文

```
┌──────────────────────────────────────────────────────┐
│                       Browser                        │
│  ┌──────────────────────────────────────────────┐   │
│  │       frontend-rs (Leptos + WASM)            │   │
│  │  index.html + trunk bundle                   │   │
│  └──────────────────────────────────────────────┘   │
│                       │ HTTP / JSON                  │
└───────────────────────┼──────────────────────────────┘
                        │
┌───────────────────────┼──────────────────────────────┐
│                       ▼                              │
│  ┌──────────────────────────────────────────────┐   │
│  │   backend (Rust + actix-web 4)               │   │
│  │   /api/v1/diagrams/*  (5 端点)               │   │
│  │   /api/v1/bridge/*    (5 端点)               │   │
│  └──────────────────────────────────────────────┘   │
│                       │ SQL                          │
│                       ▼                              │
│  ┌──────────────────────────────────────────────┐   │
│  │   SQLite (WAL 模式)                          │   │
│  │   11 张表                                    │   │
│  └──────────────────────────────────────────────┘   │
└──────────────────────────────────────────────────────┘
```

V1 系统由**两个 Rust 二进制**组成：WASM 前端 + actix-web 后端；共享 SQLite 文件。V2 计划新增 `collab-server`（独立 OT 引擎，详见 `add-v2-collab-spec`）。

## 2. 前端 4 模块 + 单向依赖

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

### 2.1 模块职责

| 模块 | 职责 | 关键 API |
|---|---|---|
| `editor_data_access` | 唯一 HTTP 出口；封装 diagrams/bridge API；debounce 自动保存 | `DiagramsApi` / `BridgeApi` |
| `editor_core` | 持有 diagram 主状态；状态变更入口；撤销/重做栈 | `Diagram` / `UndoRedoContext` |
| `editor_panels` | 侧栏 6 Tab + Issues + DBMLEditor 视图 | 7 个 panel 子模块 |
| `editor_render` | 画布渲染（Table/Field/Relationship/Area/Note）；贝塞尔连线 | `Canvas` / `calc_path` |

### 2.2 依赖方向（强制）

```
editor_data_access  ←（无依赖）
        ↑
editor_core
        ↑
   ┌────┴────┐
   │         │
editor_panels  editor_render
```

- 任何反向依赖（panels → data_access，render → data_access）**禁止**
- 模块间通信通过 `editor_core` 暴露的 signals（Leptos `Signal<T>` / `RwSignal<T>`）
- HTTP 请求必须经过 `editor_data_access`，其他模块不可直接 `fetch`

## 2.5 V2 前端布局层（redesign-phase-a/b/c 落地，2026-06-14）

V2 重构后的前端分为 **4 个 UI 层**（z-index 体系见 `core-00-information-architecture.md` §1）：

| 层 | 容器 | z-index | 职责 |
|---|---|---|---|
| L1 | `AppBar`（顶栏） | `--cdb-z-app-bar` (10) | 标题 + 主菜单 + 全局操作（Share / Undo / Redo） |
| L2 | `ToolRail`（左侧工具轨） | `--cdb-z-tool-rail` (20) | 选中 / 表 / 关系 / 区域 / 便签 / 缩放 6 个工具按钮 |
| L3 | `Inspector`（右侧检查器） | `--cdb-z-inspector` (30) | 当前选中对象的属性编辑面板（替代 V1 模态中的字段编辑） |
| L4 | `ModalRoot`（模态根） | `--cdb-z-modal` (40) | New / Open / Share / Rename / Settings / Confirm 6 个模态 |
| L5 | `Palette` / `Tooltip` / `Popover` | `--cdb-z-overlay` (50) | 颜色选择器 / 工具提示 / 弹出层 |
| L6 | `Drawer`（IO 抽屉） | `--cdb-z-drawer` (35) | 导入 / 导出抽屉（与 Inspector 同级侧栏语义，不占用模态层） |

> V1 vs V2 关键差异：V1 所有编辑操作集中在中央模态（L4），V2 拆分为侧栏（L3）+ 抽屉（L6）+ 模态（L4），降低上下文切换成本。

## 2.6 设计系统层（redesign-phase-d/e 落地，2026-06-15）

| 组件 | 来源 | 引用规格 |
|---|---|---|
| Design Tokens | 13 类约 100 个 `--cdb-*` CSS 变量 | `core-07-design-tokens.md` |
| Icon Library | 自建 SVG 模板 + `@douyinfe/semi-icons` 命名规范 | `core-08-icon-library.md` |
| Core Components | 8 类（Button / Modal / Dropdown / Tooltip / Popover / Tag / Collapse / SideSheet） | `core-09-core-components.md` |
| Code Editor | Monaco + DBML setup + 复制按钮（E4 替代 V1 `<textarea readonly>`） | `core-0a-code-editor.md` |
| Dark Mode | `<html data-mode="light\|dark">` 全局切换（E5） | `core-0b-dark-mode.md` |
| Motion | CSS `@keyframes` + transition + 工具类（E6 不引入 framer-motion） | `core-0c-motion.md` |

> **依赖方向（强制）**：所有组件 / icon / 动效 都依赖 token 层（`core-07`），不得越过 token 直接引用硬编码值。

## 3. 后端 11 子模块

```
backend/src/
├── main.rs            # actix-web 入口（绑定 /api/v1/*）
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

### 3.1 7 个领域子模块（与 drawdb 7 大画布对象对齐）

| 子模块 | 对应画布对象 | 11 张表中对应表 |
|---|---|---|
| `areas` | Area | `area` |
| `diagrams` | Diagram | `diagram` |
| `fields` | Field | `field` |
| `indices` | Index | `indice` + `indice_link`（V1 实体化但 frontend 不写） |
| `notes` | Note | `note` |
| `references` | Relationship | `reference` |
| `tables` | Table | `table` + `table_link` |

### 3.2 4 个支撑子模块

| 子模块 | 职责 |
|---|---|
| `todos` | 导入任务（task 表） |
| `common` | 共享类型（`DbId` / `Timestamp` / `Revision`） |
| `entity` | SeaORM 实体定义（与 SQL 表 1:1） |
| `error` | 错误类型 + `IntoResponse` 实现 |
| `repository` | 通用数据访问抽象（trait + SQL 实现） |

## 4. 11 张表

| 表名 | 用途 | 主键 | 关键索引 |
|---|---|---|---|
| `task` | 导入任务日志 | BIGINT auto | type / status |
| `diagram` | 主表 | BIGINT auto | id（UUID） + revision |
| `diagram_link` | diagram 关联 | BIGINT auto | source_id / target_id |
| `table` | 表元数据 | BIGINT auto | diagram_id + name |
| `field` | 字段 | BIGINT auto | table_id + name |
| `table_link` | 表关联 | BIGINT auto | source / target |
| `indice` | 索引 | BIGINT auto | table_id |
| `indice_link` | 索引字段关联 | BIGINT auto | indice_id + field_id |
| `reference` | 关系 | BIGINT auto | start_table_id / end_table_id |
| `area` | 区域 | BIGINT auto | diagram_id |
| `note` | 便签 | BIGINT auto | diagram_id |

详细 DDL 见 `deltas/database/coldrawdb-v1.sql`。

## 5. 数据流

### 5.1 加载流程（GET）

```
Browser → frontend-rs/editor_data_access
              ↓ HTTP GET /api/v1/diagrams/{id}
         backend/diagrams_v1::read
              ↓ SELECT * FROM diagram JOIN ...
         SQLite
              ↑ JSON response
         editor_core::set_diagram
              ↑ signal
         editor_panels + editor_render (reactive update)
```

### 5.2 编辑保存流程（debounce 1s PUT）

```
User edits → editor_panels.on_change
              ↓ signal
         editor_core::push_undo + mark_dirty
              ↓ 1000ms debounce
         editor_data_access.save
              ↓ HTTP PUT /api/v1/diagrams/{id}
         backend/diagrams_v1::update
              ↓ BEGIN IMMEDIATE TRANSACTION
         SQLite (write)
              ↓ COMMIT
              ↑ 200 OK + new revision
         editor_core::update_revision
```

### 5.3 冲突流程（PUT → 409）

```
editor_data_access.save → 409 Conflict
              ↓
         editor_core::conflict_detected
              ↓
         弹冲突对话框（用户决策）
         - Reload: GET /api/v1/diagrams/{id} → 覆盖本地
         - Force: PUT /api/v1/diagrams/{id} with rev+1 → 覆盖远端
         - Cancel: 保留本地 + 远端两份
```

## 6. 关键技术选型

| 维度 | V1 选型 | 备注 |
|---|---|---|
| 前端语言 | Rust + Leptos 0.x | 与 drawdb JS+React 对照 |
| 前端打包 | trunk | WASM 静态资源 |
| 渲染 | HTML5 Canvas（自绘） | 性能优于 SVG（100+ 表 60fps） |
| 后端语言 | Rust + actix-web 4 | 高性能 + 类型安全 |
| 数据库 | SQLite（WAL 模式） | 单进程 + 文件级备份 |
| ORM | SeaORM | 11 张表实体化 |
| 序列化 | serde + JSON | 与 OpenAPI 一致 |
| 配置 | TOML | `backend/config.toml` |
| 日志 | tracing + tracing-subscriber | 结构化日志 |
| 测试 | cargo test（unit + integration） | 覆盖 11 个子模块 |

## 7. 部署拓扑

### 7.1 本地 dev

```
trunk serve        # localhost:8080（WASM 前端）
cargo run -p backend   # localhost:3000（actix-web）
SQLite             # ./data/coldrawdb.db
```

### 7.2 Docker

```dockerfile
# frontend-rs 阶段
FROM rust:1.75 as wasm-build
WORKDIR /app
RUN cargo install trunk
RUN trunk build --release

# backend 阶段
FROM rust:1.75-slim as api-build
WORKDIR /app
RUN cargo build --release -p backend

# 运行阶段
FROM debian:bookworm-slim
COPY --from=api-build /app/target/release/backend /usr/local/bin/
COPY --from=wasm-build /app/dist /var/www/
EXPOSE 3000
CMD ["backend"]
```

### 7.3 staging

- 单机 Docker Compose
- 反向代理：nginx（前端静态 + 后端 API 反代）
- 数据卷：SQLite 文件
- 详细见 `deltas/.../3-deployment/core-01-deployment-plan.md`

## 8. 性能预算

| 指标 | 预算 | 测量 |
|---|---|---|
| 首屏加载 | < 3s | Phase 4 W4 perf 记录 |
| 编辑响应 | < 50ms | Leptos signal propagation |
| 自动保存 PUT | < 200ms | 100 KB payload |
| Canvas 渲染 | 60 fps / 100 表 / 200 关系 | Phase 4 W4 perf |
| WASM 体积 | < 5 MB | trunk build 产物 |
| 后端冷启动 | < 500ms | `cargo run` to ready |

## 9. V1 边界

- ❌ 微服务拆分（V1 单体 backend）
- ❌ PostgreSQL 适配（V1 仅 SQLite）
- ❌ CDN 静态资源（V1 trunk 直出）
- ❌ OT 实时协作（V2 计划）
- ❌ WebSocket（V1 全 HTTP）
- ❌ 水平扩展（V1 单实例）

## 10. 对齐参考源

- `docs/phase4/PHASE4_DONE.md`
- `docs/phase4/architecture.mmd`
- `docs/phase4/module-mapping.md`
- `RUST_WEB_REFACTOR_PLAN.md`（仓库根）
- `backend/Cargo.toml` + `frontend-rs/Cargo.toml`
- `database_design.json`（字段命名对账）
- `docs/drawdb-capability-checklist.md` §4

