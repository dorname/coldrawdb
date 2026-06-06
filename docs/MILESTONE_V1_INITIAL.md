# drawDB 初版里程碑（V1）文档总览

> 本文档用于汇总当前里程碑范围内的核心产出，减少跨阶段重复文档查找成本。

## 1. 里程碑范围

- Phase 0：方案冻结（ERD / OpenAPI 草案 / 迁移策略）
- Phase 1：数据库落地（migration / init / repository）
- Phase 2：API 完整化（diagrams v1 CRUD + import）
- Phase 3：迁移桥接（bridge 开关、本地草稿导入、日志、重试）

当前状态：**初版里程碑已达成（后端范围）**。

---

## 2. 核心产出索引

### 2.1 方案与设计
- `RUST_WEB_REFACTOR_PLAN.md`
- `docs/phase0/ERD.md`
- `docs/phase0/openapi-v1-draft.yaml`
- `docs/phase0/MIGRATION_STRATEGY.md`
- `docs/phase0/EXECUTION_PLAN.md`

### 2.2 数据库与迁移
- `backend/migrations/0001_phase1_schema.up.sql`
- `backend/migrations/0001_phase1_schema.down.sql`
- `backend/migrations/0002_phase3_bridge.up.sql`
- `backend/migrations/0002_phase3_bridge.down.sql`
- `backend/src/init.rs`

### 2.3 API 与桥接
- `backend/src/diagrams_v1.rs`
- `backend/src/phase3_bridge.rs`
- `backend/src/main.rs`
- `backend/src/repository/diagram_repository.rs`

### 2.4 校验文档
- `docs/phase1/PHASE1_VALIDATION.md`
- `docs/phase2/PHASE2_VALIDATION.md`
- `docs/phase3/PHASE3_VALIDATION.md`

---

## 3. 已完成能力（摘要）

### Phase 1
- 数据库 schema 升级与迁移幂等执行。
- 启动时基线初始化 + migration 自动应用。

### Phase 2
- `POST/GET/PUT/DELETE /api/v1/diagrams`
- `POST /api/v1/diagrams/import`
- revision 冲突语义（409）与基础容错。

### Phase 3
- `GET/PUT /api/v1/bridge/config`
- `POST /api/v1/bridge/import/local`
- `GET /api/v1/bridge/import/local/logs`
- `POST /api/v1/bridge/import/local/retry/{id}`

---

## 4. 文档清理说明（本次）

为避免重复维护，以下信息已合并到本总览与验证文档：
- Phase 2 实施状态与 OpenAPI 对齐说明
- Phase 3 实施状态说明

保留原则：
- **保留可验收文档**（`PHASE*_VALIDATION.md`）
- **保留计划主文档**（`EXECUTION_PLAN.md`）
- **删除重复状态文档**（仅描述性、与验证文档重复）

---

## 5. Phase 4 验收状态

> 收官报告：`docs/phase4/PHASE4_DONE.md`；执行计划：`.omc/plans/phase4-rust-web-mvp.md`；
> spec：`.omc/specs/deep-interview-phase4-rust-web-mvp.md`。

### 5.1 框架选型最终结果

**Leptos（85/100）** — 4 维评分（编辑器交互 40 / 性能 25 / 工程可维护性 20 /
团队学习成本 15）击败 Dioxus (73) / Yew (68)。详见
`docs/phase4/framework-poc/SCORECARD.md`。

### 5.2 模块架构（1 crate + 4 modules）

`frontend-rs/` crate 含 `editor_core` / `editor_render` / `editor_panels` /
`editor_data_access` 4 modules；依赖单向无环（`editor_core` 不反向 import）；
CI 用 ast-grep + cargo-modules 双重 gate。架构图 `docs/phase4/architecture.mmd` /
`.svg` 强制渲染。

### 5.3 §8 性能 / 稳定性

- `GET /api/v1/diagrams/{id}` P95 = **1.9ms**（阈值 300ms）
- `PUT /api/v1/diagrams/{id}` P95 = **1.0ms**（阈值 500ms）
- 4h soak 脚本就绪 + 失败恢复策略实现；完整 4h 跑 deferred 至 CI runner
- 15 E2E spec / 26 test cases / 5×5 覆盖矩阵 25/25 全绿

### 5.4 React 完全下线

W4-2 在 `feature/phase4-react-removal` 分支内分 5 个独立 commit 完成 React
下线；永久 tag `phase4-pre-react-removal` 指向回退锚点；功能回归
（email / gists / import 客户端）已在 `docs/phase4/CHANGELOG-react-removal.md`
显式声明并申报 Phase 5。

### 5.5 验证清单

- AC 自动验证：`docs/phase4/PHASE4_VALIDATION.md`（22/25 全绿；3 项 ◐ = baseline
  闭环 + rust 侧待 CI runner 补齐）

---

## 6. 下一步

- 进入 Phase 5：切流与收尾；移交清单见 `docs/phase4/PHASE4_DONE.md` §6
- 在 Phase 5 启动时 spec 化 mvp-advanced-features（模板 / 主题 / 导出 SQL / email /
  gists / import 客户端）
