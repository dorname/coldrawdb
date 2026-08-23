## 1. 范围

本文件追踪 S01～S06 的真实实现状态；“静态原型可演示”与“生产前端已接入”分别记录。

**V1 行** = 已在 Phase 1-4 完成的事实代码  
**V2 行** = S03～S05 后端已实现，生产前端 A/B/C 批次已接入，D 批全链路回归已通过 `openlogos verify` 收口
**V3 行** = S06 MCP 规格与代码已完成，并已通过 `openlogos verify` 验收

> 本清单仅作总览与状态标记。详细规格见各 Phase 2 设计文档与 Phase 3 时序图。

## 2. 前端 4 模块

### 2.1 editor_data_access

- [x] HTTP 客户端（`gloo_net::http::Request`）
- [x] diagrams API 封装（fetch_diagram / save / create / delete / import）
- [x] bridge API 封装（import / logs / retry / config）
- [x] debounce 1s 自动保存循环
- [x] 错误处理 + 指数退避重试
- [x] SaveState 状态机
- [x] S03 auth API 客户端与 refresh/logout 接入（批次 A）
- [x] S04 room/invite/member API 客户端接入（批次 B）
- [x] S05 collab REST head/ops、WS URL 与 frame 解析契约接入（批次 C）

### 2.2 editor_core

- [x] Diagram 状态机（RwSignal<Diagram>）
- [x] 字段级 UndoRedoContext
- [x] dirty 标记
- [x] revision 跟踪
- [x] set_diagram / push_undo / undo / redo
- [x] S05 最小 OT 操作队列、ack/serverRev、断线排队与 sync 状态（批次 C）

### 2.3 editor_panels

- [x] Tables Tab + 列表项 + 增删改
- [x] Areas Tab
- [x] Enums Tab（V1 仅前端 state）
- [x] Notes Tab
- [x] Relationships Tab
- [x] Types Tab（V1 仅前端 state）
- [x] Issues Tab + 校验引擎
- [x] AppBar（SaveState / revision / Share / Undo / Redo）
- [x] ToolRail（6 工具按钮）
- [x] Inspector（7 Tab + 字段编辑）
- [x] ModalRoot（New / Open / Share / Rename / Settings / Confirm / Conflict）
- [x] IO Drawer（导入 / 导出，redesign-phase-c）
- [x] 全局搜索 + 类型筛选
- [x] `?share=` URL 解析 + 冷启动 GET 加载（`lib.rs` + `AppRoot`）
- [x] Command Palette 交互（Ctrl+K / 搜索 / Enter 选中，`command_palette.rs`）
- [x] Code View 交互（SQL/DBML/JSON + 复制，`code_view.rs` + AppBar 按钮）
- [x] 保存失败指数退避（3s/6s/12s，`save_with_retry`）
- [x] S03 登录/注册/会话界面、user-menu、session-indicator 生产接入（批次 A）
- [x] S04 房间列表、创建房间、邀请、成员、角色和 viewer 只读生产接入（批次 B）
- [x] S05 ws-status、room-presence、ot-rev、reconnect-banner、activity-feed 生产接入（批次 C）
- [ ] Monaco wasm 完整挂载（可选升级，不阻塞本变更）

### 2.4 editor_render

- [x] Canvas 容器 + 平移 + 缩放 + 框选
- [x] Table 渲染
- [x] Field 行渲染
- [x] Relationship 贝塞尔连线
- [x] Area 矩形 + 标签
- [x] Note 富文本
- [x] 选中 / 高亮 / 闪烁
- [x] 撤销栈深度指示
- [x] S05 协作者光标、presence 标签渲染 DTO 与稳定位置分配（批次 C）

### 2.5 设计系统（redesign-phase-e）

- [x] Design Tokens（`styles.css` ~100 个 `--cdb-*`）
- [x] Icon Library（`icons.rs` + SVG 模板）
- [x] Core Components（Button / Modal / Dropdown / Tooltip / Popover / Tag / Collapse / SideSheet）
- [x] Dark Mode（`<html data-mode="light|dark">` + `prefers-color-scheme`）
- [x] Motion（CSS transition + `prefers-reduced-motion` 降级）

## 3. 后端 11 子模块

### 3.1 7 领域子模块

#### areas
- [x] Area 实体（SeaORM）
- [x] AreaRepo CRUD
- [x] cascade delete
- [x] 单元测试 + 集成测试

#### diagrams
- [x] Diagram 实体
- [x] DiagramRepo CRUD
- [x] revision 乐观锁 + 409 冲突
- [x] 关联查询（tables / fields / references / areas / notes）
- [x] 单元测试 + 集成测试
- [ ] 版本历史（V2）

#### fields
- [x] Field 实体
- [x] FieldRepo CRUD
- [x] sort_order 排序
- [x] 单元测试

#### indices
- [x] Indice 实体（V1 实体化但 frontend 不写）
- [x] IndiceLink 实体
- [x] 单元测试
- [ ] 接收 frontend 写入（V2）

#### notes
- [x] Note 实体
- [x] NoteRepo CRUD
- [x] 单元测试

#### references
- [x] Reference 实体
- [x] ReferenceRepo CRUD
- [x] cardinality 枚举校验
- [x] on_update / on_delete 枚举校验
- [x] 单元测试

#### tables
- [x] Table 实体
- [x] TableRepo CRUD
- [x] TableLink 多对多关联
- [x] 单元测试

### 3.2 4 支撑子模块

#### todos
- [x] Task 实体
- [x] TaskRepo CRUD
- [x] 导入任务状态机
- [x] 单元测试

#### common
- [x] DbId 类型
- [x] Timestamp 类型
- [x] Revision 类型
- [x] 共享错误

#### entity
- [x] SeaORM 11 张表实体
- [x] 实体关系映射

#### error
- [x] AppError 枚举
- [x] IntoResponse 实现
- [x] 400 / 404 / 409 语义

#### repository
- [x] 通用 repository trait
- [x] SQL 实现

### 3.3 S03～S05 后端增量

- [x] S03 `backend/src/auth/` + `auth_v1.rs` + auth migration + 测试
- [x] S04 `backend/src/rooms/` + `rooms_v1.rs` + rooms migration + `core-S04-room-lifecycle.json`
- [x] S05 `backend/src/collab/` + `collab_v1.rs` + `/ws/rooms/{room_id}` + collab migration + `core-S05-ot-collab.json`
- [x] S03 编排文件 `core-S03-user-auth.json`

## 4. API 端点

### 4.1 diagrams（5 端点）

- [x] POST /api/v1/diagrams
- [x] GET /api/v1/diagrams/{id}
- [x] PUT /api/v1/diagrams/{id}
- [x] DELETE /api/v1/diagrams/{id}
- [x] POST /api/v1/diagrams/import

### 4.2 bridge（5 端点）

- [x] POST /api/v1/bridge/import/local
- [x] GET /api/v1/bridge/import/local/logs
- [x] POST /api/v1/bridge/import/local/retry/{id}
- [x] GET /api/v1/bridge/config
- [x] PUT /api/v1/bridge/config

### 4.3 S03～S05 生产后端路由

- [x] auth：5 个 REST 端点
- [x] rooms：11 个 REST 端点
- [x] collab：2 个 REST 端点 + 1 个 WebSocket 入口
- [x] 遗留 `/diagrams/*` 路由单列，不混入 v1 端点统计

### 4.4 S06 MCP（不计入 HTTP 端点数）

- [x] 1 个独立 `coldrawdb-mcp` stdio 服务
- [x] 7 个 tools：list/get/create/update/delete/import/export

## 5. 数据库（11 张表）

- [x] task
- [x] diagram
- [x] diagram_link
- [x] table
- [x] field
- [x] table_link
- [x] indice
- [x] indice_link
- [x] reference
- [x] area
- [x] note
- [x] init.sql 脚本

### 5.1 V2 增量表

- [x] auth migration / `coldrawdb-v2-auth.sql`
- [x] rooms migration / `coldrawdb-v2-rooms.sql`
- [x] collab migration / `coldrawdb-v2-collab.sql`

## 6. 桥接（7 引擎 SQL）

- [x] MySQL 导出 + 导入
- [x] PostgreSQL 导出 + 导入
- [x] SQLite 导出 + 导入
- [x] MariaDB 导出 + 导入
- [x] MSSQL 导出 + 导入
- [x] OracleSQL 导出 + 导入
- [x] Generic 导出
- [x] DBML 导出
- [x] JSON 导入（drawdb 兼容）
- [ ] Mermaid 导出（V1 未实现）
- [ ] PNG / PDF 导出（V1 未实现）

## 7. 测试

### 7.1 单元测试

- [x] backend/src/diagrams_v1.rs（5 端点）
- [x] backend/src/diagrams/service.rs（事务）
- [x] backend/src/fields/ 实体
- [x] backend/src/references/ 实体
- [x] backend/src/areas/ 实体
- [x] backend/src/notes/ 实体
- [x] backend/src/indices/ 实体
- [x] backend/src/tables/ 实体
- [x] backend/src/todos/ 实体
- [x] backend/src/phase3_bridge.rs（5 端点）
- [x] frontend-rs/src/editor_core.rs
- [x] frontend-rs/src/editor_data_access.rs
- [x] frontend-rs/src/editor_panels.rs
- [x] frontend-rs/src/editor_render.rs

### 7.2 集成 / 场景测试

- [x] S01: 编辑保存（Rust integration + wasm-pack headless）
- [x] S02: 分享链接加载
- [x] SMOKE: staging 5 项

### 7.3 编排测试

- [x] S01: 7 步骤 JSON
- [x] S02: 7 步骤 JSON
- [x] S03: register → login → me → refresh → logout → refresh 失效 JSON
- [x] S04: 房间生命周期 JSON
- [x] S05: HTTP + WebSocket OT 协作 JSON
- [x] S06: MCP stdio JSON + Rust 协议/HTTP mock 编排测试

### 7.4 统一原型与 MCP

- [x] ST-PU-01～ST-PU-19 Playwright 自动回归 + OpenLogos reporter
- [x] UT-MCP-01～UT-MCP-15
- [x] ST-MCP-01～ST-MCP-09 + OpenLogos reporter

### 7.5 V2 生产前端接入

- [x] 批次 A：S03 鉴权生产接入，覆盖 `UT-S03-01`～`UT-S03-07`、`ST-S03-01`、`UT-FE-S03-01`～`UT-FE-S03-05`；浏览器联调 `ST-FE-S03-01`～`ST-FE-S03-05` 已由 reporter 标记为 e2e harness 待接入
- [x] 批次 B：S04 房间与邀请生产接入，覆盖 `UT-S04-01`～`UT-S04-10`、`ST-S04-01`、`UT-FE-S04-01`～`UT-FE-S04-06`；浏览器联调 `ST-FE-S04-01`～`ST-FE-S04-06` 已由 reporter 标记为 e2e harness 待接入
- [x] 批次 C：S05 WS/OT/presence 生产接入，覆盖 `UT-C-01`～`UT-C-05`、`ST-C-01`、`UT-FE-S05-01`～`UT-FE-S05-06`；浏览器联调 `ST-FE-S05-01`～`ST-FE-S05-06` 已由 reporter 标记为 e2e harness 待接入
- [x] 批次 D：全链路回归与状态收口，覆盖 `ST-FE-V2-01`～`ST-FE-V2-04`、S01/S02/PU 回归和 OpenLogos reporter 聚合，`openlogos verify` 结果 PASS

### 7.6 生产前端继续对齐主原型（align-frontend-to-prototype）

- [x] 批次 A：Auth 与 Invite 页面流对齐，覆盖 `UT-FE-PROTO-01`、`UT-FE-PROTO-02`、`ST-FE-PROTO-01`、`ST-FE-PROTO-02`。
- [x] 批次 B：Rooms 列表页对齐，覆盖 `UT-FE-PROTO-03`、`UT-FE-PROTO-04`、`ST-FE-PROTO-03`、`ST-FE-PROTO-04`。
- [x] 批次 C：Collab Editor 可见状态与响应式对齐，覆盖 `UT-FE-PROTO-05`、`UT-FE-PROTO-06`、`ST-FE-PROTO-05`、`ST-FE-PROTO-06`、`ST-FE-PROTO-07`。
- [x] 批次 D：全链路回归与状态收口，覆盖 `ST-FE-PROTO-08`、`ST-FE-V2-01`～`ST-FE-V2-04` 回归、ST-PU 回归。

## 8. 部署

- [x] 本地 dev 双进程（trunk serve + cargo run）
- [x] Docker 多阶段构建
- [x] Staging docker-compose
- [x] nginx 反代
- [x] 数据备份 cron
- [x] JSON 日志 + logrotate
- [ ] Kubernetes 部署（V1 未实现）
- [ ] 生产 TLS（V1 未实现）
- [ ] Prometheus 指标（V1 未实现）

## 9. 文档

### 9.1 V1 文档（已完成）

- [x] 需求层 5 文件（含场景总览 + S01/S02 详述）
- [x] 设计层功能规格 + 1 个现行主原型 + 3 个历史参考原型 + 共享 CSS
- [x] 技术方案层 8 文件（架构 + 场景 S01–S05 + 部署）
- [x] API 2 文件（diagrams.yaml + bridge.yaml）
- [x] DB 1 文件（coldrawdb-v1.sql）
- [x] 测试 3 文件（S01 + S02 + smoke）
- [x] 场景 2 文件（S01 + S02 JSON）
- [x] 实现清单 1 文件（本文件）

### 9.2 V2 文档（后端已实现，生产前端 A/B/C 已接入）

- [x] 设计层 3 场景设计（S03/S04/S05）；统一主原型为 `core-01-editor-prototype.html`
- [x] 技术方案层 3 场景时序（S03/S04/S05）
- [x] API 3 文件（auth.yaml + rooms.yaml + collab.yaml）
- [x] DB 3 文件（v2-auth / v2-rooms / v2-collab SQL）
- [x] S03 编排测试 `core-S03-user-auth.json`
- [x] 场景 3 编排 JSON（S03 + S04 + S05）
- [x] V2 后端实现（auth / rooms / collab REST、DB、WS 与测试）
- [x] V2 生产前端 API 接入（auth / rooms / collab REST + OT 状态 + presence，按 A/B/C 批次交付）
- [x] V2 生产前端体验对齐统一主原型页面流（auth → rooms → editor、invite 独立页、协作状态可见性，`align-frontend-to-prototype` 收口）

### 9.3 S06 MCP 文档与实现

- [x] S06 需求、设计、时序、工具契约、测试与编排规格
- [x] 独立 Rust `coldrawdb-mcp` stdio 服务
- [x] initialize / tools/list / instructions
- [x] 读工具：list/get/export
- [x] 写工具：create/update/delete/import
- [x] revision、错误映射、日志脱敏
- [x] Claude/Codex/Cursor/OpenCode 配置

## 10. 关键指标

| 指标 | V1 实际 | V2 计划 |
|---|---|---|
| 前端模块 | 4 | 4 + WS client |
| 后端模块 | 11 + 5 routing | 11 + 5 routing + collab-server |
| 数据表 | 11 | 17（+ users / auth_tokens / rooms / room_members / operations / operation_log） |
| API 端点 | diagram v1 5 + bridge 5 | auth 5 + rooms 11 + collab REST 2 + WS 1；遗留 `/diagrams/*` 单列 |
| MCP | 无 | 1 个 stdio 服务、7 个 tools（不计入 HTTP 端点） |
| 引擎支持 | 7 SQL + DBML + JSON | + Room 协议 |
| 实时协作 | ❌ | ✅ OT |
| 用户系统 | ❌ | ✅ 注册 / 登录 / Token |

## 11. V1 → V2 演进路径

| V1 资产 | V2 演进 |
|---|---|
| `diagrams.yaml` | 保持兼容；新增 `users.yaml` / `auth.yaml` / `rooms.yaml` / `collab.yaml` |
| 11 张表 | 在 V1 基础上新增 6 张表 |
| 5 端点 diagrams | 保持兼容；权限校验从无 → 有 |
| 5 端点 bridge | 保持兼容；增加 multi-user 来源追踪 |
| 5 modules frontend | 在 `editor_data_access` 加 WS 客户端；`editor_core` 加 OT 队列；`editor_panels` 加房间 Tab |
| 7 引擎 SQL 导出 | 保持不变 |

## 12. 对齐参考源

- 批次 1 全部 25 个 delta 文件
- `RUST_WEB_REFACTOR_PLAN.md`
- `docs/phase4/PHASE4_DONE.md`
- `docs/drawdb-capability-checklist.md`
- `backend/Cargo.toml` + `frontend-rs/Cargo.toml`

## 13. 统一原型规格收口状态

本清单对统一原型相关能力采用**三列状态**，禁止把未实现或未逐项验证的项目标记为完成。

| 列 | 含义 |
|---|---|
| 已有能力 | 仓库中已存在且可运行的事实能力（可为部分接入） |
| 规格已收口 | 上一变更 `align-all-docs-to-unified-prototype` 已对齐文档/测试合同 |
| 本提案实现 | `implement-unified-prototype-spec-parity` 按 A～D 批对照主原型补齐与验收；代码完成前不得把该项标为生产完成 |

统一措辞：后端已实现；生产前端部分接入。本提案执行逐项对齐，完成前不得勾选「相对主原型已对齐」。

### 13.1 S01～S05 / 壳层三列总表

| 能力项 | 已有能力 | 规格已收口 | 本提案实现 |
|---|---|---|---|
| S01 保存 / SaveState / 非 OT 409 | 是（后端+前端保存链路） | 是（含协作禁 409） | C 批（已验证） |
| S02 分享只读 / 404 / 无 share→auth | 是（分享加载） | 是 | A 批（已验证） |
| S03 auth→rooms / 会话 / 不枚举用户 | 是（API+部分 UI） | 是 | A 批（已验证） |
| S04 rooms/invite/成员/Viewer | 是（API+部分 UI） | 是 | B 批（已验证） |
| S05 ws-status/ot-rev/presence/reconnect/queue/local-only | 是（WS+部分 UI） | 是 | C 批（已验证） |
| 画布拖表 GRID_SIZE=20 + 跟线 | 部分 | 是 | D 批（已验证） |
| 关系 4px / rubber-band / 两点 / 确认条 | 部分 | 是 | D 批（已验证） |
| IO 更多菜单→抽屉 | 部分 | 是 | D 批（已验证） |
| 主模态 Esc 无残留 | 部分 | 是 | D 批（已验证） |
| Inspector 锚点 + 响应式抽屉 | 部分 | 是 | D 批（已验证） |
| ⌘K / Esc / T / R | 部分 | 是 | D 批（已验证） |
| Design system 主题/motion | 部分（E1–E6 已落地基础） | 是（与统一壳层对齐合同） | D 批（已验证） |
| 主原型演示器本身 | 是（静态 HTML） | N/A（不改原型） | 禁止标生产完成 |

### 13.2 既有勾选区解读规则

- 历史 `[x]` 仅表示**已有能力**列意义上的存在，**不**自动等于「相对主原型逐项对齐完成」。
- 凡涉及 auth/rooms/invite/room-editor 视觉与交互贴合主原型的项：在本提案对应批次验证前保持未完成，不得提前勾选。
- Monaco 完整挂载、Mermaid/PNG 导出、K8s 等原边界项：仍为未完成，不得改完成。

### 13.3 本提案执行批次

本提案即第二阶段执行入口。验收输入：已合并测试矩阵 + `core-frontend-alignment-acceptance.md` §7。

| 批次 | 范围 | 主要用例 |
|---|---|---|
| A（已验证） | auth / share / 页面流入口 | ST-S03-UI-*、S02 SHARE/*、ST-FE-ALIGN-01/02、ST-PU-22 |
| B（已验证） | rooms / invite | ST-S04-UI-*、ST-PU-23 |
| C（已验证） | room-editor 壳层 / 保存 / 协作 | S01-SS/*、S01-409/*、ST-S05-UI-*、ST-FE-ALIGN-03/04、ST-PU-24 |
| D（已验证） | IO / 快捷键 / 主题 / 响应式 / 画布与关系 | ST-KB-*、ST-PC-MENU-01/FMT-01/INSPECTOR、ST-PU-25/26、ST-CR-02、ST-PB-01/02 |
