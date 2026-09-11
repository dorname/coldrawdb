# 变更提案：add-baseline-docs

> module: core | created: 2026-06-08

## 变更原因

### 现状

项目 `coldrawdb` 已处于 `launched` 生命周期，分支目标分三步推进：

| 阶段 | 目标 | 现状 |
|---|---|---|
| ① Rust 技术栈迁移 | 替换原 React 前端 | ✅ **已完成**（Phase 4 收官，CI green） |
| ② Web 版本 | 浏览器端可访问 | ✅ **已完成**（Leptos 0.x + WASM + trunk + 4 模块架构） |
| ②-补 V1 功能/界面/能力对齐 | **与 drawdb 主分支一致**：编辑器、SQL 导入导出、桥接、JSON 导入/导出、主题、撤销/重做等 | ✅ **已对齐**（功能/界面/能力与 drawdb 一致，可参考主分支 drawdb 源码） |
| ③ 完整用户系统 + 实时多人协作 | 注册/登录、协作房间、OT 实时同步 | ❌ **未实现**（V2 计划） |

### V1 对齐参考源（事实锚点）

V1 文档不是凭空设计，而是**对照以下事实源逐项记录 drawdb 已实现的功能/界面/能力**。规格写作者在产出 delta 时必须以这些源为依据：

| 参考源 | 路径 | 用途 |
|---|---|---|
| drawdb 主分支源码 | https://github.com/drawdb-io/drawdb.git（GitHub 公开仓库 `main` 分支） | drawdb 功能/界面/能力全集（JavaScript/React） |
| drawdb 仓官方仓库 | https://github.com/drawdb-io/drawdb | 文档站（drawdb.app）、issue tracker、参考实现 |
| drawdb-server | https://github.com/drawdb-io/drawdb-server | 分享/协作后端实现（V2 时序图可参考） |
| coldrawdb 主分支 | `origin/main` | 当前仓库的主分支；本提案所在分支 `drawdb-web` 是开发分支 |
| coldrawdb 历史 | `origin/coldrawdb-app`、`origin/legacys`、`origin/try-by-ai` | React → Rust 迁移的中间形态（`legacys` 是 coldrawdb 旧版本，**不是** drawdb 主分支） |
| 改造实施文档 | `RUST_WEB_REFACTOR_PLAN.md`（仓库根） | Rust 体系分层、表结构、事务策略、迁移流程 |
| Phase 4 收官报告 | `docs/phase4/PHASE4_DONE.md` | 4 模块 + 单向依赖、CI green、W4 perf |
| 数据库设计样本 | `database_design.json`（仓库根，26KB） | drawdb diagram JSON 真实结构样本（tables/fields/indices/relationships/notes/subjectAreas） |
| 后端 Rust 现状 | `backend/src/`（11 个子模块） | entities：areas / diagrams / fields / indices / notes / references / tables + todos / common / error / repository；v1 路由：`diagrams_v1.rs`（5 端点）+ `phase3_bridge.rs`（5 端点） |
| 数据库 DDL 现状 | `backend/init.sql` + `backend/migrations/0001_phase1_schema.{up,down}.sql` + `0002_phase3_bridge.{up,down}.sql` | V1 实际表清单：`task` / `diagram` / `diagram_link` / `table` / `field` / `table_link` / `indice` / `indice_link` / `reference` / `area` / `note`（**共 11 张表**） |
| 前端 Rust 现状 | `frontend-rs/src/`（4 模块） | `editor_core`（状态机 + 撤销/重做）/ `editor_data_access`（HTTP API 客户端 + debounce 1s）/ `editor_panels`（侧栏）/ `editor_render`（canvas + 贝塞尔连线） |
| 架构图 | `docs/phase4/architecture.mmd` | 4 模块 + 单向依赖图 |
| 模块映射 | `docs/phase4/module-mapping.md` | 模块边界与依赖方向 |

### 文档侧的空缺

`logos/resources/` 下的规格文档仅有占位 `.gitkeep`，**没有任何可执行的主规格文档**。这导致：

1. AI 助手（Claude Code 等）缺少"真相源"，无法基于 OpenLogos 规范进行需求分析、设计评审、影响范围评估；
2. 后续任何需求级/设计级/接口级变更（V2 多人协作即属此类）都没有可对照的基线，Delta 合并（`openlogos merge`）缺乏落点；
3. 团队成员难以快速理解当前系统的需求、设计、架构、API、DB、测试契约。

### 本变更的目的

**一次性建立 V1（事实）+ V2（计划）的跨阶段完整基线文档**：

- V1 层以 Phase 4 完成态为事实锚点，记录 Rust + actix-web + SQLite + Leptos 0.x + WASM 的实际架构；
- V2 层以"SPEC-FUTURE"标记覆盖用户系统 + OT 实时协作的**设计骨架**（不实现，仅规格），使后续 V2 实施直接以本基线为起点做增量 Delta，避免重开 change 提案。

注意：本变更**仅新增规格文档**，不修改任何源代码（`backend/`、`frontend-rs/` 保持现状）；不涉及部署；不涉及数据迁移。V2 的 OT 服务、WS 协议、用户表结构**仅以规格形式存在**，本次不实现、不部署。

## 变更类型

需求级

## 变更范围

> 版本分层标注：
> - `[V1]` = Phase 4 已实现事实（基线锚点）
> - `[V2 / SPEC-FUTURE]` = 用户系统 + OT 协作的设计骨架（待实现规格，本次不实现）
> - 本变更**仅新增**文件；MODIFIED/REMOVED 不适用。

### 对齐对账支撑（事实源 ↔ 规格的中间桥梁）

- `[V1]` `docs/drawdb-capability-checklist.md` — **drawdb 主分支能力比对母版**（从 `https://github.com/drawdb-io/drawdb` GitHub 公开仓库 `main` 分支整理），作为 V1 写作的事实源；`add-baseline-docs` 的每个 V1 文档必须在该清单中能找到对应能力项。**不进入 `logos/resources/`**，仅在 `docs/` 下作为写作参考。

### Phase 1：需求层

- `[V1]` `logos/resources/prd/1-product-requirements/core-00-scenario-overview.md` — 场景总览表（覆盖 V1+V2）
- `[V1]` `logos/resources/prd/1-product-requirements/core-01-requirements.md` — 现状需求（编辑器、SQL 导出、浏览器端使用）
- `[V2 / SPEC-FUTURE]` `logos/resources/prd/1-product-requirements/core-02-v2-requirements.md` — V2 需求（用户系统、协作房间、OT 实时同步）

### Phase 2：设计层

- `[V1]` `logos/resources/prd/2-product-design/1-feature-specs/core-00-information-architecture.md` — 信息架构
- `[V1]` `logos/resources/prd/2-product-design/1-feature-specs/core-01-editor-canvas.md` — 编辑器画布与表/字段/连线交互
- `[V1]` `logos/resources/prd/2-product-design/1-feature-specs/core-02-diagram-persistence.md` — 图表持久化与 revision 冲突语义
- `[V1]` `logos/resources/prd/2-product-design/2-page-design/core-01-editor-prototype.html` — 编辑器主页面 HTML 原型
- `[V2 / SPEC-FUTURE]` `logos/resources/prd/2-product-design/1-feature-specs/core-03-user-account.md` — 用户系统规格（注册/登录/资料/密码哈希/会话）
- `[V2 / SPEC-FUTURE]` `logos/resources/prd/2-product-design/1-feature-specs/core-04-room-and-membership.md` — 协作房间与成员规格（创建/加入/邀请/角色）
- `[V2 / SPEC-FUTURE]` `logos/resources/prd/2-product-design/1-feature-specs/core-05-ot-collab-engine.md` — OT 协作引擎规格（op 协议、转换函数、状态机、撤销重做）
- `[V2 / SPEC-FUTURE]` `logos/resources/prd/2-product-design/2-page-design/core-02-auth-prototype.html` — 登录/注册页面原型
- `[V2 / SPEC-FUTURE]` `logos/resources/prd/2-product-design/2-page-design/core-03-room-prototype.html` — 协作房间选择页原型

### Phase 3：技术方案层

- `[V1]` `logos/resources/prd/3-technical-plan/1-architecture/core-01-architecture-overview.md` — V1 技术架构（4 模块 + 单向依赖）
- `[V1]` `logos/resources/prd/3-technical-plan/2-scenario-implementation/core-S01-edit-and-save-diagram.md` — S01 时序图（编辑保存）
- `[V1]` `logos/resources/prd/3-technical-plan/2-scenario-implementation/core-S02-load-shared-diagram.md` — S02 时序图（分享链接加载）
- `[V1]` `logos/resources/prd/3-technical-plan/3-deployment/core-01-deployment-plan.md` — V1 部署方案（本地 dev / Docker / staging）
- `[V2 / SPEC-FUTURE]` `logos/resources/prd/3-technical-plan/1-architecture/core-02-v2-architecture.md` — V2 架构（新增 `collab-server` / WS 网关 / OT 引擎；前后端状态同步策略）
- `[V2 / SPEC-FUTURE]` `logos/resources/prd/3-technical-plan/2-scenario-implementation/core-S03-user-auth.md` — S03 时序图（注册 + 登录 + Token 续期）
- `[V2 / SPEC-FUTURE]` `logos/resources/prd/3-technical-plan/2-scenario-implementation/core-S04-room-lifecycle.md` — S04 时序图（创建房间、邀请成员、加入房间）
- `[V2 / SPEC-FUTURE]` `logos/resources/prd/3-technical-plan/2-scenario-implementation/core-S05-ot-collab.md` — S05 时序图（OT 实时协作：本地 op → 服务端转换 → 广播给所有客户端）
- `[V2 / SPEC-FUTURE]` `logos/resources/prd/3-technical-plan/3-deployment/core-02-v2-deployment.md` — V2 多服务部署（`backend` + `collab-server` + WS 网关 + 前端 WASM）

### API / DB / 测试 / 编排 / 实现清单

- `[V1]` `logos/resources/api/diagrams.yaml` — `/api/v1/diagrams/*` v1 CRUD + 409 revision 冲突（5 端点）
- `[V1]` `logos/resources/api/bridge.yaml` — `/api/v1/bridge/*` 桥接 API（5 端点：导入/导出 SQL、JSON 导入/导出、导入日志、本地重试、bridge 配置）
- `[V2 / SPEC-FUTURE]` `logos/resources/api/users.yaml` — 用户 CRUD
- `[V2 / SPEC-FUTURE]` `logos/resources/api/auth.yaml` — 注册/登录/Token 刷新/登出
- `[V2 / SPEC-FUTURE]` `logos/resources/api/rooms.yaml` — 协作房间 CRUD + 成员管理
- `[V2 / SPEC-FUTURE]` `logos/resources/api/collab.yaml` — OT 协作 WebSocket 子协议（op 帧、ack、rev、cursor 帧）
- `[V1]` `logos/resources/database/coldrawdb-v1.sql` — V1 完整 DDL（**11 张表**：`task` / `diagram` / `diagram_link` / `table` / `field` / `table_link` / `indice` / `indice_link` / `reference` / `area` / `note`），字段命名与 `database_design.json` 完全对齐
- `[V2 / SPEC-FUTURE]` `logos/resources/database/v2-coldrawdb.sql` — `users` / `auth_tokens` / `rooms` / `room_members` / `operations` / `operation_log` 表
- `[V1]` `logos/resources/test/core-S01-test-cases.md`、`core-S02-test-cases.md`
- `[V1]` `logos/resources/test/smoke/core-smoke-test-cases.md` — `SMOKE-core-01..03`
- `[V2 / SPEC-FUTURE]` `logos/resources/test/core-S03-test-cases.md`（用户鉴权）
- `[V2 / SPEC-FUTURE]` `logos/resources/test/core-S04-test-cases.md`（房间生命周期）
- `[V2 / SPEC-FUTURE]` `logos/resources/test/core-S05-test-cases.md`（OT 转换与广播正确性）
- `[V2 / SPEC-FUTURE]` `logos/resources/test/smoke/core-v2-smoke-test-cases.md` — `SMOKE-core-V2-01..05`（V2 部署后冒烟）
- `[V1]` `logos/resources/scenario/core-S01-diagram-save.json`、`core-S02-shared-link-load.json`
- `[V2 / SPEC-FUTURE]` `logos/resources/scenario/core-S03-user-auth.json`、`core-S04-room-lifecycle.json`、`core-S05-ot-collab.json`
- `[V1]` `logos/resources/implementation/core-implementation-checklist.md` — V1 勾选完成；V2 行标记 `[ ]` 待实施

### V1 验证测试运行（**用户授权后追加**）

- `[V1]` `backend/src/verify_reporter.rs`（**新增**，**仅本提案范围**）— OpenLogos reporter 小工具；测试运行时调用 `report_pass` / `report_fail` / `report_skip` 追加写入 `logos/resources/verify/test-results.jsonl`（覆盖 28 个 UT/ST 用例 ID：`UT-S01-01..10` + `ST-S01-01..03` + `UT-S02-01..09` + `ST-S02-01..06`）
- `[V1]` `backend/src/diagrams_v1.rs::tests` 与 `phase3_bridge.rs::tests`（**在已有测试**上改造）— 调用 reporter；为新增 UT/ST 用例 ID 添加 `#[actix_web::test]` 函数（沿用现有 `build_db` 模式）
- `[V1]` `backend/src/main.rs`（**新增一行** `mod verify_reporter;`）— 让 reporter 模块纳入 cargo 编译单元（**不**在 `main()` 启动 reporter；reporter 仅测试调用）
- `[V1]` `cargo test` 执行 — 收集 jsonl；所有用例 ID 一一对应 `logos/resources/test/core-S01-test-cases.md` 与 `core-S02-test-cases.md` 规格

> **本节为何追加**：原 `变更概述` 假设 V1 文档经 openlogos merge 后可直接 archive；实际执行 `openlogos verify` 失败（缺 `test-results.jsonl`）。用户明确授权扩大本提案范围以运行 V1 后端已实现代码的 E2E 测试，产出 verify 期望的 jsonl。本节不重新打开 guard；它落在已存在 `logos/.openlogos-guard` 保护的提案范围内。

## 部署影响

- 是否需要部署：否
- 部署原因：本变更主体仅在 `logos/resources/` 目录内新增 Markdown / YAML / SQL / JSON 规格文档，不修改任何运行时代码（`Dockerfile`、CI workflow 不改动）。**例外**：为产生 `verify` 期望的 `test-results.jsonl`，需新增 `backend/src/verify_reporter.rs` + `backend/src/main.rs` 一行 `mod` 注册 + 扩展 `backend/src/diagrams_v1.rs::tests` / `phase3_bridge.rs::tests` 调用 reporter（详见上文"V1 验证测试运行"小节）。**仅运行 `cargo test`**，不启动后端服务。
- 影响环境：无（仅 `cargo test` 在本地跑）
- 是否涉及数据迁移：否
- 是否需要回滚预案：否
- 是否需要 smoke：否

> 部署决策一致性：因 `是否需要部署：否`，`tasks.md` 不创建 `[deploy]` section；`openlogos verify` 通过后下一步直接为 `openlogos archive <slug>`。

## 变更概述

本变更一次性产出 OpenLogos 三层推进模型对应的**V1 + V2 完整基线文档**。

### V1 层（事实锚点，对应 Phase 4 完成态 + drawdb 功能对齐）

记录 Rust + actix-web + SQLite + Leptos 0.x + WASM 的真实结构。**V1 的功能/界面/能力与 drawdb 主分支一致**（对齐参考源见"现状"小节），覆盖：

- **编辑器画布**：表/字段/连线/索引/区域/便签的创建、编辑、删除、移动、撤销/重做；
- **持久化**：diagram 全量数据存 SQLite，11 张表（task / diagram / diagram_link / table / field / table_link / indice / indice_link / reference / area / note），乐观锁 `revision` + 409 冲突语义；
- **桥接导入/导出**：`/api/v1/bridge/*`（SQL 导入/导出、JSON 导入/导出、导入日志、本地重试、bridge 配置）；
- **HTTP REST v1**：`/api/v1/diagrams/*`（POST 创建 / GET 读取 / PUT 全量更新 / DELETE / POST 导入）；
- **4 模块前端**：editor-data-access（HTTP 客户端 + debounce 1s）→ editor-core（状态机）→ editor-panels / editor-render（Leptos signals 细粒度更新）；
- **单实例部署**：本地 dev（cargo run） / Docker（`Dockerfile` + 端口 3000） / GitHub Actions CI。

覆盖 Phase 1（WHY）→ Phase 2（WHAT）→ Phase 3（HOW）→ API/DB/测试/编排/实现清单。规格写作者产出 delta 时，**必须以"对齐参考源"为事实锚点**，不得凭推断或简化。

### 批次划分（方案 A — 单提案分两批 merge）

为避免单次 guard 周期内 35+ 文件的大 PR 风险，本提案采用**两批次交付**：

| 批次 | 范围 | 实际文件数 | 执行时机 |
|---|---|---|---|
| 批次 1（V1 基线） | 全部 V1 delta + 写作母版 | **25** 个 delta + **1** 个 docs/ 母版 | 本提案 `add-baseline-docs` 内 |
| 批次 2（V2 协作规格） | 全部 V2 / SPEC-FUTURE delta | **19** 个 delta | 后续新提案 `add-v2-collab-spec` |

**批次 1 完成并 archive 后**，V1 基线即成为"可用的真相源"，团队可基于此维护 V1 文档或为 V2 评审提供事实锚点。批次 2 的 V2 文档以本批次输出的 V1 文档为前提做增量 Delta，避免 ID / 命名漂移。

### V2 层（SPEC-FUTURE 计划锚点）

记录分支目标 ③「完整用户系统 + 实时 OT 协作」的设计骨架：

- **用户系统**：注册/登录、密码哈希（Argon2id）、Token 刷新、用户资料；
- **协作房间**：创建/加入/邀请/角色（owner / editor / viewer）；
- **OT 协作引擎**：客户端 op 协议（add_table / add_field / add_relationship / delete_xxx / move_xxx）、服务端 `transform(a, b)` 函数、operation log、ack/rev 帧、cursor 帧、撤销/重做策略；
- **新组件拓扑**：`backend`（不变）+ 新增 `collab-server`（OT 引擎）+ WS 网关 + 前端 WASM（扩展为订阅 WS）；
- **多服务部署**：`docker-compose` / staging / 灰度策略（V2 部署方案仅规格，本提案不实际部署）。

V2 文档**显式标注 `[V2 / SPEC-FUTURE]`** 或在每节顶部声明"本节为待实现规格，本提案不实现"。后续 V2 实施将以本基线为起点做增量 Delta；规格与代码的对账通过 `core-implementation-checklist.md` 维护（V1 行 `[x]`，V2 行 `[ ]`）。

在 delta 阶段将按 `tasks.md` 的 `[delta]` section 任务清单逐项产出 delta 文件至 `deltas/prd/`、`deltas/api/`、`deltas/database/`、`deltas/scenario/` 等子目录（与 `logos/resources/` 目录一一对应），merge 时由 `openlogos merge` 统一合并到主文档。

## 文档语言

- 所有新增文档的语言：**中文**（与 `logos.config.json` → `locale: "zh"` 一致）。
- 文档中的技术名词、API 路径、字段名、文件路径、命令保持英文。
- 文档内的代码片段、SQL、YAML、JSON 保持英文（标识符 / 字面量）。
