# ADDED — V1 + V2 代码实现清单
# 模块：core | 提案：add-baseline-docs
# 路径：`logos/resources/implementation/core-implementation-checklist.md`
# 对齐参考源：Phase 4 已实现 + 批次 1/2 文档

## 1. 范围

本文件追踪 coldrawdb V1（已实现）+ V2（待实现）代码行勾选状态。

**V1 行** = 已在 Phase 1-4 完成的事实代码
**V2 行** = 待 V2 阶段（`add-v2-collab-spec`）实现

> 本清单仅作总览与状态标记。详细规格见各 delta 文件。

## 2. 前端 4 模块

### 2.1 editor_data_access

- [x] HTTP 客户端（`gloo_net::http::Request`）
- [x] diagrams API 封装（fetch_diagram / save / create / delete / import）
- [x] bridge API 封装（import / logs / retry / config）
- [x] debounce 1s 自动保存循环
- [x] 错误处理 + 指数退避重试
- [x] SaveState 状态机
- [ ] WebSocket 客户端（V2 实时协作）

### 2.2 editor_core

- [x] Diagram 状态机（RwSignal<Diagram>）
- [x] 字段级 UndoRedoContext
- [x] dirty 标记
- [x] revision 跟踪
- [x] set_diagram / push_undo / undo / redo
- [ ] OT 操作队列（V2）

### 2.3 editor_panels

- [x] Tables Tab + 列表项 + 增删改
- [x] Areas Tab
- [x] Enums Tab（V1 仅前端 state）
- [x] Notes Tab
- [x] Relationships Tab
- [x] Types Tab（V1 仅前端 state）
- [x] Issues Tab + 校验引擎
- [ ] DBMLEditor 备选视图（V1 边界，spec core-04 §9 留 V2）
- [x] 全局搜索 + 类型筛选
- [ ] 房间成员列表（V2）

### 2.4 editor_render

- [x] Canvas 容器 + 平移 + 缩放 + 框选
- [x] Table 渲染
- [x] Field 行渲染
- [x] Relationship 贝塞尔连线
- [x] Area 矩形 + 标签
- [x] Note 富文本
- [x] 选中 / 高亮 / 闪烁
- [x] 撤销栈深度指示
- [ ] 协作者光标渲染（V2）

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

### 9.1 V1 文档（已完成 25/25）

- [x] 需求层 2 文件
- [x] 设计层 10 文件（含 1 HTML 原型）
- [x] 技术方案层 4 文件
- [x] API 2 文件（diagrams.yaml + bridge.yaml）
- [x] DB 1 文件（coldrawdb-v1.sql）
- [x] 测试 3 文件（S01 + S02 + smoke）
- [x] 场景 2 文件（S01 + S02 JSON）
- [x] 实现清单 1 文件（本文件）

### 9.2 V2 文档（待 19/19，本提案外）

- [ ] 需求层 1 文件
- [ ] 设计层 5 文件
- [ ] 技术方案层 4 文件
- [ ] API 4 文件
- [ ] DB 1 文件
- [ ] 测试 3 文件
- [ ] 场景 3 文件

## 10. 关键指标

| 指标 | V1 实际 | V2 计划 |
|---|---|---|
| 前端模块 | 4 | 4 + WS client |
| 后端模块 | 11 + 5 routing | 11 + 5 routing + collab-server |
| 数据表 | 11 | 17（+ users / auth_tokens / rooms / room_members / operations / operation_log） |
| API 端点 | 10 | 10 + 4 鉴权 + 3 房间 + WS 1 端口 |
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
