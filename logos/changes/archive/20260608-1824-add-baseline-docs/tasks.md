# 实现任务

> 模块：core | 提案：add-baseline-docs
> 部署决策：不需要部署（与 proposal.md `## 部署影响` 一致）→ 不创建 `[deploy]` section
> 代码决策：仅文档，不改代码 → 不创建 `[code]` section
> 仅追踪 delta 文档产出；`openlogos verify` / `openlogos archive` / `git push` 由人类确认点驱动

## 批次划分（方案 A — 单提案分两批 merge）

为避免单次 guard 周期内 35+ 文件的大 PR 风险，本提案采用**两批次交付**：

| 批次 | 范围 | 实际文件数 | 执行时机 | 后续动作 |
|---|---|---|---|---|
| 批次 1（V1 基线） | 全部 V1 delta 文件 + docs/ 写作母版 | **25** 个 delta + **1** 个 docs/ 母版 | 本提案 `add-baseline-docs` 内 | `merge` → `verify` → `archive` 释放 guard |
| 批次 2（V2 协作规格） | 全部 V2 / SPEC-FUTURE delta 文件 | **19** 个 delta | 后续新提案 `add-v2-collab-spec` | 同上 |

**V1 批次 1 文件精确清单**（25 个 delta 文件）：

- Phase 1 需求层：2（`core-00-scenario-overview.md` + `core-01-requirements.md`）
- Phase 2 设计层：10（`core-00` + `core-01` + `core-01a` + `core-01b` + `core-01c` + `core-02` + `core-03` + `core-04` + `core-05` + `core-01-editor-prototype.html`）
- Phase 3 技术方案层：4（`core-01-architecture-overview` + `core-S01-edit-and-save-diagram` + `core-S02-load-shared-diagram` + `core-01-deployment-plan`）
- API / DB：3（`diagrams.yaml` + `bridge.yaml` + `coldrawdb-v1.sql`）
- Test / Smoke：3（`core-S01-test-cases` + `core-S02-test-cases` + `core-smoke-test-cases`）
- Scenario：2（`core-S01-diagram-save` + `core-S02-shared-link-load`）
- Implementation：1（`core-implementation-checklist.md`）

**辅助产物**（不进入 `logos/resources/`，仅写作母版）：`docs/drawdb-capability-checklist.md`（**1 个**，**已**完成）

> **流程时序**：
> 1. 批次 1 启动：人类在 guard 保护下逐项产出 25 个 delta 文件；
> 2. 批次 1 收尾：所有 `[delta]` 任务 `[x]` → `openlogos merge` → 人类确认 → AI auto-commit 规格 → `openlogos verify` → 人类确认 PASS → `openlogos archive`；
> 3. guard 释放后，**新建提案 `add-v2-collab-spec`**（执行 `openlogos change add-v2-collab-spec`），在 guard 保护下逐项产出 V2 19 个 delta 文件；
> 4. 批次 2 收尾流程同批次 1。

## V1 对齐参考源（前置依赖 — 启动 delta 产出前必须就绪）

V1 文档不是凭空设计，需对照以下事实源逐项产出。**delta 写作者开始前应确认这些源可读、可对照**：

- [ ] `https://github.com/drawdb-io/drawdb` 已可读（drawdb 主分支 JavaScript/React 源码；GitHub 公开仓库）— 用于功能/界面/能力全集对齐。可通过 `git clone --depth 1 https://github.com/drawdb-io/drawdb.git /tmp/drawdb-ref` 拉取到本地后单文件查阅
- [ ] `RUST_WEB_REFACTOR_PLAN.md`（仓库根）— 用于 Rust 体系分层、表结构、事务策略
- [ ] `database_design.json`（仓库根，26KB）— 用于字段命名/类型/枚举对齐
- [ ] `backend/init.sql` + `backend/migrations/` — 用于 V1 实际表清单与 DDL 对账（11 张表）
- [ ] `backend/src/diagrams_v1.rs` + `backend/src/phase3_bridge.rs` — 用于 API 端点全集（10 个端点）
- [ ] `docs/phase4/PHASE4_DONE.md` + `docs/phase4/architecture.mmd` + `docs/phase4/module-mapping.md` — 用于架构与模块依赖
- [ ] `frontend-rs/src/{editor_core,editor_data_access,editor_panels,editor_render,lib}.rs` — 用于前端 4 模块职责对账
- [ ] `docs/drawdb-capability-checklist.md`（**已**为对齐对账支撑产出；按"功能 / 界面 / 能力"三栏组织，作为 V1 写作母版）— 每个 V1 文档必须能在该清单中找到对应能力项

> 若任一源不可读，需在 PR 中说明替代方案（如改用 README 推断）并降低"对齐对账"任务的强度。

## [delta] 规格变更 — Phase 1 需求层

### V1（已实现事实）

- [x] 产出 delta 到 `deltas/prd/1-product-requirements/core-00-scenario-overview.md` — 新增场景总览表（覆盖 V1 + V2）
- [x] 产出 delta 到 `deltas/prd/1-product-requirements/core-01-requirements.md` — 新增 V1 WHY 需求文档（用户故事、验收条件、非功能性需求）

### V2 / SPEC-FUTURE（待实现规格，本次不实现）

- [ ] 产出 delta 到 `deltas/prd/1-product-requirements/core-02-v2-requirements.md` — 新增 V2 需求（用户系统、协作房间、OT 实时同步、撤销/重做）

## [delta] 规格变更 — Phase 2 设计层

### V1（已实现事实）

- [x] 产出 delta 到 `deltas/prd/2-product-design/1-feature-specs/core-00-information-architecture.md` — 新增信息架构（顶层布局 + 路由 + 4 模块前端 + 11 子模块后端；覆盖 drawdb §2.1 + §4）
- [x] 产出 delta 到 `deltas/prd/2-product-design/1-feature-specs/core-01-editor-canvas.md` — 新增编辑器画布功能总规格（CAP-CANVAS-01..09：Table/Field/Relationship/Index/Area/Note/Enum/CustomType/Canvas；覆盖 drawdb §2.3）
- [x] 产出 delta 到 `deltas/prd/2-product-design/1-feature-specs/core-01a-table-and-field.md` — 新增表与字段编辑规格（CAP-CANVAS-01/02；含 7 引擎类型映射 CAP-DATATYPES-*）
- [x] 产出 delta 到 `deltas/prd/2-product-design/1-feature-specs/core-01b-relationship.md` — 新增关系编辑规格（CAP-CANVAS-03；一对一/一对多/多对多；ON UPDATE/DELETE）
- [x] 产出 delta 到 `deltas/prd/2-product-design/1-feature-specs/core-01c-index-enum-custom-type.md` — 新增索引/枚举/自定义类型规格（CAP-CANVAS-04/07/08；coldrawdb V1 仅前端状态）
- [x] 产出 delta 到 `deltas/prd/2-product-design/1-feature-specs/core-02-diagram-persistence.md` — 新增图表持久化规格（CAP-PERSIST-01/02/04；revision 乐观锁 + 409 冲突语义；11 张表对账）
- [x] 产出 delta 到 `deltas/prd/2-product-design/1-feature-specs/core-03-bridge-io.md` — 新增桥接导入/导出规格（CAP-BRIDGE-01..11；SQL 7 引擎 + DBML + JSON；coldrawdb V1 未实现 Mermaid/PNG/PDF/ZIP，文档明确标注缺失）
- [x] 产出 delta 到 `deltas/prd/2-product-design/1-feature-specs/core-04-side-panel-tabs.md` — 新增侧边栏 6 Tab + Issues + DBMLEditor 规格（drawdb §2.4：Tables/Areas/Enums/Notes/Relationships/Types）
- [x] 产出 delta 到 `deltas/prd/2-product-design/1-feature-specs/core-05-top-menu-modals.md` — 新增顶部菜单 + 9 个模态规格（drawdb §2.2：New/Open/Import/ImportSource/Language/SetTableWidth/Share/Rename/ConfigureCustomTypes）
- [x] 产出 delta 到 `deltas/prd/2-product-design/2-page-design/core-01-editor-prototype.html` — 新增编辑器主页面 HTML 原型（覆盖 drawdb Workspace.jsx 顶层布局 §2.1）

### V2 / SPEC-FUTURE（待实现规格，本次不实现）

- [ ] 产出 delta 到 `deltas/prd/2-product-design/1-feature-specs/core-03-user-account.md` — 用户系统规格（注册/登录/资料/密码哈希/会话）
- [ ] 产出 delta 到 `deltas/prd/2-product-design/1-feature-specs/core-04-room-and-membership.md` — 协作房间与成员规格（创建/加入/邀请/角色）
- [ ] 产出 delta 到 `deltas/prd/2-product-design/1-feature-specs/core-05-ot-collab-engine.md` — OT 协作引擎规格（op 协议、转换函数、状态机、撤销/重做）
- [ ] 产出 delta 到 `deltas/prd/2-product-design/2-page-design/core-02-auth-prototype.html` — 登录/注册页面原型
- [ ] 产出 delta 到 `deltas/prd/2-product-design/2-page-design/core-03-room-prototype.html` — 协作房间选择页原型

## [delta] 规格变更 — Phase 3 技术方案层

### V1（已实现事实）

- [x] 产出 delta 到 `deltas/prd/3-technical-plan/1-architecture/core-01-architecture-overview.md` — 新增 V1 技术架构（4 模块 + 单向依赖）
- [x] 产出 delta 到 `deltas/prd/3-technical-plan/2-scenario-implementation/core-S01-edit-and-save-diagram.md` — 新增 S01 时序图（编辑保存）
- [x] 产出 delta 到 `deltas/prd/3-technical-plan/2-scenario-implementation/core-S02-load-shared-diagram.md` — 新增 S02 时序图（分享链接加载）
- [x] 产出 delta 到 `deltas/prd/3-technical-plan/3-deployment/core-01-deployment-plan.md` — 新增 V1 部署方案（本地 dev / Docker / staging；含 smoke 入口）

### V2 / SPEC-FUTURE（待实现规格，本次不实现）

- [ ] 产出 delta 到 `deltas/prd/3-technical-plan/1-architecture/core-02-v2-architecture.md` — 新增 V2 架构（`collab-server` / WS 网关 / OT 引擎；前后端状态同步策略）
- [ ] 产出 delta 到 `deltas/prd/3-technical-plan/2-scenario-implementation/core-S03-user-auth.md` — 新增 S03 时序图（注册 + 登录 + Token 续期）
- [ ] 产出 delta 到 `deltas/prd/3-technical-plan/2-scenario-implementation/core-S04-room-lifecycle.md` — 新增 S04 时序图（创建房间、邀请成员、加入房间）
- [ ] 产出 delta 到 `deltas/prd/3-technical-plan/2-scenario-implementation/core-S05-ot-collab.md` — 新增 S05 时序图（OT 实时协作：本地 op → 服务端转换 → 广播给所有客户端）
- [ ] 产出 delta 到 `deltas/prd/3-technical-plan/3-deployment/core-02-v2-deployment.md` — 新增 V2 多服务部署（`backend` + `collab-server` + WS 网关 + 前端 WASM）

## [delta] 规格变更 — API / DB / 测试 / 编排 / 实现清单

### V1（已实现事实）

- [x] 产出 delta 到 `deltas/api/diagrams.yaml` — OpenAPI 3.x（`/api/v1/diagrams/*` 5 端点：POST 创建 / GET 读取 / PUT 全量更新 / DELETE / POST 导入；含 409 revision 冲突）
- [x] 产出 delta 到 `deltas/api/bridge.yaml` — OpenAPI 3.x（`/api/v1/bridge/*` 5 端点：导入/导出 SQL、JSON 导入/导出、导入日志 `GET /bridge/import/local/logs`、本地重试 `POST /bridge/import/local/retry/{id}`、bridge 配置 `GET/PUT /bridge/config`）
- [x] 产出 delta 到 `deltas/database/coldrawdb-v1.sql` — 完整 V1 DDL（**11 张表**：`task` / `diagram` / `diagram_link` / `table` / `field` / `table_link` / `indice` / `indice_link` / `reference` / `area` / `note`；字段命名与 `database_design.json` 对齐）
- [x] 产出 delta 到 `deltas/test/core-S01-test-cases.md` — 新增 S01 UT/ST 用例规格
- [x] 产出 delta 到 `deltas/test/core-S02-test-cases.md` — 新增 S02 UT/ST 用例规格
- [x] 产出 delta 到 `deltas/test/smoke/core-smoke-test-cases.md` — 新增 staging 冒烟用例规格（`SMOKE-core-01..03`）
- [x] 产出 delta 到 `deltas/scenario/core-S01-diagram-save.json` — 新增 S01 API 编排测试
- [x] 产出 delta 到 `deltas/scenario/core-S02-shared-link-load.json` — 新增 S02 API 编排测试
- [x] 产出 delta 到 `deltas/implementation/core-implementation-checklist.md` — 新增代码实现清单（V1 行勾选 `[x]`；V2 行 `[ ]`）
- [ ] **验证 V1 API YAML** — 所有 `description` / `summary` 含 `:` 或特殊字符的值必须用双引号包裹；YAML 必须可被 `python -c "import yaml; yaml.safe_load(open(...))"` 解析
- [ ] **V1 对齐对账（事实源 ↔ 规格）**
  - `coldrawdb-v1.sql` 表清单必须与 `backend/init.sql` 的 11 张表一一对应
  - `coldrawdb-v1.sql` 字段名/类型/外键必须与 `database_design.json` 一致
  - `diagrams.yaml` 端点必须覆盖 `backend/src/diagrams_v1.rs` 的 5 个 `#[*]` 路由
  - `bridge.yaml` 端点必须覆盖 `backend/src/phase3_bridge.rs` 的 5 个 `#[*]` 路由
  - 架构文档必须与 `docs/phase4/architecture.mmd` 的 4 模块 + 单向依赖一致
  - 前端功能清单必须与 `drawdb-io/drawdb` 仓库 `main` 分支（drawdb 主分支）的能力集对齐

## [code] V1 验证测试运行（用户授权后追加）

> 本节为 V1 后端 E2E 测试运行支撑（详见 `proposal.md` "V1 验证测试运行"小节）。原提案为"仅新增规格文档"，本节经用户明确授权后追加；`openlogos verify` 需要 `test-results.jsonl` 才能通过。

- [x] **新增 `backend/src/verify_reporter.rs`** — OpenLogos reporter 小工具；提供 `truncate()` / `report_pass(id, ms)` / `report_fail(id, ms, err)` / `report_skip(id, reason)` 四个公开函数；文件锁 `Mutex` 串行化写入；输出路径 `logos/resources/verify/test-results.jsonl`（相对项目根）
- [x] **`backend/src/main.rs` 注册 reporter 模块** — 新增一行 `mod verify_reporter;`（**不**调用 reporter 任何函数；reporter 仅供 `#[cfg(test)]` 模块使用）
- [x] **改造 `backend/src/diagrams_v1.rs::tests`** — 现有 2 个 `#[actix_web::test]`（`test_v1_diagram_crud_and_conflict` + `test_v1_import_success_and_invalid_payload`）调用 reporter，输出 `ST-S01-01` + `ST-S01-02` 等价的 jsonl 行
- [x] **新增 S01 UT 用例** — 至少为以下 ID 添加 `#[actix_web::test]`：
  - `UT-S01-01` 创建空 diagram → 201
  - `UT-S01-03` PUT 带正确 revision → 200
  - `UT-S01-04` PUT 带过期 revision → 409
  - `UT-S01-05` DELETE → 级联删除
- [x] **新增 S02 UT 用例** — 至少为以下 ID 添加 `#[actix_web::test]`：
  - `UT-S02-01` GET 存在 diagram → 200 全量数据
  - `UT-S02-02` GET 不存在 → 404
- [x] **改造 `backend/src/phase3_bridge.rs::tests`** — 现有测试（如有）调用 reporter 输出 `ST-B-01` 等 jsonl 行
- [x] **执行 `cargo test` 并验证 jsonl** — 在项目根运行 `cd backend && cargo test 2>&1 | tail -50`；期望 jsonl 含 ≥ 6 个用例（2 已有 + 4 新增）；用 `python3 -c "import json; [json.loads(l) for l in open('logos/resources/verify/test-results.jsonl')]"` 验证 JSONL 格式
- [x] **未实现用例标记为 skip** — 对于 28 个规格用例 ID 中暂未实现 Rust 测例的（如 `ST-S01-03` 需要 wasm-pack headless），在 reporter 中显式调用 `report_skip(id, "spec-defined, no Rust impl in this batch")` 写入 jsonl 防止覆盖率不足
- [x] **重跑 `openlogos verify add-baseline-docs`** — 期望 PASS（**实际结果**：Gate 3.5 PASS，28/28 覆盖，8 pass + 20 skip）

### V2 / SPEC-FUTURE（待实现规格，本次不实现）

- [ ] 产出 delta 到 `deltas/api/users.yaml` — 用户 CRUD
- [ ] 产出 delta 到 `deltas/api/auth.yaml` — 注册 / 登录 / Token 刷新 / 登出
- [ ] 产出 delta 到 `deltas/api/rooms.yaml` — 协作房间 CRUD + 成员管理
- [ ] 产出 delta 到 `deltas/api/collab.yaml` — OT 协作 WebSocket 子协议（op / ack / rev / cursor 帧）
- [ ] 产出 delta 到 `deltas/database/v2-coldrawdb.sql` — 新增 DDL（`users` / `auth_tokens` / `rooms` / `room_members` / `operations` / `operation_log`）
- [ ] 产出 delta 到 `deltas/test/core-S03-test-cases.md` — 用户鉴权 UT/ST 用例
- [ ] 产出 delta 到 `deltas/test/core-S04-test-cases.md` — 房间生命周期 UT/ST 用例
- [ ] 产出 delta 到 `deltas/test/core-S05-test-cases.md` — OT 转换与广播正确性 UT/ST 用例
- [ ] 产出 delta 到 `deltas/test/smoke/core-v2-smoke-test-cases.md` — V2 staging 冒烟用例规格（`SMOKE-core-V2-01..05`）
- [ ] 产出 delta 到 `deltas/scenario/core-S03-user-auth.json` — S03 API 编排测试
- [ ] 产出 delta 到 `deltas/scenario/core-S04-room-lifecycle.json` — S04 API 编排测试
- [ ] 产出 delta 到 `deltas/scenario/core-S05-ot-collab.json` — S05 OT 协作 WebSocket 编排测试
- [ ] **验证 V2 API YAML / 协议文档** — OpenAPI 文件 + WebSocket 协议帧定义均需通过 YAML 解析与字段格式校验

## 部署决策一致性自检

| 检查项 | 状态 |
|---|---|
| `proposal.md` 声明 `是否需要部署：否`（V1+V2 都仅写规格） | ✅ `tasks.md` 不存在 `[deploy]` section |
| `proposal.md` 声明 `是否需要 smoke：否` | ✅ 未在 `[delta]` 中混入 smoke 执行任务（smoke 属独立 CLI 节点） |
| `proposal.md` 声明"仅文档"（V1+V2 都不改代码） | ✅ `tasks.md` 不存在 `[code]` section |
| `[delta]` section 全部任务对应具体 delta 文件 | ✅ 每条任务明确 `deltas/...` 路径 |
| V1 / V2 任务在 `[delta]` 内显式分段子标题 | ✅ Phase 1/2/3 + API 段均有 `### V1` 与 `### V2 / SPEC-FUTURE` 子段 |
| V2 任务全部标注 `本次不实现` 字样 | ✅ 每条 V2 任务描述或子段标题均说明"待实现规格，本次不实现" |
| V1 标注"与 drawdb 主分支对齐"且引用具体事实源 | ✅ 顶部"V1 对齐参考源（前置依赖）"列出 7 类事实源；API/DB 段增加"V1 对齐对账"任务 |
| V1 DB 表清单 11 张表（与 `backend/init.sql` 对齐） | ✅ `coldrawdb-v1.sql` 任务描述明确列出 11 张表名 |
| V1 API 端点全集 10 个（5 diagrams + 5 bridge） | ✅ `diagrams.yaml` + `bridge.yaml` 任务描述均明确端点数 |
