# 变更提案：fix-pg-types-listview-zindex-lock-engine

> module: core | created: 2026-09-07

## 变更原因

`fix-listview-grid-and-room-dbtype` 验收通过后真机回归发现 3 个缺陷（用户截图反馈）：

1. **PostgreSQL 字段类型清单错误**：选 PG 后 Inspector 类型下拉为 `INT/BIGINT/VARCHAR(255)/TEXT/SERIAL/TIMESTAMP/NUMERIC(10,2)`，缺 UUID/BOOLEAN；且字段当前类型（UUID）不在清单时被兜底逻辑追加为占位项，下拉出现重复 UUID。生产常量（`editor_core.rs:140`）与既有规格 `core-01a-table-and-field.md:82`（PG 含 UUID/SERIAL 等）不一致。
2. **列表视图真机仍空白**：`.cdb-aurora` 背景层是 `.cdb-app` 首子元素且 `position:fixed; inset:0`（不透明渐变），`.cdb-list-view-panel` 是非定位元素，被 aurora 整个盖在下方。ST-SP-LIST-01 只断言几何尺寸（boundingBox），Playwright `:visible` 不检测绘制遮挡，存在测试盲区。
3. **引擎创建后应锁定**：房间编辑器 AppBar 引擎下拉（`app-db-select`）仅 `disabled=read_only`，创建空间后仍可随意切换引擎，导致既有字段类型与新引擎清单脱节；引擎应在「创建房间」时确定、之后只读。

## 变更类型

设计级变更（功能规格 + 原型 + 测试用例 + 代码）

## 变更范围

- 影响的需求文档：无（验收条件不变）
- 影响的功能规格：
  - `core-01a-table-and-field.md`（每引擎 MVP 类型下拉短清单权威表）
  - `core-S04-room-lifecycle-design.md`（S04.1 增补「引擎创建后锁定」约束）
- 影响的原型：`core-01-editor-prototype.html`（typesForDatabase 清单对齐 + AppBar 引擎下拉禁用）
- 影响的业务场景：S04（房间生命周期）、SP（侧栏/列表视图）
- 影响的部署方案：无
- 影响的 API：无（后端 `database` 字段已就绪）
- 影响的 DB 表：无
- 影响的编排测试：无
- 影响的 ST 用例：
  - `core-UI-modals-2-test-cases.md` ST-DB-01（引擎切换语义作废 → REMOVED，由 ST-DB-02 锁定语义替代）
  - `core-SP-side-panel-test-cases.md` ST-SP-LIST-01（MODIFIED：补绘制遮挡断言）
- 影响的 smoke 测试：无

## 部署影响

- 是否需要部署：否
- 部署原因：纯前端修复（常量、CSS 层叠、下拉禁用），无后端/部署方案变更
- 影响环境：无
- 是否涉及数据迁移：否
- 是否需要回滚预案：否
- 是否需要 smoke：否

## 变更概述

1. **类型清单对齐**：`types_for_database` 按 `core-01a` 权威表补齐——PostgreSQL 增 UUID/BOOLEAN（`UUID/INT/BIGINT/SERIAL/VARCHAR(255)/TEXT/BOOLEAN/TIMESTAMP/NUMERIC(10,2)`），Generic 增 UUID（避免 Generic 房间 UUID 字段同样触发占位重复）；MySQL 不变。原型 `typesForDatabase` 同步对齐。UUID 入列后 Inspector 占位追加逻辑不再触发，重复项消失。
2. **列表视图层叠修复**：`.cdb-list-view-panel` 增加 `position: relative; z-index: 1`，抬升到 aurora 背景层之上；ST-SP-LIST-01 补 `document.elementFromPoint` 断言，堵住绘制遮挡盲区。
3. **引擎锁定**：房间编辑器 AppBar 引擎下拉禁用（仅作创建时所选引擎的只读标识），引擎只在「创建房间 → 新建空白模型」时可选；ST-DB-01 的切换语义作废，替换为 ST-DB-02 锁定断言。
