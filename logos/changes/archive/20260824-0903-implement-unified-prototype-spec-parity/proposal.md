# 变更提案：implement-unified-prototype-spec-parity

> module: core | created: 2026-08-23

## 变更原因

上一变更 `align-all-docs-to-unified-prototype` 已把 S01～S05 的需求、产品设计、技术场景、测试合同与验收清单对齐到唯一现行主原型 `core-01-editor-prototype.html`，并留下第二阶段代码合同：生产前端须按页面区域 checklist 与 `SPEC_PARITY` 用例逐项贴合主原型。

当前仓库状态是：后端 auth/rooms/collab 已实现；生产前端已有 API 接入和部分页面流（auth → rooms → room-editor、`?share=` 旁路、invite 页）；但相对主原型的结构、视觉、交互、权限与降级仍未逐项验收。为让 `openlogos verify` 通过账本覆盖，上一变更把约 45 个新增 UT/ST 写成 `SPEC_PARITY_SKIP_IDS`。这些 skip 不是完成标记，必须在本提案中变成真实测试或带缺口说明的显式 skip。

本提案执行该第二阶段：按已合并规格补齐生产前端差距，把 skip 用例落地，并回写实现/验收状态。不重新设计需求，不把主原型演示器当成生产能力。

## 变更类型

代码级为主，外加实现状态与测试合同状态的规格回写。不改变已合并的需求语义、API、DB 或主原型 HTML。

## 变更范围

- 基准原型（只读，不修改）：
  - `logos/resources/prd/2-product-design/2-page-design/core-01-editor-prototype.html`
  - `core-03/04/05-*-prototype.html` 仅历史参考，不作为验收入口
- 影响的需求文档：无语义变更；追溯 `core-00-scenario-overview.md`、`core-01-requirements.md`、`core-04-scenario-detail.md`
- 影响的功能规格：无交互重设计；实现对照
  - `core-00-information-architecture.md`
  - `core-S01-edit-and-save-design.md`～`core-S05-ot-collab-design.md`
  - `core-01-editor-canvas.md`、`core-01a-table-and-field.md`、`core-01b-relationship.md`、`core-01d-import-export.md`
  - `core-04-side-panel-tabs.md`、`core-05-top-menu-modals.md`
  - `core-07`～`core-0c` 设计系统规格
- 影响的业务场景：S01、S02、S03、S04、S05；S06 仅回归边界，本提案不改 MCP
- 影响的部署方案：无
- 影响的 API：`auth.yaml`、`rooms.yaml`、`collab.yaml`、`diagrams.yaml`、`bridge.yaml` 仅作调用契约；不新增端点。若实现中发现契约无法表达主原型行为，先停写并给出差异，不在本提案内发明字段/端点
- 影响的 DB 表：无新增或迁移
- 影响的编排测试：不新增后端 JSON；回归既有 S01～S05 编排。新增/改写的是前端 UT/ST/e2e 与 OpenLogos reporter
- 影响的测试文档（状态回写，不改用例语义）：
  - `core-S01-test-cases.md`～`core-S05-test-cases.md`
  - `core-PU-unified-prototype-test-cases.md`
  - `core-V2-production-frontend-test-cases.md`
  - `core-KB-shortcut-test-cases.md`、`core-PC-import-export-test-cases.md`
- 影响的实现/验收文档：
  - `core-frontend-alignment-acceptance.md`（§7 区域 checklist 随批次勾选；§4 verify slug 改为本提案）
  - `core-implementation-checklist.md`（§13 第二阶段由本提案执行并在收口时改「待验证」）
- 影响的 smoke 测试：无
- 影响的代码（预期）：
  - `frontend-rs/src/lib.rs`、`editor_panels.rs`、`editor_render.rs`、`editor_data_access.rs`、`editor_core.rs` 及样式
  - `frontend-rs/tests/`、`frontend-rs/scripts/test-unified-prototype*.mjs`、e2e
  - `frontend-rs/tests/openlogos_reporter.rs`：随用例落地删除对应 `SPEC_PARITY_SKIP_IDS`
  - 后端仅在现有契约下修 bug；不扩表、不加路由

## 部署影响

- 是否需要部署：否
- 部署原因：不改 API、DB、发布拓扑或运行中服务配置；对齐工作在本地 `trunk` / 既有前后端进程上验收。模块级 `deployment_required` 不覆盖本提案明确决策
- 影响环境：无
- 是否涉及数据迁移：否
- 是否需要回滚预案：否；可用 git 回退本提案提交
- 是否需要 smoke：否

## UI/UX 变更声明

```yaml
ui_impact: true
design_system_mode: generated
design_system_fallback_reason: ""
pages:
  - id: auth
    prototype: core-01-editor-prototype.html
    description: 未登录默认入口、双 tab、字段错误、loading、成功后进入 rooms
  - id: rooms
    prototype: core-01-editor-prototype.html
    description: 房间列表/空状态、创建入口、用户菜单、进入协作编辑器
  - id: invite
    prototype: core-01-editor-prototype.html
    description: 邀请预览、过期、未登录衔接、接受后进入同一房间
  - id: room-editor
    prototype: core-01-editor-prototype.html
    description: 壳层、保存态、协作可见状态、Viewer 只读、IO/命令/代码、主题与 720px
```

## 变更概述

以 `core-frontend-alignment-acceptance.md` §7 页面区域 checklist 为验收入口，以 FEALIGN/FEUX 为能力维度。生产状态必须来自真实 REST/WS 或明确降级文案；禁止把主原型「模拟远端 / 断线 / 诊断」控件标成生产完成。

实现按页面流分四批，每批同时交付：业务代码、对应 UT/ST（或 e2e）、OpenLogos reporter。输出代码前先列出本批用例 ID。`SPEC_PARITY_SKIP_IDS` 随批次从 reporter 中移除；不得静默缺失。

后端以已合并 API/DB/场景为唯一契约。S01 保存/409、S02 分享只读、IO、命令面板、设计系统与既有 ST-PU 回归不可回退。
