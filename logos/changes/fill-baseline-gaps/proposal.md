# 变更提案：fill-baseline-gaps

> module: core | created: 2026-06-19

## 变更原因

用户运行 `openlogos status` 时观察到 `core` 模块的 Phase 1（需求）/ Phase 2（设计）/ Phase 3-0（架构）三阶段被标记为「⏭️ 文档基线已跳过（存量项目接入）」，磁盘调查后发现物理文档实际已存在：

- Phase 1：`core-00-scenario-overview.md`（6.6 KB）+ `core-01-requirements.md`（5.1 KB，含 US/FR/NFR 清单）
- Phase 2：`1-feature-specs/` 17 份规格文件 + `2-page-design/` 1 份 HTML 原型（合计 ~150 KB）
- Phase 3-0：`core-01-architecture-overview.md`（13.7 KB）

但对照 `logos/skills/prd-writer/`、`product-designer/`、`architecture-designer/` 三份 Skill 的"应有结构"，现有文档存在以下缺口：

1. Phase 1 缺少**产品背景与目标**、**用户痛点因果链**、**场景总览表**、**G/W/T 验收条件**
2. Phase 3-0 技术选型表**缺少"选型理由 + 备选方案"列**、**没有 Mermaid 系统架构图**、**缺非功能约束 / 外部依赖章节**
3. `logos/logos-project.yaml` 的 `tech_stack: {}` 为空、`scenarios` / `resource_index` / `external_dependencies` 字段完全缺失，导致后续 AI 无法读取架构决策与场景列表

本次变更旨在**补齐符合 OpenLogos Skill 规范的缺失章节**，让 `openlogos status` 与 `logos-project.yaml` 能正确反映"文档已就位"的事实，同时为后续 AI Agent 提供完整的技术栈与场景清单索引。

## 变更类型

**设计级变更**（Documentation / Specification Only）

- 不涉及代码（src/、backend/、frontend-rs/ 均不动）
- 不涉及 API 契约变更
- 不涉及 DB Schema 变更
- 不涉及部署变更

## 变更范围

- 影响的需求文档：
  - `logos/resources/prd/1-product-requirements/core-01-requirements.md`（MODIFIED：拆分"约束与边界"为 3 小节 + 新增"场景总览"小节）
  - 新增 `logos/resources/prd/1-product-requirements/core-02-product-vision.md`（产品背景 / 定位 / 用户画像 / 成功指标）
  - 新增 `logos/resources/prd/1-product-requirements/core-03-pain-points.md`（P01/P02 因果链 + 不做清单）
  - 新增 `logos/resources/prd/1-product-requirements/core-04-scenario-detail.md`（S01/S02 GIVEN/WHEN/THEN 详述）
- 影响的功能规格：**无**（Phase 2 现有 17 份规格已足够，无需修改）
- 影响的业务场景：**无场景 ID 变更**（不新增、不废弃场景编号）
- 影响的技术架构：
  - `logos/resources/prd/3-technical-plan/1-architecture/core-01-architecture-overview.md`（MODIFIED：Mermaid 图 + 选型表补理由列 + 非功能约束 + 外部依赖）
- 影响的 `logos-project.yaml`：
  - `tech_stack: {}` → 填入 11 项技术选型
  - 新增 `scenarios` 字段（S01~S05）
  - 新增 `external_dependencies` 字段（明示 V1 空）
  - 新增 `resource_index` 字段（索引全部 baseline 文档）
- 影响的 API：**无**
- 影响的 DB 表：**无**
- 影响的编排测试：**无**
- 影响的 smoke 测试：**无**

## 部署影响

- 是否需要部署：**否**
- 部署原因：本次变更仅修改 `logos/resources/` 下的 Markdown 与 `logos-project.yaml` 元数据，不涉及任何运行时行为变更，无需重新构建或部署
- 影响环境：**无**
- 是否涉及数据迁移：**否**
- 是否需要回滚预案：**否**
- 是否需要 smoke：**否**

## 变更概述

本次变更产出 **8 个 delta 文件**，按 Skill 规范的 ADDED / MODIFIED 标记分块：

### 新增（4 份文档）

1. `core-02-product-vision.md` — Phase 1 第一节"产品背景与目标"，含一句话定位、目标用户画像（数据库设计者 / 个人开发者画像）、成功指标（drawdb 能力对齐度 ≥ 95%、P95 < 200ms 等）
2. `core-03-pain-points.md` — Phase 1 第二节"用户痛点分析"，4 条因果链（P01 缺少自托管 / P2 协作成本高 / P03 现有工具无后端 / P04 数据迁移门槛高）
3. `core-04-scenario-detail.md` — Phase 1 第四节"核心场景详述"，S01 编辑保存 / S02 分享链接加载 两个 V1 场景的 GIVEN/WHEN/THEN 验收（每场景 ≥1 正常 + ≥1 异常）

### 修改（3 份主文档 + 1 份 yaml）

4. `core-01-requirements.md`（MODIFIED）— 拆分原"5. 范围边界（V1 不做）"为标准"约束与边界"三小节（技术 / 资源 / 不做），并在 FR 表前新增"场景总览"小节
5. `core-01-architecture-overview.md`（MODIFIED）— 在 §1 后插入 Mermaid 系统架构图；§6 选型表重写为标准三列（选型 / 理由 / 备选方案）；新增 §11 非功能性约束、§12 外部依赖与测试策略
6. `logos-project.yaml`（MODIFIED）— 填入 `tech_stack` 11 项；新增 `scenarios` 5 条；新增 `external_dependencies`（明示空）；新增 `resource_index` 18 条索引

### 设计哲学

- **不重写已有内容**：现有 150 KB 文档质量良好，仅在缺失章节处增量补充，不破坏既有结构
- **遵守 Skill 输出规范**：新增文档严格按 `prd-writer` SKILL 模板（产品背景 / 痛点 / 场景 / 约束四段式）
- **YAML 字段对齐 Skill**：参照 `architecture-designer` SKILL §6 的 `tech_stack` / `external_dependencies` / `scenarios` 字段模板
- **不做范围扩大**：Phase 2 的 17 份规格本次不动，避免无谓的 churn