# 变更提案：listview-tree-master-detail

> module: core | created: 2026-09-07

## 变更原因

用户真机反馈：列表视图（ListView）当前为「全量表纵向堆叠」的单张编辑网格——所有表以组头行串联在一个超长 tbody 中。表数量增多时，定位目标表/字段需要长距离滚动，体验差。经方案比选（A. 表树+单表网格 / B. 全量网格增强 / C. 树+全部模式），用户确认采用**方案 A：左侧表树 + 右侧单表字段网格**（master-detail，对齐 PDManer/Navicat 标准形态）。

## 变更类型

设计级变更（功能规格 + 原型 + 测试用例 + 代码）

## 变更范围

- 影响的需求文档：无（core-01-requirements.md 仅指针性引用 §10.5，章节号不变）
- 影响的功能规格：
  - `core-04-side-panel-tabs.md`（§10.5 重写：全量分组网格 → 左侧表树 + 右侧单表字段明细网格）
- 影响的原型：`core-01-editor-prototype.html`（**先行**：renderListView 重构为树+单表布局，新增树面板样式与交互）
- 影响的业务场景：SP（侧栏/列表视图）
- 影响的部署方案：无
- 影响的 API：无
- 影响的 DB 表：无
- 影响的编排测试：无
- 影响的 ST/UT 用例：
  - `core-SP-side-panel-test-cases.md`：
    - ST-SP-LIST-01（MODIFIED：全屏渲染断言改为树+单表网格布局，组头行断言移除）
    - ST-SP-LIST-02（MODIFIED：行内编辑落账路径改为「树选表 → 网格选行」）
    - 新增 ST-SP-LIST-03（表树搜索过滤 + 切表渲染 + 空态）
  - `core-UI-modals-2-test-cases.md`：
    - ST-LV-01（MODIFIED：e2e 断言从「两表各出组头行」改为「树切换渲染对应单表网格」）
    - UT-MM-22（MODIFIED：ListView tab 切换断言同步树布局）
    - 新增 UT（表树过滤纯函数复用口径/选中表状态纯函数，具体编号产出 delta 时按既有序号续编）
  - UT-MM-21/23/24/26/27（sort/filter/batch_rename/batch_change_types/export_csv 纯函数）：不受本变更影响，保持既有口径（filter_tables 纯函数由表树搜索复用）
- 影响的 smoke 测试：无

## 部署影响

- 是否需要部署：否
- 部署原因：纯前端交互重设计（ListView 布局与导航），无后端/部署方案变更
- 影响环境：无
- 是否涉及数据迁移：否（数据模型不变，仅渲染与选中态组织方式变化）
- 是否需要回滚预案：否
- 是否需要 smoke：否

## 变更概述

1. **ListView 布局重构（先原型后代码）**：从「全量表组头堆叠单网格」改为左右 master-detail 两栏——
   - 左侧表树（窄栏）：顶部搜索框（复用 `filter_tables` 名称模糊匹配口径）；树节点显示表名 + 字段数；单击选中该表（Inspector 同步，对齐原组头单击行为）；双击跳回画布并选中该表（对齐原组头双击行为）。
   - 右侧单表网格：只渲染当前选中表的字段明细，10 列行内编辑（字段代码/显示名称/主键/不为空/唯一/自增/数据类型/长度/小数位/说明）完全复用现有 `ListFieldRow` 行组件与受控输入口径；网格顶部显示当前表名标题；工具条（增字段/删字段/上移/下移/返回画布）不变，增字段目标从「选中表优先否则首表」简化为「当前树选中表」。
   - 默认选中首张表；清空全部表后展示空态；切换表时清空字段行选中态（`list_sel`）。
2. **删除组头行机制**：`ListRowItem::Header` / `ListGroupHeader` 及其 testid 契约（`list-view-group-*`）随全量网格一并移除，相关 e2e 锚点改为树节点锚点（如 `list-tree-node-<table_name>`）。
3. **不变部分**：字段行内编辑落账链路、参数化类型解析/合成（`parse_field_type`/`compose_field_type`）、删字段级联清理关系、Command 快照撤销、列宽会话态（ColumnWidths）等既有能力全部保持。
