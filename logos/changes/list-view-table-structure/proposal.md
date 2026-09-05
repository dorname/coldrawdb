# 变更提案：list-view-table-structure

> module: core | created: 2026-09-05

## 变更原因

用户反馈问题 1（2026-09-04，p0-fix 暂留待办）：**产品没有类似 PDManer 的纯列表表结构视图**。

现状：ListView（`SidePanelTab::ListView`）已具备字段行列表 + 排序/过滤/分组基础设施，但分组仅支持 `GroupByMode::None`（扁平）/ `ByTag`（按字段 tag）。**缺少「按表分组」模式**——无法像 PDManer 那样按表一览字段结构（表名 → 该表全部字段明细）。

## 变更类型

**代码级**（前端 UI 增强：复用既有 ListView/group_tables 通路，新增一个分组模式；无 API/DB/部署变更）

## 变更范围

- 影响的需求文档：无（用户反馈驱动，p0-fix 提案「不在范围」条目转出）
- 影响的功能规格：`frontend-rs/src/editor_panels.rs`（GroupByMode / group_tables / ListView 分桶渲染）
- 影响的业务场景：S01 画布编辑（ListView tab）
- 影响的 API：无
- 影响的 DB 表：无
- 影响的编排测试：`frontend-rs/scripts/test-spec-parity-d.mjs`（新增 ST-LV-01）

## 部署影响

- 是否需要部署：**否**（纯前端交互增强，无后端变更）
- 部署原因：仅前端 Rust/WASM 代码变更（frontend-rs/src/editor_panels.rs）
- 影响环境：无
- 是否涉及数据迁移：否
- 是否需要回滚预案：否
- 是否需要 smoke：否

## 变更概述

**ListView 新增「按表分组」模式（PDManer 式表结构清单）**

- `GroupByMode` 新增 `ByTable` 变体；`group_tables` 按 Table 分桶（桶键 = 表名，空表也要出桶头）
- ListView 分组下拉新增「按表分组」选项；桶头行显示**表名 + 字段数**，桶内字段行沿用既有渲染
- 桶头行点击 → 选中该表（复用 `on_select_table` 通路，同步 Inspector）
- 默认分组模式改为 `ByTable`（打开列表视图即见 PDManer 式表结构清单）

## 不在范围

- 列表入口强化（键盘快捷键 L / 浮动返回按钮）——待本提案闭环后单独评估
- 桶内字段行内联编辑（沿用既有 Inspector 编辑通路）
- 桶折叠/展开交互
- 问题 2（图级 PostgreSQL/MySQL 类型）

## 验收标准

- 打开 ListView → 默认按表分组：每张表一个桶头行（表名 + 字段数），桶内列出该表全部字段
- 空表（无字段）也显示桶头行
- 点击桶头行 → 选中该表 + Inspector 打开
- 分组下拉切回「不分组 / 按字段 tag 分组」→ 行为与现状一致（回归不破）
