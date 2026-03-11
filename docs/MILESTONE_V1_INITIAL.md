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
- `backend/migrations/0003_frontend_integration.up.sql`（diagram 表新列 + template 表）
- `backend/src/init.rs`

### 2.3 API 与桥接
- `backend/src/diagrams_v1.rs`
- `backend/src/phase3_bridge.rs`
- `backend/src/templates/mod.rs`（模板 CRUD API）
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

### 前端集成（迁移 0003）
- diagram 表新增列：gist_id、loaded_from_gist_id、tables_json、references_json、notes_json、areas_json、tasks_json、enums_json、types_json
- template 表与 `/templates` Legacy 模板 API

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

## 5. 下一步

- 进入 Phase 4：Rust Web MVP（PoC 选型 + 最小可用编辑链路）
- 在 Phase 5 前补齐灰度发布与双写窗口实操演练文档
