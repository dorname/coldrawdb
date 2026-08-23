# 变更提案：align-all-docs-to-unified-prototype

> module: core | created: 2026-08-22

## 变更原因

现行唯一视觉与交互基线 `core-01-editor-prototype.html` 已串联 S01～S05 的 Auth、Rooms、Invite、协作编辑器、画布建模、导入导出、命令/代码视图、权限与断线恢复体验；但现有需求、功能规格、技术场景、测试矩阵和实现清单是在多轮独立变更中增量形成，仍混有以下风险：

1. 部分文档继续使用旧 Landing / 编辑器直达 / 独立 S03～S05 原型等历史叙述，与现行 `auth → rooms → room-editor` 页面流不一致。
2. 页面锚点、状态文案、权限、响应式、浮层、保存/协作状态与主原型之间缺少一份完整的端到端追溯基线。
3. 既有实现清单曾标记“已对齐”，但不足以证明生产前端逐项达到主原型的结构、视觉和交互要求，也不能直接作为下一轮前后端实现的任务合同。

本变更先完成 Why → What → How 的文档收口：以主原型为事实输入，审计并对齐 S01～S05 所有受影响文档，为后续独立代码变更 `implement-unified-prototype-spec-parity` 建立唯一、可测试的实现基线。

## 变更类型

需求级纯规格变更，外加验收预跑代码级修复。覆盖需求、产品设计、技术场景、测试与验收追溯；不改变主原型与生产前后端业务逻辑，不执行部署。

因本提案尚未归档，无法另建 `fix-verify-playwright-sandbox`；`openlogos verify` 在沙箱内因 Playwright 浏览器缓存不完整失败，故将浏览器解析修复收口到本提案 `[code]`。

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
- 影响的 smoke 测试：无。
- 影响的验收预跑：
  - `scripts/run-verify-tests.sh`：解析可用 Playwright 浏览器后再跑单文件原型回归
  - `frontend-rs/scripts/resolve-playwright-browsers.mjs`：忽略不完整的 `PLAYWRIGHT_BROWSERS_PATH`，回退到 `~/.cache/ms-playwright` 或可写临时目录；仅有完整 Chromium 时关闭 headless shell
  - `frontend-rs/scripts/test-unified-prototype.mjs` / `test-unified-prototype-render.mjs`：启动前应用同一解析策略；工作区只读时截图落到临时目录
  - `frontend-rs/tests/openlogos_reporter.rs`：本提案新增、待 `implement-unified-prototype-spec-parity` 实现的用例写入 skip

## 部署影响

- 是否需要部署：否
- 部署原因：规格合并与验收预跑脚本修复均不改变运行中的前后端、数据库或部署拓扑
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
