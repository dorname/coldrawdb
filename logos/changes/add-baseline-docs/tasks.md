# 实现任务

## [delta] 规格变更

### D-01 顶层基线文档刷新（4 文件，资源内）

- [x] 产出 delta 到 `deltas/prd/1-product-requirements/core-01-requirements.md` — 顶部 delta 标记剥离 + NFR 章节补充（Monaco 体积、设计 token、暗色模式、动效）
- [x] 产出 delta 到 `deltas/prd/1-product-requirements/core-00-scenario-overview.md` — 顶部 delta 标记剥离 + 场景 ↔ 文档映射表补充 7 个新规格引用
- [x] 产出 delta 到 `deltas/prd/3-technical-plan/1-architecture/core-01-architecture-overview.md` — 顶部 delta 标记剥离 + §2.5/§2.6 补充 V2 布局层与设计系统层
- [x] 产出 delta 到 `deltas/reference/core-baseline-reference.md` — §2.1.x 模块清单补充 AppBar/ToolRail/Inspector/IO Drawer 与设计系统条目；§3.1.1 补充 Monaco 缓存提示；§4.1 补充 redesign phases 归档索引

### D-02 功能规格顶部 meta 清理（16 文件）

> Phase A/B/C/D/E 已在各自提案中完成功能规格实质更新，本次只清理顶部 delta 标记，不再重写正文。

- [x] 产出 delta 到 `deltas/prd/2-product-design/1-feature-specs/core-00-information-architecture.md`
- [x] 产出 delta 到 `deltas/prd/2-product-design/1-feature-specs/core-01-editor-canvas.md`
- [x] 产出 delta 到 `deltas/prd/2-product-design/1-feature-specs/core-01a-table-and-field.md`
- [x] 产出 delta 到 `deltas/prd/2-product-design/1-feature-specs/core-01b-relationship.md`
- [x] 产出 delta 到 `deltas/prd/2-product-design/1-feature-specs/core-01c-index-enum-custom-type.md`
- [x] 产出 delta 到 `deltas/prd/2-product-design/1-feature-specs/core-01d-import-export.md`
- [x] 产出 delta 到 `deltas/prd/2-product-design/1-feature-specs/core-02-diagram-persistence.md`
- [x] 产出 delta 到 `deltas/prd/2-product-design/1-feature-specs/core-03-bridge-io.md`
- [x] 产出 delta 到 `deltas/prd/2-product-design/1-feature-specs/core-04-side-panel-tabs.md`
- [x] 产出 delta 到 `deltas/prd/2-product-design/1-feature-specs/core-05-top-menu-modals.md`
- [x] 产出 delta 到 `deltas/prd/2-product-design/1-feature-specs/core-07-design-tokens.md`
- [x] 产出 delta 到 `deltas/prd/2-product-design/1-feature-specs/core-08-icon-library.md`
- [x] 产出 delta 到 `deltas/prd/2-product-design/1-feature-specs/core-09-core-components.md`
- [x] 产出 delta 到 `deltas/prd/2-product-design/1-feature-specs/core-0a-code-editor.md`
- [x] 产出 delta 到 `deltas/prd/2-product-design/1-feature-specs/core-0b-dark-mode.md`
- [x] 产出 delta 到 `deltas/prd/2-product-design/1-feature-specs/core-0c-motion.md`

### D-03 场景与测试顶部 meta 清理（5 文件）

- [x] 产出 delta 到 `deltas/prd/3-technical-plan/2-scenario-implementation/core-S01-edit-and-save-diagram.md`
- [x] 产出 delta 到 `deltas/prd/3-technical-plan/2-scenario-implementation/core-S02-load-shared-diagram.md`
- [x] 产出 delta 到 `deltas/test/core-S01-test-cases.md`
- [x] 产出 delta 到 `deltas/test/core-S02-test-cases.md`
- [x] 产出 delta 到 `deltas/test/smoke/core-smoke-test-cases.md`

### D-04 部署方案与实现清单顶部 meta 清理（2 文件）

- [x] 产出 delta 到 `deltas/prd/3-technical-plan/3-deployment/core-01-deployment-plan.md` — 顶部 delta 标记剥离 + §3.4.6 补充 Monaco lazy-load 与浏览器缓存策略说明
- [x] 产出 delta 到 `deltas/implementation/core-implementation-checklist.md` — 仅顶部 delta 标记剥离（E1-E6 实现条目由 redesign-phase-e 单独跟踪，本提案不重写）

### D-05 API / DB / Scenario 顶部 meta 清理（5 文件）

- [x] 产出 delta 到 `deltas/api/bridge.yaml` — 仅顶部元数据注释剥离（保留全部 endpoints 定义不变）
- [x] 产出 delta 到 `deltas/api/diagrams.yaml` — 仅顶部元数据注释剥离
- [x] 产出 delta 到 `deltas/database/coldrawdb-v1.sql` — 仅顶部元数据注释剥离（保留全部 DDL 不变）
- [x] 产出 delta 到 `deltas/scenario/core-S01-diagram-save.json` — 仅顶部元数据剥离（保留 steps 体不变）
- [x] 产出 delta 到 `deltas/scenario/core-S02-shared-link-load.json` — 仅顶部元数据剥离

### D-06 delta 任务总计

共 **32 个 delta 文件** 全部产出，分布在 5 个目标子目录（prd、api、database、scenario、test、implementation、reference）。

> **生成工具**：
> - 27 个机械剥离型（28 个中的 27，1 个 deployment 含 Monaco 实质内容）由 `scripts/build-deltas.py` 生成（支持 md-a / md-b / md-c / md-d / yaml / sql / json 6 种 pattern）
> - 5 个含实质内容的 delta（D-01 4 + D-04 deployment）由 `scripts/build-substantive-deltas.py` 生成

## [code] 代码实现（项目级文档直接修改，**SPEC_MERGED 后执行**）

> README.md / AGENTS.md 不在 `logos/resources/` 目录下，按 OpenLogos 规范属于项目级源文档，**不走 delta 流程**，直接修改源文件（与 `add-baseline-docs` 20260612 archive 中 [code] section 的处理方式一致）。`openlogos merge add-baseline-docs` 执行并写回 SPEC_MERGED 后才能开始。

- [ ] 修改仓库根 `README.md` — 补充 5 个 redesign phase 归档索引 + 当前技术栈摘要（Semi Design tokens + Leptos 0.5 + WASM）
- [ ] 修改仓库根 `AGENTS.md` — 补充 5 个 redesign phase 归档索引 + `core` 模块当前 phase 状态（`lifecycle: launched`）