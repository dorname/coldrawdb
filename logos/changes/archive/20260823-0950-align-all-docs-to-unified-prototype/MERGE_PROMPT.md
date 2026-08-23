# 合并指令

## 变更提案
- 提案名称：align-all-docs-to-unified-prototype
- 提案目录：logos/changes/align-all-docs-to-unified-prototype/

## 提案内容

# 变更提案：align-all-docs-to-unified-prototype

> module: core | created: 2026-08-22

## 变更原因

现行唯一视觉与交互基线 `core-01-editor-prototype.html` 已串联 S01～S05 的 Auth、Rooms、Invite、协作编辑器、画布建模、导入导出、命令/代码视图、权限与断线恢复体验；但现有需求、功能规格、技术场景、测试矩阵和实现清单是在多轮独立变更中增量形成，仍混有以下风险：

1. 部分文档继续使用旧 Landing / 编辑器直达 / 独立 S03～S05 原型等历史叙述，与现行 `auth → rooms → room-editor` 页面流不一致。
2. 页面锚点、状态文案、权限、响应式、浮层、保存/协作状态与主原型之间缺少一份完整的端到端追溯基线。
3. 既有实现清单曾标记“已对齐”，但不足以证明生产前端逐项达到主原型的结构、视觉和交互要求，也不能直接作为下一轮前后端实现的任务合同。

本变更先完成 Why → What → How 的文档收口：以主原型为事实输入，审计并对齐 S01～S05 所有受影响文档，为后续独立代码变更 `implement-unified-prototype-spec-parity` 建立唯一、可测试的实现基线。

## 变更类型

需求级纯规格变更。覆盖需求、产品设计、技术场景、测试与验收追溯；本提案不修改生产代码，不改变主原型，不执行部署。

## 变更范围

- 基准原型：
  - `logos/resources/prd/2-product-design/2-page-design/core-01-editor-prototype.html`（只读事实基线，不在本提案中修改）
  - `core-03-auth-prototype.html`、`core-04-collab-prototype.html`、`core-05-ot-collab-prototype.html` 继续保持历史参考身份，不作为验收入口
- 影响的需求文档：
  - `core-00-scenario-overview.md`：统一 S01～S05 状态、页面流、主原型与文档映射
  - `core-01-requirements.md`：补齐统一工作空间、鉴权/房间/协作、角色只读、响应式与可访问性需求
  - `core-04-scenario-detail.md`：将 S01/S02 与 S03～S05 连续体验和验收边界对齐主原型
- 影响的功能规格：
  - 信息架构：`core-00-information-architecture.md`
  - S01～S05 场景设计：`core-S01-edit-and-save-design.md`、`core-S02-load-shared-diagram-design.md`、`core-S03-user-auth-design.md`、`core-S04-room-lifecycle-design.md`、`core-S05-ot-collab-design.md`
  - 编辑器与横切规格：`core-01-editor-canvas.md`、`core-01a-table-and-field.md`、`core-01b-relationship.md`、`core-01d-import-export.md`、`core-04-side-panel-tabs.md`、`core-05-top-menu-modals.md`
  - 设计系统与辅助视图：`core-07-design-tokens.md`、`core-08-icon-library.md`、`core-09-core-components.md`、`core-0a-code-editor.md`、`core-0b-dark-mode.md`、`core-0c-motion.md`
- 影响的技术文档：
  - `core-01-architecture-overview.md`：页面状态、前端模块职责、REST/WS 状态来源与原型/生产边界
  - `core-00-scenario-overview.md`（技术场景）：统一 S01～S05 技术状态和依赖
  - `core-S01-edit-and-save-diagram.md`～`core-S05-ot-collab.md`：按主原型交互补齐前端参与者、异常/降级和测试映射
- 影响的业务场景：S01、S02、S03、S04、S05；S06 不受主原型 UI 对齐影响，仅作为回归边界
- 影响的 API：
  - `diagrams.yaml`、`bridge.yaml`、`auth.yaml`、`rooms.yaml`、`collab.yaml` 全量审计
  - 预期不新增端点；如主原型行为无法由现有契约表达，必须先在场景时序图中明确，再通过本提案 delta 更新对应 OpenAPI/WS 契约
- 影响的 DB 表：
  - V1/V2 DDL 全量审计；预期无新增表或迁移
  - 原型本地模拟状态不得反推为持久化字段
- 影响的测试文档：
  - `core-PU-unified-prototype-test-cases.md`
  - `core-V2-production-frontend-test-cases.md`
  - S01～S05 场景测试用例
  - Canvas、Relationship、Import/Export、Modal、SidePanel、Shortcut、Design System 等受影响专项测试矩阵
  - 新增统一原型 → 生产实现逐项对齐矩阵，作为第二阶段代码变更的 UT/ST 合同
- 影响的编排测试：
  - `core-S01-diagram-save.json`、`core-S02-shared-link-load.json`、`core-S03-user-auth.json`、`core-S04-room-lifecycle.json`、`core-S05-ot-collab.json` 全量审计
  - 仅在 API/WS 场景语义变化时产出 delta；纯视觉行为不写入 API 编排
- 影响的实现与验收文档：
  - `core-frontend-alignment-acceptance.md`：重建主原型逐项验收标准
  - `core-implementation-checklist.md`：区分“已有能力”“规格待实现”“第二阶段待验证”，禁止提前标记完成
- 影响的 smoke 测试：无。第一阶段不改代码或部署拓扑。

## 部署影响

- 是否需要部署：否
- 部署原因：本提案仅生成并合并规格 delta，不修改前端、后端、数据库或部署资产
- 影响环境：无
- 是否涉及数据迁移：否
- 是否需要回滚预案：否；文档可通过规格提交回退
- 是否需要 smoke：否

## UI/UX 变更声明

```yaml
ui_impact: true
design_system_mode: generated
design_system_fallback_reason: ""
pages:
  - id: auth
    prototype: core-01-editor-prototype.html
    description: 登录、注册、会话状态与进入房间列表的完整规格基线
  - id: rooms
    prototype: core-01-editor-prototype.html
    description: 房间列表、创建房间、用户菜单与进入协作编辑器的规格基线
  - id: invite
    prototype: core-01-editor-prototype.html
    description: 邀请预览、失效、登录衔接与接受邀请的规格基线
  - id: room-editor
    prototype: core-01-editor-prototype.html
    description: AppBar、ToolRail、Canvas、Inspector、StatusBar、IO、命令/代码视图和协作状态的完整规格基线
```

## 变更概述

本提案不把“看起来相似”视为已对齐。每个主原型可见区域、用户操作、状态转换、权限分支、错误/降级、响应式与无障碍约束，都必须在需求 → 产品设计 → 技术场景 → 测试/验收之间形成可追溯链；文档中的 `data-testid` 只作为测试锚点，生产语义仍以真实 REST/WS 状态为准。

主原型中的演示器、模拟错误、模拟远端操作和示例数据只用于表达体验，不自动成为生产需求。API/DB 是否变更必须由 S01～S05 场景时序推导，不能直接从 HTML 实现细节反推。第一阶段完成、合并、验收并归档后，才创建第二个代码变更 `implement-unified-prototype-spec-parity`，按更新后的规格分批实现前端与必要后端差距；前端以主原型逐项对齐，后端以 API/DB/场景规格为唯一契约。


## 需要合并的 Delta 文件

### 1. deltas/prd/1-product-requirements/core-00-scenario-overview.md

- Delta 文件：`logos/changes/align-all-docs-to-unified-prototype/deltas/prd/1-product-requirements/core-00-scenario-overview.md`
- 目标目录：`logos/resources/prd/1-product-requirements/`
- 操作：读取 delta 中的 ADDED / MODIFIED / REMOVED 标记，合并到目标目录中对应的主文档

### 2. deltas/prd/1-product-requirements/core-01-requirements.md

- Delta 文件：`logos/changes/align-all-docs-to-unified-prototype/deltas/prd/1-product-requirements/core-01-requirements.md`
- 目标目录：`logos/resources/prd/1-product-requirements/`
- 操作：读取 delta 中的 ADDED / MODIFIED / REMOVED 标记，合并到目标目录中对应的主文档

### 3. deltas/prd/1-product-requirements/core-04-scenario-detail.md

- Delta 文件：`logos/changes/align-all-docs-to-unified-prototype/deltas/prd/1-product-requirements/core-04-scenario-detail.md`
- 目标目录：`logos/resources/prd/1-product-requirements/`
- 操作：读取 delta 中的 ADDED / MODIFIED / REMOVED 标记，合并到目标目录中对应的主文档

### 4. deltas/prd/2-product-design/1-feature-specs/core-00-information-architecture.md

- Delta 文件：`logos/changes/align-all-docs-to-unified-prototype/deltas/prd/2-product-design/1-feature-specs/core-00-information-architecture.md`
- 目标目录：`logos/resources/prd/2-product-design/1-feature-specs/`
- 操作：读取 delta 中的 ADDED / MODIFIED / REMOVED 标记，合并到目标目录中对应的主文档

### 5. deltas/prd/2-product-design/1-feature-specs/core-01-editor-canvas.md

- Delta 文件：`logos/changes/align-all-docs-to-unified-prototype/deltas/prd/2-product-design/1-feature-specs/core-01-editor-canvas.md`
- 目标目录：`logos/resources/prd/2-product-design/1-feature-specs/`
- 操作：读取 delta 中的 ADDED / MODIFIED / REMOVED 标记，合并到目标目录中对应的主文档

### 6. deltas/prd/2-product-design/1-feature-specs/core-01a-table-and-field.md

- Delta 文件：`logos/changes/align-all-docs-to-unified-prototype/deltas/prd/2-product-design/1-feature-specs/core-01a-table-and-field.md`
- 目标目录：`logos/resources/prd/2-product-design/1-feature-specs/`
- 操作：读取 delta 中的 ADDED / MODIFIED / REMOVED 标记，合并到目标目录中对应的主文档

### 7. deltas/prd/2-product-design/1-feature-specs/core-01b-relationship.md

- Delta 文件：`logos/changes/align-all-docs-to-unified-prototype/deltas/prd/2-product-design/1-feature-specs/core-01b-relationship.md`
- 目标目录：`logos/resources/prd/2-product-design/1-feature-specs/`
- 操作：读取 delta 中的 ADDED / MODIFIED / REMOVED 标记，合并到目标目录中对应的主文档

### 8. deltas/prd/2-product-design/1-feature-specs/core-01d-import-export.md

- Delta 文件：`logos/changes/align-all-docs-to-unified-prototype/deltas/prd/2-product-design/1-feature-specs/core-01d-import-export.md`
- 目标目录：`logos/resources/prd/2-product-design/1-feature-specs/`
- 操作：读取 delta 中的 ADDED / MODIFIED / REMOVED 标记，合并到目标目录中对应的主文档

### 9. deltas/prd/2-product-design/1-feature-specs/core-04-side-panel-tabs.md

- Delta 文件：`logos/changes/align-all-docs-to-unified-prototype/deltas/prd/2-product-design/1-feature-specs/core-04-side-panel-tabs.md`
- 目标目录：`logos/resources/prd/2-product-design/1-feature-specs/`
- 操作：读取 delta 中的 ADDED / MODIFIED / REMOVED 标记，合并到目标目录中对应的主文档

### 10. deltas/prd/2-product-design/1-feature-specs/core-05-top-menu-modals.md

- Delta 文件：`logos/changes/align-all-docs-to-unified-prototype/deltas/prd/2-product-design/1-feature-specs/core-05-top-menu-modals.md`
- 目标目录：`logos/resources/prd/2-product-design/1-feature-specs/`
- 操作：读取 delta 中的 ADDED / MODIFIED / REMOVED 标记，合并到目标目录中对应的主文档

### 11. deltas/prd/2-product-design/1-feature-specs/core-07-design-tokens.md

- Delta 文件：`logos/changes/align-all-docs-to-unified-prototype/deltas/prd/2-product-design/1-feature-specs/core-07-design-tokens.md`
- 目标目录：`logos/resources/prd/2-product-design/1-feature-specs/`
- 操作：读取 delta 中的 ADDED / MODIFIED / REMOVED 标记，合并到目标目录中对应的主文档

### 12. deltas/prd/2-product-design/1-feature-specs/core-08-icon-library.md

- Delta 文件：`logos/changes/align-all-docs-to-unified-prototype/deltas/prd/2-product-design/1-feature-specs/core-08-icon-library.md`
- 目标目录：`logos/resources/prd/2-product-design/1-feature-specs/`
- 操作：读取 delta 中的 ADDED / MODIFIED / REMOVED 标记，合并到目标目录中对应的主文档

### 13. deltas/prd/2-product-design/1-feature-specs/core-09-core-components.md

- Delta 文件：`logos/changes/align-all-docs-to-unified-prototype/deltas/prd/2-product-design/1-feature-specs/core-09-core-components.md`
- 目标目录：`logos/resources/prd/2-product-design/1-feature-specs/`
- 操作：读取 delta 中的 ADDED / MODIFIED / REMOVED 标记，合并到目标目录中对应的主文档

### 14. deltas/prd/2-product-design/1-feature-specs/core-0a-code-editor.md

- Delta 文件：`logos/changes/align-all-docs-to-unified-prototype/deltas/prd/2-product-design/1-feature-specs/core-0a-code-editor.md`
- 目标目录：`logos/resources/prd/2-product-design/1-feature-specs/`
- 操作：读取 delta 中的 ADDED / MODIFIED / REMOVED 标记，合并到目标目录中对应的主文档

### 15. deltas/prd/2-product-design/1-feature-specs/core-0b-dark-mode.md

- Delta 文件：`logos/changes/align-all-docs-to-unified-prototype/deltas/prd/2-product-design/1-feature-specs/core-0b-dark-mode.md`
- 目标目录：`logos/resources/prd/2-product-design/1-feature-specs/`
- 操作：读取 delta 中的 ADDED / MODIFIED / REMOVED 标记，合并到目标目录中对应的主文档

### 16. deltas/prd/2-product-design/1-feature-specs/core-0c-motion.md

- Delta 文件：`logos/changes/align-all-docs-to-unified-prototype/deltas/prd/2-product-design/1-feature-specs/core-0c-motion.md`
- 目标目录：`logos/resources/prd/2-product-design/1-feature-specs/`
- 操作：读取 delta 中的 ADDED / MODIFIED / REMOVED 标记，合并到目标目录中对应的主文档

### 17. deltas/prd/2-product-design/1-feature-specs/core-S01-edit-and-save-design.md

- Delta 文件：`logos/changes/align-all-docs-to-unified-prototype/deltas/prd/2-product-design/1-feature-specs/core-S01-edit-and-save-design.md`
- 目标目录：`logos/resources/prd/2-product-design/1-feature-specs/`
- 操作：读取 delta 中的 ADDED / MODIFIED / REMOVED 标记，合并到目标目录中对应的主文档

### 18. deltas/prd/2-product-design/1-feature-specs/core-S02-load-shared-diagram-design.md

- Delta 文件：`logos/changes/align-all-docs-to-unified-prototype/deltas/prd/2-product-design/1-feature-specs/core-S02-load-shared-diagram-design.md`
- 目标目录：`logos/resources/prd/2-product-design/1-feature-specs/`
- 操作：读取 delta 中的 ADDED / MODIFIED / REMOVED 标记，合并到目标目录中对应的主文档

### 19. deltas/prd/2-product-design/1-feature-specs/core-S03-user-auth-design.md

- Delta 文件：`logos/changes/align-all-docs-to-unified-prototype/deltas/prd/2-product-design/1-feature-specs/core-S03-user-auth-design.md`
- 目标目录：`logos/resources/prd/2-product-design/1-feature-specs/`
- 操作：读取 delta 中的 ADDED / MODIFIED / REMOVED 标记，合并到目标目录中对应的主文档

### 20. deltas/prd/2-product-design/1-feature-specs/core-S04-room-lifecycle-design.md

- Delta 文件：`logos/changes/align-all-docs-to-unified-prototype/deltas/prd/2-product-design/1-feature-specs/core-S04-room-lifecycle-design.md`
- 目标目录：`logos/resources/prd/2-product-design/1-feature-specs/`
- 操作：读取 delta 中的 ADDED / MODIFIED / REMOVED 标记，合并到目标目录中对应的主文档

### 21. deltas/prd/2-product-design/1-feature-specs/core-S05-ot-collab-design.md

- Delta 文件：`logos/changes/align-all-docs-to-unified-prototype/deltas/prd/2-product-design/1-feature-specs/core-S05-ot-collab-design.md`
- 目标目录：`logos/resources/prd/2-product-design/1-feature-specs/`
- 操作：读取 delta 中的 ADDED / MODIFIED / REMOVED 标记，合并到目标目录中对应的主文档

### 22. deltas/prd/3-technical-plan/1-architecture/core-01-architecture-overview.md

- Delta 文件：`logos/changes/align-all-docs-to-unified-prototype/deltas/prd/3-technical-plan/1-architecture/core-01-architecture-overview.md`
- 目标目录：`logos/resources/prd/3-technical-plan/1-architecture/`
- 操作：读取 delta 中的 ADDED / MODIFIED / REMOVED 标记，合并到目标目录中对应的主文档

### 23. deltas/prd/3-technical-plan/2-scenario-implementation/core-00-scenario-overview.md

- Delta 文件：`logos/changes/align-all-docs-to-unified-prototype/deltas/prd/3-technical-plan/2-scenario-implementation/core-00-scenario-overview.md`
- 目标目录：`logos/resources/prd/3-technical-plan/2-scenario-implementation/`
- 操作：读取 delta 中的 ADDED / MODIFIED / REMOVED 标记，合并到目标目录中对应的主文档

### 24. deltas/prd/3-technical-plan/2-scenario-implementation/core-S01-edit-and-save-diagram.md

- Delta 文件：`logos/changes/align-all-docs-to-unified-prototype/deltas/prd/3-technical-plan/2-scenario-implementation/core-S01-edit-and-save-diagram.md`
- 目标目录：`logos/resources/prd/3-technical-plan/2-scenario-implementation/`
- 操作：读取 delta 中的 ADDED / MODIFIED / REMOVED 标记，合并到目标目录中对应的主文档

### 25. deltas/prd/3-technical-plan/2-scenario-implementation/core-S02-load-shared-diagram.md

- Delta 文件：`logos/changes/align-all-docs-to-unified-prototype/deltas/prd/3-technical-plan/2-scenario-implementation/core-S02-load-shared-diagram.md`
- 目标目录：`logos/resources/prd/3-technical-plan/2-scenario-implementation/`
- 操作：读取 delta 中的 ADDED / MODIFIED / REMOVED 标记，合并到目标目录中对应的主文档

### 26. deltas/prd/3-technical-plan/2-scenario-implementation/core-S03-user-auth.md

- Delta 文件：`logos/changes/align-all-docs-to-unified-prototype/deltas/prd/3-technical-plan/2-scenario-implementation/core-S03-user-auth.md`
- 目标目录：`logos/resources/prd/3-technical-plan/2-scenario-implementation/`
- 操作：读取 delta 中的 ADDED / MODIFIED / REMOVED 标记，合并到目标目录中对应的主文档

### 27. deltas/prd/3-technical-plan/2-scenario-implementation/core-S04-room-lifecycle.md

- Delta 文件：`logos/changes/align-all-docs-to-unified-prototype/deltas/prd/3-technical-plan/2-scenario-implementation/core-S04-room-lifecycle.md`
- 目标目录：`logos/resources/prd/3-technical-plan/2-scenario-implementation/`
- 操作：读取 delta 中的 ADDED / MODIFIED / REMOVED 标记，合并到目标目录中对应的主文档

### 28. deltas/prd/3-technical-plan/2-scenario-implementation/core-S05-ot-collab.md

- Delta 文件：`logos/changes/align-all-docs-to-unified-prototype/deltas/prd/3-technical-plan/2-scenario-implementation/core-S05-ot-collab.md`
- 目标目录：`logos/resources/prd/3-technical-plan/2-scenario-implementation/`
- 操作：读取 delta 中的 ADDED / MODIFIED / REMOVED 标记，合并到目标目录中对应的主文档

### 29. deltas/test/core-CR-canvas-test-cases.md

- Delta 文件：`logos/changes/align-all-docs-to-unified-prototype/deltas/test/core-CR-canvas-test-cases.md`
- 目标目录：`logos/resources/test/`
- 操作：读取 delta 中的 ADDED / MODIFIED / REMOVED 标记，合并到目标目录中对应的主文档

### 30. deltas/test/core-KB-shortcut-test-cases.md

- Delta 文件：`logos/changes/align-all-docs-to-unified-prototype/deltas/test/core-KB-shortcut-test-cases.md`
- 目标目录：`logos/resources/test/`
- 操作：读取 delta 中的 ADDED / MODIFIED / REMOVED 标记，合并到目标目录中对应的主文档

### 31. deltas/test/core-PB-relationship-test-cases.md

- Delta 文件：`logos/changes/align-all-docs-to-unified-prototype/deltas/test/core-PB-relationship-test-cases.md`
- 目标目录：`logos/resources/test/`
- 操作：读取 delta 中的 ADDED / MODIFIED / REMOVED 标记，合并到目标目录中对应的主文档

### 32. deltas/test/core-PC-import-export-test-cases.md

- Delta 文件：`logos/changes/align-all-docs-to-unified-prototype/deltas/test/core-PC-import-export-test-cases.md`
- 目标目录：`logos/resources/test/`
- 操作：读取 delta 中的 ADDED / MODIFIED / REMOVED 标记，合并到目标目录中对应的主文档

### 33. deltas/test/core-PE-design-system-test-cases.md

- Delta 文件：`logos/changes/align-all-docs-to-unified-prototype/deltas/test/core-PE-design-system-test-cases.md`
- 目标目录：`logos/resources/test/`
- 操作：读取 delta 中的 ADDED / MODIFIED / REMOVED 标记，合并到目标目录中对应的主文档

### 34. deltas/test/core-PU-unified-prototype-test-cases.md

- Delta 文件：`logos/changes/align-all-docs-to-unified-prototype/deltas/test/core-PU-unified-prototype-test-cases.md`
- 目标目录：`logos/resources/test/`
- 操作：读取 delta 中的 ADDED / MODIFIED / REMOVED 标记，合并到目标目录中对应的主文档

### 35. deltas/test/core-S01-test-cases.md

- Delta 文件：`logos/changes/align-all-docs-to-unified-prototype/deltas/test/core-S01-test-cases.md`
- 目标目录：`logos/resources/test/`
- 操作：读取 delta 中的 ADDED / MODIFIED / REMOVED 标记，合并到目标目录中对应的主文档

### 36. deltas/test/core-S02-test-cases.md

- Delta 文件：`logos/changes/align-all-docs-to-unified-prototype/deltas/test/core-S02-test-cases.md`
- 目标目录：`logos/resources/test/`
- 操作：读取 delta 中的 ADDED / MODIFIED / REMOVED 标记，合并到目标目录中对应的主文档

### 37. deltas/test/core-S03-test-cases.md

- Delta 文件：`logos/changes/align-all-docs-to-unified-prototype/deltas/test/core-S03-test-cases.md`
- 目标目录：`logos/resources/test/`
- 操作：读取 delta 中的 ADDED / MODIFIED / REMOVED 标记，合并到目标目录中对应的主文档

### 38. deltas/test/core-S04-test-cases.md

- Delta 文件：`logos/changes/align-all-docs-to-unified-prototype/deltas/test/core-S04-test-cases.md`
- 目标目录：`logos/resources/test/`
- 操作：读取 delta 中的 ADDED / MODIFIED / REMOVED 标记，合并到目标目录中对应的主文档

### 39. deltas/test/core-S05-test-cases.md

- Delta 文件：`logos/changes/align-all-docs-to-unified-prototype/deltas/test/core-S05-test-cases.md`
- 目标目录：`logos/resources/test/`
- 操作：读取 delta 中的 ADDED / MODIFIED / REMOVED 标记，合并到目标目录中对应的主文档

### 40. deltas/test/core-SP-side-panel-test-cases.md

- Delta 文件：`logos/changes/align-all-docs-to-unified-prototype/deltas/test/core-SP-side-panel-test-cases.md`
- 目标目录：`logos/resources/test/`
- 操作：读取 delta 中的 ADDED / MODIFIED / REMOVED 标记，合并到目标目录中对应的主文档

### 41. deltas/test/core-UI-modals-2-test-cases.md

- Delta 文件：`logos/changes/align-all-docs-to-unified-prototype/deltas/test/core-UI-modals-2-test-cases.md`
- 目标目录：`logos/resources/test/`
- 操作：读取 delta 中的 ADDED / MODIFIED / REMOVED 标记，合并到目标目录中对应的主文档

### 42. deltas/test/core-UI-modals-test-cases.md

- Delta 文件：`logos/changes/align-all-docs-to-unified-prototype/deltas/test/core-UI-modals-test-cases.md`
- 目标目录：`logos/resources/test/`
- 操作：读取 delta 中的 ADDED / MODIFIED / REMOVED 标记，合并到目标目录中对应的主文档

### 43. deltas/test/core-V2-production-frontend-test-cases.md

- Delta 文件：`logos/changes/align-all-docs-to-unified-prototype/deltas/test/core-V2-production-frontend-test-cases.md`
- 目标目录：`logos/resources/test/`
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
   git add -A && git commit -m "docs(align-all-docs-to-unified-prototype): merge spec deltas"
   然后提示用户：按更新后的规格实现代码，代码完成后运行 `openlogos verify` 验收，验收通过后明确授权执行 `openlogos archive align-all-docs-to-unified-prototype`。
