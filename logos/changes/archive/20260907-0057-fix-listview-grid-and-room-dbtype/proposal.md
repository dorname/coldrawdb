# 变更提案：fix-listview-grid-and-room-dbtype

> module: core | created: 2026-09-06

## 变更原因

用户反馈三个问题（2026-09-06）：

1. **列表视图（ViewMode::List）空白**：`.cdb-app` 是固定 3 行网格（`appbar / minmax(0,1fr) / statusbar`），List 模式下 `.cdb-main` 被 `display:none` 后，StatusBar 被自动放置顶到 1fr 大行，`.cdb-list-view-panel` 被挤到状态栏高度的行/隐式行并被 `overflow:hidden` 裁掉；且 `.cdb-list-view-panel` 在 styles.css 中**完全没有样式定义**。数据本身无问题（状态栏显示 3 张表，ListView 读同一 `store.tables`）。
2. **原型未同步**：唯一主原型 `core-01-editor-prototype.html` 最后提交 2026-08-22，p0-fix（区域/便签）、list-view-table-structure、diagram-database-dialect 三批已合入功能均未回写（list-view 0 处、引擎切换 0 处、数据库类型 0 处）。
3. **创建协作空间/新图未先选数据库类型（逻辑漏洞）**：房间创建模态（选「新建空白模型」）与新建图模态均无引擎选择，新图 `database` 永远默认 Generic，而字段类型清单依赖引擎（diagram-database-dialect 已建依赖关系）。后端 `POST /api/v1/diagrams` **已支持** `database` 可选入参（diagrams_v1.rs:26），前端 `DiagramClient::create()` 硬编码 `database: None`（editor_data_access.rs:1052）——本变更为纯前端接通，无 API/DB 变更。

## 变更类型

**设计级**（交互流程补全：创建流程增加引擎前置选择；原型回写已合入功能；代码 + 测试跟随。无 API/DB/部署变更——后端已支持 database 入参）

## 变更范围

- 影响的需求文档：无（用户反馈驱动的交互补全，不改需求条目）
- 影响的功能规格：
  - `prd/2-product-design/1-feature-specs/core-S04-room-lifecycle-design.md`（创建房间流程加引擎前置选择）
  - `prd/2-product-design/2-page-design/core-01-editor-prototype.html`（回写：列表视图、AppBar 引擎下拉、创建房间/新图引擎选择）
- 影响的业务场景：S01（画布编辑——列表视图修复）、S04（房间生命周期——创建流程）
- 影响的部署方案：无
- 影响的 API：无（database 入参已存在于 `api/diagrams.yaml`）
- 影响的 DB 表：无
- 影响的编排测试：`test/core-S04-test-cases.md`（新增创建房间选引擎 ST）；`test/core-SP-side-panel-test-cases.md`（新增列表视图全屏渲染 ST）
- 影响的 smoke 测试：无
- 影响的代码：
  - `frontend-rs/src/styles.css`（`.cdb-list-view-panel` 网格定位与布局样式）
  - `frontend-rs/src/editor_panels.rs`（房间创建模态引擎下拉）
  - `frontend-rs/src/editor_data_access.rs`（`DiagramClient::create` 透传 database）

**实现期范围修正（2026-09-06，用户决策）**：原计划含新建图模态（`ModalKind::New`）引擎下拉 + ST-S04-UI-10。勘察发现其唯一触发点 TopBar/TopMenuBar 为死代码（生产端无独立编辑器页，图只在房间内编辑），且 `on_new_diagram` 的 set_href 导航在无 room 参数时落回房间列表页。决策：撤销独立新建图入口，引擎前置选择由「创建房间 → 新建空白模型」唯一活路径承载；ST-S04-UI-10 删除。

## 部署影响

- 是否需要部署：**否**（纯前端交互补全 + CSS 修复，无后端变更）
- 部署原因：仅前端 Rust/WASM 与样式变更
- 影响环境：无
- 是否涉及数据迁移：否（database 字段已在序列化 schema 内，默认 Generic 向后兼容）
- 是否需要回滚预案：否
- 是否需要 smoke：否

## 变更概述

1. **列表视图网格修复**：为 `.cdb-list-view-panel` 补样式（钉入 `grid-row: 2`、`min-height:0`、`overflow:auto`、纵向 flex 布局），消除对网格自动放置顺位的依赖；List 模式下表格与过滤工具条正常可见。
2. **创建流程引擎前置**：房间创建模态在选「新建空白模型」时露出数据库引擎下拉（Generic/MySQL/PostgreSQL，与 AppBar 三项对齐）；新建图模态同样加引擎下拉；`DiagramClient::create(name, database)` 透传引擎，建图即落账正确引擎，Inspector 字段类型清单从建图起即按引擎过滤。
3. **原型回写**：`core-01-editor-prototype.html` 补齐列表视图、AppBar 引擎下拉、创建房间/新图引擎选择三处已合入能力的交互展示。

## 不在范围

- 房间创建时选择既有模型的引擎变更（引擎随既有模型，不在创建流程修改）
- 切换引擎后的类型迁移映射（沿用 diagram-database-dialect 既定「原样保留」决策）
- Sqlite/Mssql/Oracle 的 UI 暴露（UI 仍只出 Generic/MySQL/PostgreSQL 三项）
- S03/S04/S05 历史独立原型的回写（仅维护唯一主原型）

## 验收标准

- 画布有表时切列表视图 → 表结构清单（表名/字段数/类型/索引）正常展示，过滤/排序/分组可用；空表时展示为空而非布局塌陷
- 创建房间选「新建空白模型」+ 选 MySQL → 进入房间后 AppBar 引擎 = MySQL，Inspector 字段类型含 `DATETIME` 无 `SERIAL`；后端落账 `database='mysql'`
- 新建图选 PostgreSQL → 新图引擎 = PostgreSQL，字段类型含 `SERIAL`
- 不选引擎（默认 Generic）时行为与现状一致（回归不破）
- 原型中可看到列表视图与引擎选择交互
