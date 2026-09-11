# 合并指令

## 变更提案
- 提案名称：v1-register-scenario-docs
- 提案目录：logos/changes/v1-register-scenario-docs/

## 提案内容

# 变更提案：v1-register-scenario-docs

> module: core | created: 2026-06-19

## 变更原因

用户运行 `openlogos status` 时观察到 `core` 模块 Phase 3 Step 1（场景建模）被标记为「missing S01 S02 S03 S04 S05」。磁盘核查后发现实际状态与检测结论之间存在系统性偏差：

1. **S01 / S02 时序图已存在**：`logos/resources/prd/3-technical-plan/2-scenario-implementation/core-S01-edit-and-save-diagram.md`（222 行，含完整 Mermaid 时序图 + 步骤详解 + 错误处理 + 性能资源 + 测试映射 + V1 边界 + 对齐参考源 8 章）与 `core-S02-load-shared-diagram.md`（270 行，结构对齐）。
2. **S01 / S02 编排测试 JSON 已存在**：`logos/resources/scenario/core-S01-diagram-save.json`（7 步 POST/PUT/GET/DELETE 链路）与 `core-S02-shared-link-load.json`。
3. **Phase 1 业务场景总览已存在**：`logos/resources/prd/1-product-requirements/core-00-scenario-overview.md`（含场景索引、场景图谱、文档映射）。
4. **Phase 3 技术实现概览缺失**：按 `scenario-architect` SKILL §Step 5 输出规范，需要一份位于 `prd/3-technical-plan/2-scenario-implementation/core-00-scenario-overview.md` 的「技术实现状态总览」，目前缺失。
5. **`logos-project.yaml` 的 `resource_index` 未登记上述 4 份场景建模文档** + 1 份待新增的技术实现概览，导致 OpenLogos CLI 把 S01/S02 也误报为「missing」。

S03 / S04 / S05 在 `logos-project.yaml` 的 `scenarios` 字段中明确标记为 `status: planned, version: V2`，且 `core` 模块配置 `skip_phases: [api, database, scenario]`，架构 § 9 V1 边界列出「❌ OT 实时协作（V2 计划）」「❌ WebSocket（V1 全 HTTP）」——V1 不应建模 S03–S05，本提案维持该边界。

本次变更旨在**让 `resource_index` 与 OpenLogos 检测结果对齐 V1 实际状态**，并补齐 scenario-architect SKILL 要求的 Phase 3 技术实现概览文档，不引入任何代码、API、DB、部署变更。

## 变更类型

**设计级变更（Documentation / Specification Only）**

- 不涉及代码（`src/`、`backend/`、`frontend-rs/` 均不动）
- 不涉及 API 契约变更
- 不涉及 DB Schema 变更
- 不涉及部署变更
- 仅修改 `logos/resources/` 下的 Markdown 与 `logos/logos-project.yaml` 元数据

## 变更范围

### 新增文档（1 份）

- `logos/resources/prd/3-technical-plan/2-scenario-implementation/core-00-scenario-overview.md`（ADDED）— Phase 3 技术实现状态总览，含场景地图（V1 ✅ / V2 deferred）、场景依赖关系、场景索引（技术实现维度）、与 Phase 1 总览的引用关系；按 `scenario-architect` SKILL §Step 5 模板产出

### 修改文档（1 份 yaml）

- `logos/logos-project.yaml`（MODIFIED）— 在 `resource_index` 末尾追加 5 条登记：
  1. Phase 3 场景实现概览（新增）
  2. `core-S01-edit-and-save-diagram.md`（时序图）
  3. `core-S02-load-shared-diagram.md`（时序图）
  4. `core-S01-diagram-save.json`（编排测试）
  5. `core-S02-shared-link-load.json`（编排测试）

### 不变更文档

- `logos/resources/prd/1-product-requirements/core-00-scenario-overview.md` —— Phase 1 业务总览已存在且内容完整（含文档映射表），保持不动
- `logos/resources/prd/3-technical-plan/2-scenario-implementation/core-S01-edit-and-save-diagram.md` —— S01 时序图已存在，保持不动
- `logos/resources/prd/3-technical-plan/2-scenario-implementation/core-S02-load-shared-diagram.md` —— S02 时序图已存在，保持不动
- `logos/resources/scenario/core-S01-diagram-save.json` / `core-S02-shared-link-load.json` —— 编排测试已存在，保持不动
- `core-03-pain-points.md` / `core-01-requirements.md` 等 Phase 1 文档 —— 均不动
- `core-01-architecture-overview.md` —— 不动

### 影响的其他维度

- 影响的需求文档：**无变更**
- 影响的功能规格：**无变更**
- 影响的业务场景：**无场景 ID 变更**（不新增、不废弃场景编号；S03/S04/S05 维持 V2 planned）
- 影响的部署方案：**无**
- 影响的 API：**无**
- 影响的 DB 表：**无**
- 影响的编排测试：**无新增/修改**（仅在 yaml 中追加登记引用）
- 影响的 smoke 测试：**无**

## 部署影响

- 是否需要部署：**否**
- 部署原因：本次变更仅修改 `logos/resources/` 下的 Markdown 与 `logos/logos-project.yaml` 元数据，不涉及任何运行时行为变更，无需重新构建或部署
- 影响环境：**无**
- 是否涉及数据迁移：**否**
- 是否需要回滚预案：**否**（spec 文档变更可通过 git revert 回滚；本次不影响运行时）
- 是否需要 smoke：**否**

## 变更概述

本次变更产出 **2 个 delta 文件**，按 skill 规范的 `## ADDED —` / `## MODIFIED —` 标记分块：

### delta 1（新增）：`deltas/prd/3-technical-plan/2-scenario-implementation/core-00-scenario-overview.md`

按 `scenario-architect` SKILL §Step 5 输出规范，产出 Phase 3 技术实现状态总览：

- **场景地图**（表格形式）：S01 ✅ / S02 ✅ / S03-S05 ❌ V2 deferred，每行标注 Phase 1 / Phase 2 / Phase 3 时序图 / API / 编排 / 状态
- **场景依赖关系**：S01 与 S02 无前置；S03–S05 之间存在鉴权 → 房间 → OT 的链式依赖，但 V1 不实现
- **场景索引**（技术维度）：每个 V1 场景一行，标注时序图路径、编排 JSON 路径、相关功能规格、相关后端子模块
- **与 Phase 1 总览的引用关系**：明确指向 `prd/1-product-requirements/core-00-scenario-overview.md`，避免两份概览被误读为冲突

### delta 2（修改）：`deltas/logos-project.yaml`

- 保留现有 5 条 `resource_index`（Phase 1 文档索引）
- 在末尾追加 5 条新登记（Phase 3 场景实现概览 + S01/S02 时序图 + S01/S02 编排 JSON）
- 每条带 `desc` 一句话说明使用场景，便于 AI Agent 发现

### 设计哲学

- **不重写已有内容**：S01/S02 时序图与 Phase 1 总览质量良好，仅补齐缺失的 Phase 3 概览 + yaml 登记
- **遵守 SKILL 输出规范**：Phase 3 概览严格按 `scenario-architect` SKILL §Step 5 模板（场景地图 / 依赖关系 / 场景索引）
- **YAML 字段对齐 SKILL**：参照 `architecture-designer` SKILL §6 的 `resource_index` 字段约定（每条含 `path` + `desc`）
- **明确 V1/V2 边界**：在 Phase 3 概览中显式标注 S03–S05 为「V2 deferred」，与 `logos-project.yaml` 的 `scenarios` 字段保持一致
- **不做范围扩大**：不补 S03/S04/S05 时序图，避免违反 `core.skip_phases` 与 V1 边界

## 需要合并的 Delta 文件

### 1. deltas/prd/3-technical-plan/2-scenario-implementation/core-00-scenario-overview.md

- Delta 文件：`logos/changes/v1-register-scenario-docs/deltas/prd/3-technical-plan/2-scenario-implementation/core-00-scenario-overview.md`
- 目标目录：`logos/resources/prd/3-technical-plan/2-scenario-implementation/`
- 操作：读取 delta 中的 ADDED / MODIFIED / REMOVED 标记，合并到目标目录中对应的主文档

## 执行要求

1. 逐个 Delta 文件处理，每处理完一个报告修改摘要
2. 对于 ADDED 标记：在主文档的指定位置插入新内容
3. 对于 MODIFIED 标记：替换主文档中同名章节的内容
4. 对于 REMOVED 标记：从主文档中删除对应章节
5. 保持主文档的原有格式和风格
6. 如果主文档有"最后更新"时间戳，同步更新
7. 所有变更完成后，列出修改清单
8. 所有变更合并完成后，自动执行 git commit（告知用户，无需确认）：
   git add -A && git commit -m "docs(v1-register-scenario-docs): merge spec deltas"
   然后提示用户：按更新后的规格实现代码，代码完成后运行 `openlogos verify` 验收，验收通过后明确授权执行 `openlogos archive v1-register-scenario-docs`。
