# 变更提案：add-frontend-completeness

> module: core | created: 2026-06-08
> guard: logos/.openlogos-guard

## 变更原因

用户反馈（2026-06-08）：前端界面仍是一个简单页面，spec 中规划的 90% UI 功能（顶部菜单、6 Tab、9 模态、CSS 样式、撤销/重做 UI、画布增强交互、搜索筛选）尚未实现。

**事实盘点**（`frontend-rs/src/`，5 个文件共 1620 行）：

| 模块 | 已实现 | 距 spec 差距 |
|---|---|---|
| `editor_core` | types/Store/Command 栈/Debounce/409 弹窗基础 | 撤销/重做 UI 未暴露 |
| `editor_render` | draw_table/draw_bezier/draw_area/draw_note 函数都在 | Canvas 组件传空 areas/notes；references 渲染空 |
| `editor_panels` | TopBar（建表+保存）、LeftPanel（**仅 Tables**）、RightPanel（字段增删改） | 6 Tab 缺 5 个；无顶部菜单；无模态 |
| `editor_data_access` | gloo-net HTTP 客户端 + create/get/save/delete | — |
| 样式 | `dist/index.html` **零自定义 CSS** | 完全裸奔 |

**spec 已规划但未实现**（来源 `logos/resources/prd/2-product-design/`）：
- 顶部 4 下拉菜单（File/Edit/View/Help）+ 撤销/重做/Share/Export 工具栏
- 9 个模态：New / Open / Import / ImportSource / Language / SetTableWidth / Share / Rename / ConfigureCustomTypes
- 6+1 个侧边栏 Tab：Tables ✅ / **Areas / Enums / Notes / Relationships / Types / Issues**
- 画布：拖拽端点 / 框选 / 右键菜单 / 键盘快捷键
- 搜索 + 筛选
- 20 个 skipped 测试用例（`logos/resources/verify/acceptance-report.md`）

**附加问题**：`core-implementation-checklist.md` §2.3 误将 Areas/Enums Tab 标记为 `[x]`，但实际 `editor_panels.rs` 中并无对应组件。清单与代码不一致，需要随此提案修正。

## 变更类型

**设计级 + 代码级**（前端 UI 全面补全）。需求文档已存在（baseline docs 提案 add-baseline-docs），API/DB 不变。

## 变更范围

- **影响的需求文档**：无（需求层已就绪，定义在 `core-00-scenario-overview.md` / `core-01-requirements.md`）
- **影响的功能规格**：
  - `prd/2-product-design/1-feature-specs/core-01-editor-canvas.md` — 章节 5 渲染策略需要补充「样式底座」段落
  - `prd/2-product-design/1-feature-specs/core-04-side-panel-tabs.md` — Areas/Enums/Notes/Relationships/Types/Issues Tab 详细设计（已存在但未实现）
  - `prd/2-product-design/1-feature-specs/core-05-top-menu-modals.md` — 9 模态详细规格（已存在但未实现）
- **影响的业务场景**：S01（编辑+自动保存）、S02（共享链接加载）— 仅前端 UI 补全，不改变后端契约
- **影响的 API**：无（`backend/src/diagrams_v1.rs` 已就绪）
- **影响的 DB 表**：无
- **影响的编排测试**：
  - `logos/resources/test/core-S01-test-cases.md` — 12 个 skipped 用例（UT-S01-02/06-10、ST-S01-03）
  - `logos/resources/test/core-S02-test-cases.md` — 8 个 skipped 用例（UT-S02-03~09、ST-S02-01~05）
- **影响的 smoke 测试**：`logos/resources/test/smoke/` — 前端 e2e smoke 需要补全
- **影响的实现清单**：`logos/resources/implementation/core-implementation-checklist.md` — §2.3 修正 Areas/Enums 误标，新增未勾选项

## 部署影响

- **是否需要部署**：是
- **部署原因**：前端静态资源（`frontend-rs/dist/`）内容大幅扩展；staging 上需要重新 build + 替换
- **影响环境**：staging（V1 不实现 production，见 `core-01-deployment-plan.md`）
- **是否涉及数据迁移**：否（仅前端代码 + dist 产物）
- **是否需要回滚预案**：是（`git revert` 到当前 `0375339` 即可恢复 dist）
- **是否需要 smoke**：是（`openlogos smoke` 跑前端 e2e 覆盖核心 5 happy path）

## 变更概述

本提案将前端 UI 从 MVP（5 大 happy path）补全至 spec 定义的功能全集。鉴于工作量，按**5 个独立闭环批次**分批实施，每批都同时交付业务代码 + UT/ST 测试 + OpenLogos reporter：

### 批次划分（按依赖顺序）

| 批次 | 范围 | 闭环交付物 |
|---|---|---|
| **B1** | CSS 样式底座（设计 token + 主题色 + 布局栅格） + 顶部菜单骨架（4 下拉空壳） + 撤销/重做 UI（底座已就绪，零成本开启） | editor_panels 增量 + 新增 styles.css + UT/ST 用例 |
| **B2** | 5 个剩余侧栏 Tab（Areas / Enums / Notes / Relationships / Types）+ Tab 切换器 + 列表项 | editor_panels 大幅扩展 + UT/ST 用例 |
| **B3** | 画布渲染补全（areas/notes/references 真正接入）+ Issues Tab | editor_render/panels 改造 + UT/ST 用例 |
| **B4** | 4 个核心模态（New / Open / Share / Rename） | editor_panels 新增 Modal 子模块 + UT/ST 用例 |
| **B5** | 剩余 5 模态（Import / ImportSource / Language / SetTableWidth / ConfigureCustomTypes）+ 撤销/重做键盘快捷键 + 搜索筛选 + 修正实现清单 | editor_panels 完结 + 实现清单修正 + UT/ST 用例 |

### 关键约束

1. 每批必须独立可运行（不破坏已通过的核心 5 happy path）
2. 每批必须为当前批覆盖的 skipped 用例补 UT/ST 测试代码
3. 每批提交后 `openlogos verify` 必须仍然 PASS（或 skipped 减少）
4. **B1 优先**：CSS 底座不落地，后续 4 批都将"裸奔"在无样式 HTML 中
5. **B1 → B2 → B3 → B4 → B5 严格串行**：后批依赖前批的样式/组件

### 风险与缓解

- **风险**：Leptos 0.5 + 大量组件可能导致 wasm 体积膨胀 > 5MB
  - **缓解**：B1 完成后实测 `trunk build` 体积；超阈值则按需拆分 lazy component
- **风险**：CSS 手写易与 Leptos 类名冲突
  - **缓解**：B1 中统一前缀 `.cdb-*`（coldrawdb），所有组件 class 强制加前缀
- **风险**：9 模态工作量评估可能偏低
  - **缓解**：B4/B5 各自产出后回看实际进度，可拆 B5 为 B5a/B5b

### 不在本提案范围

- 后端 API 任何变更（已就绪）
- 数据库 schema 任何变更（已就绪）
- 实时协作（V2 范围，由 `add-v2-collab-spec` 提案处理）
- 生产环境部署（V1 仅 staging）
