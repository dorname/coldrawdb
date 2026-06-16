# 变更提案：add-baseline-docs（第三次迭代）

> module: core | created: 2026-06-16
> 提案序号：第三次以 `add-baseline-docs` 为 slug（历史：20260608-1824 重型 / 20260612-1710 轻型，均已 archive）
> 前置：redesign-phase-a/b/c/d/e 五个提案已全部归档

## 变更原因

本项目已完成基线建立（20260608 archive，26 个 delta 文件覆盖 Phase 1/2/3 全链路）和轻量刷新（20260612 archive，仅 README/AGENTS/reference）。但自 20260612 之后又相继归档了 **redesign-phase-a/b/c/d/e** 五个 V2 重构提案，引入 7 个新规格文件与全新 UI 架构。本次复核基线时发现两类遗留问题：

### 1. 残留 delta 标记（merge 流程缺陷）

`logos/resources/` 下 **19 个主文档** 顶部仍残留 `## ADDED — ...` / `## MODIFIED — ...` 块，并指向 `add-baseline-docs` 提案（包括 PR 第 1 阶段、第 2 阶段、第 3 阶段、API、DB、Implementation、Scenario、Test 共 19 个文件）。`openlogos merge` 应在合并完成后剥离这些标记，但历史执行显然未剥离，导致当前主文档可读性受损且未来变更管理工具可能误判。

> 受影响文件清单：
> - prd/1-product-requirements/：`core-00-scenario-overview.md`、`core-01-requirements.md`
> - prd/2-product-design/1-feature-specs/`core-00-information-architecture.md`、`core-01-editor-canvas.md`、`core-01a-table-and-field.md`、`core-01b-relationship.md`、`core-01c-index-enum-custom-type.md`、`core-01d-import-export.md`、`core-02-diagram-persistence.md`、`core-03-bridge-io.md`、`core-04-side-panel-tabs.md`、`core-05-top-menu-modals.md`、`core-07-design-tokens.md`、`core-08-icon-library.md`、`core-09-core-components.md`、`core-0a-code-editor.md`、`core-0b-dark-mode.md`、`core-0c-motion.md`
> - prd/3-technical-plan/1-architecture/`core-01-architecture-overview.md`
> - prd/3-technical-plan/2-scenario-implementation/`core-S01-edit-and-save-diagram.md`、`core-S02-load-shared-diagram.md`
> - prd/3-technical-plan/3-deployment/`core-01-deployment-plan.md`
> - api/`bridge.yaml`、`diagrams.yaml`
> - database/`coldrawdb-v1.sql`
> - implementation/`core-implementation-checklist.md`
> - scenario/`core-S01-diagram-save.json`、`core-S02-shared-link-load.json`
> - test/`core-S01-test-cases.md`、`core-S02-test-cases.md`、`test/smoke/core-smoke-test-cases.md`
>
> 注：实际命中 19 个文本类文件（sql/yaml/json 顶部若只是元数据说明，则需按文件类型决定剥离策略，本提案对 sql/yaml/json 类只剥离文件开头的元数据注释头，不重写正文）。

### 2. 基线内容未跟随 redesign phases A-E 刷新

`core-baseline-reference.md`、`core-01-architecture-overview.md`、`core-00-scenario-overview.md`、`core-01-requirements.md`、`README.md`、`AGENTS.md` 这 6 个核心基线文档仍停留在 V1 早期状态：

- **未提及** redesign phases 引入的 7 个新规格：`core-01d-import-export.md`、`core-07-design-tokens.md`、`core-08-icon-library.md`、`core-09-core-components.md`、`core-0a-code-editor.md`、`core-0b-dark-mode.md`、`core-0c-motion.md`
- **未提及** 新 UI 架构层：AppBar + ToolRail + Inspector + IO 抽屉（Phase A/B/C 落地）
- **未提及** 设计系统层：Semi Design token 体系 + icon 库 + 8 类核心组件 + Monaco 集成 + 暗色模式 + 动效（Phase D/E 落地）
- **未同步** 5 个 redesign phase 的归档索引（README/AGENTS 中仅列到 20260612）

> grep 验证：`core-baseline-reference.md` / `core-01-architecture-overview.md` / `core-00-scenario-overview.md` / `core-01-requirements.md` 中均无 `core-01d` / `core-07` / `core-08` / `core-09` / `core-0a` / `core-0b` / `core-0c` 引用。

## 变更类型

**设计级**（基线文档刷新 + 既有规格 meta 清理）

## 变更范围

### 影响的需求文档

- `logos/resources/prd/1-product-requirements/core-00-scenario-overview.md` — MODIFIED：补充 S01/S02 与新增 7 个功能规格的引用映射
- `logos/resources/prd/1-product-requirements/core-01-requirements.md` — MODIFIED：补充 NFR（Monaco WASM 体积、设计 token、暗色模式、动效）+ 顶部 delta 标记剥离

### 影响的功能规格（仅顶部 meta 清理）

> Phase A/B/C/D/E 已在各自提案中完成功能规格的实质更新，本次只清理顶部 delta 标记，不再重写正文。

- `logos/resources/prd/2-product-design/1-feature-specs/` 下 16 个文件 — MODIFIED：仅剥离顶部 delta 标记

### 影响的业务场景

- `logos/resources/prd/3-technical-plan/2-scenario-implementation/core-S01-edit-and-save-diagram.md` — MODIFIED：剥离顶部 delta 标记
- `logos/resources/prd/3-technical-plan/2-scenario-implementation/core-S02-load-shared-diagram.md` — MODIFIED：剥离顶部 delta 标记

### 影响的部署方案

- `logos/resources/prd/3-technical-plan/3-deployment/core-01-deployment-plan.md` — MODIFIED：剥离顶部 delta 标记（如正文需补充 Monaco lazy-load 与浏览器缓存策略说明，列入本次 [delta] 任务项 D-04）

### 影响的架构文档

- `logos/resources/prd/3-technical-plan/1-architecture/core-01-architecture-overview.md` — MODIFIED：补充 V2 布局层（AppBar + ToolRail + Inspector + IO Drawer）+ 设计系统层（Tokens + Icons + Components + Monaco）+ 顶部 delta 标记剥离

### 影响的 API

- `logos/resources/api/bridge.yaml`、`diagrams.yaml` — MODIFIED：仅剥离顶部元数据注释（保留所有 endpoints 定义不变）

### 影响的 DB

- `logos/resources/database/coldrawdb-v1.sql` — MODIFIED：仅剥离顶部元数据注释（保留所有 DDL 不变）

### 影响的编排测试

- `logos/resources/scenario/core-S01-diagram-save.json`、`core-S02-shared-link-load.json` — MODIFIED：仅剥离顶部元数据（保留 steps 体不变）

### 影响的 smoke 测试

- `logos/resources/test/core-S01-test-cases.md`、`core-S02-test-cases.md`、`test/smoke/core-smoke-test-cases.md` — MODIFIED：仅剥离顶部 delta 标记

### 影响的基线参考

- `logos/resources/reference/core-baseline-reference.md` — MODIFIED：
  - §2 模块清单：补充 AppBar / ToolRail / Inspector / IO Drawer 模块（redesign-phase-a/b/c）
  - §2 补充设计系统条目：design tokens / icon library / core components / Monaco CodeEditor / dark mode / motion（redesign-phase-d/e）
  - §3 开发环境速查：补充 Monaco 浏览器缓存提示
  - §4 常用 CLI：补充 redesign phases 归档索引链接

### 影响的仓库根文档

- `README.md` — MODIFIED：补充 5 个 redesign phase 归档索引 + 当前技术栈摘要（Semi Design tokens + Leptos 0.5）
- `AGENTS.md` — MODIFIED：补充 5 个 redesign phase 归档索引 + `core` 模块当前 phase 状态

### 影响的实现清单

- `logos/resources/implementation/core-implementation-checklist.md` — MODIFIED：仅剥离顶部 delta 标记（如需补充 redesign phases E1-E6 实现条目，由 redesign-phase-e 单独跟踪，本提案不重写）

## 部署影响

- 是否需要部署：**否**
- 部署原因：本次纯文档刷新（清理残留 delta 标记 + 基线内容补充），不修改任何运行时代码、API、DB schema 或部署脚本
- 影响环境：**无**
- 是否涉及数据迁移：**否**
- 是否需要回滚预案：**否**（文档变更可一次性 git revert）
- 是否需要 smoke：**否**

## 变更概述

本次为 `add-baseline-docs` 第三次迭代，目标是修复前两轮遗留的两类问题：

1. **清理残留**：将 19 个 `logos/resources/` 主文档顶部的 `## ADDED — ...` / `## MODIFIED — ...` / `## REMOVED — ...` delta 标记块一次性剥离，恢复主文档的可读性与变更管理工具的正确性判断。
2. **刷新基线**：在 `core-baseline-reference.md` / `core-01-architecture-overview.md` / `core-00-scenario-overview.md` / `core-01-requirements.md` / `README.md` / `AGENTS.md` 中同步引用 redesign phases A-E 引入的 7 个新规格、新 UI 架构层（AppBar/ToolRail/Inspector/Drawer）、设计系统层（Tokens/Icons/Components/Monaco/Dark/Motion）以及 5 个归档索引，让基线文档真实反映当前项目形态。

本提案为纯 delta 变更，不涉及代码或部署。merge 完成后可立即 `openlogos archive <slug>`。