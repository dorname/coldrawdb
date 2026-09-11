# 变更提案：diagram-database-dialect

> module: core | created: 2026-09-05

## 变更原因

用户反馈问题 2（2026-09-04，p0-fix 暂留待办）：**产品声称支持 PostgreSQL 和 MySQL，但没有交互界面和逻辑**。

现状（代码勘察结论）：
- `Database` 枚举（Generic/Mysql/Postgresql/Sqlite/Mssql/Oracle）与 `store.database` 信号**已存在**，且已随图序列化到后端（`editor_data_access.rs` 双向映射完备）——但**没有任何 UI 可以切换它**，永远停在 `Generic`
- `ExportDrawer` 的引擎下拉（`export-engine-select`）是独立信号，默认硬编码 `"mysql"`，与图级 database **不联动**
- Inspector 字段类型下拉（`inspector-field-type`）硬编码 5 个通用类型，不按引擎过滤（MySQL 没有 `BOOLEAN` 原生类型、PostgreSQL 用 `SERIAL` 等差异被抹平）

## 变更类型

**代码级**（前端 UI/联动补全：数据模型与序列化通路已存在，只接 UI；无 API/DB/部署变更）

## 变更范围

- 影响的需求文档：无（用户反馈驱动，p0-fix 提案「不在范围」条目转出）
- 影响的功能规格：`frontend-rs/src/editor_panels.rs`（AppBar / Inspector 字段类型 / ExportDrawer）；`frontend-rs/src/editor_core.rs`（类型清单纯函数）
- 影响的业务场景：S01 画布编辑（字段类型编辑 + SQL 导出）
- 影响的 API：无（database 字段已在既有 PUT /diagrams/{id} 序列化通路内）
- 影响的 DB 表：无
- 影响的编排测试：`frontend-rs/scripts/test-spec-parity-d.mjs`（新增 ST-DB-01）

## 部署影响

- 是否需要部署：**否**（纯前端交互补全，无后端变更）
- 部署原因：仅前端 Rust/WASM 代码变更
- 影响环境：无
- 是否涉及数据迁移：否（database 字段已在序列化 schema 内，默认 Generic 向后兼容）
- 是否需要回滚预案：否
- 是否需要 smoke：否

## 变更概述

**图级 database 切换 + 类型按引擎过滤 + 导出默认跟随**

1. **工具栏切换**：AppBar 加数据库引擎下拉（Generic / MySQL / PostgreSQL），切换 → `store.database.set` + dirty + PUT 落账
2. **字段类型按引擎过滤**：`types_for_database(engine)` 纯函数返回该引擎类型清单（Generic/MySQL/PostgreSQL 三组），Inspector 字段类型下拉按当前图引擎渲染选项
3. **导出默认跟随**：ExportDrawer 引擎下拉初始值改为读 `store.database`（用户仍可手动改），CodeView SQL 同样跟随图引擎

## 不在范围

- Sqlite/Mssql/Oracle 的 UI 暴露（枚举保留，UI 只出 Generic/MySQL/PostgreSQL 三项）
- 类型转换迁移（切换引擎后既有字段类型原样保留，不做映射改写）
- 引擎专属 DDL 深度定制（分区/表空间/存储引擎等）
- 导出引擎下拉与图引擎的双向绑定（导出内手改不回写图）

## 验收标准

- AppBar 引擎下拉切换 MySQL → Inspector 字段类型下拉出现 MySQL 组（含 `DATETIME`），无 `SERIAL`；切 PostgreSQL → 出现 `SERIAL`/`TIMESTAMP`；保存落账 `database` 字段
- ExportDrawer 打开 → 引擎下拉默认 = 图当前引擎；SQL 预览用该引擎方言
- Generic 图行为与现状一致（回归不破）
