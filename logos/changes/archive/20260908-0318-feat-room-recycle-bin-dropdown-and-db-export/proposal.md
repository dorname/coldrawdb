# 变更提案：房间回收站（含硬删除）+ 下拉列表 UI 修复 + 数据库连接导出执行

> module: core | created: 2026-09-08

## 变更原因

用户目标（2026-09-08，三项）：

1. **房间回收站**：房间删除目前是软删除（`DELETE /rooms/{id}` 写 `archived_at`，diagram 保留），但数据虽在、**没有任何找回入口**——归档房间从列表消失后即不可达。用户要求提供回收站找回能力，同时支持硬删除（彻底物理删除）。另外编辑器内 DeleteRoomModal 文案「删除后不可恢复，房间及其所有协作数据将被永久删除」（editor_panels.rs:10695）与软删除实际行为矛盾，属误导性文案，需一并修正。
2. **下拉列表 UI**：导入抽屉等处使用原生 `<select>`（`.cdb-form-select` / `.cdb-select`），样式极简——无 `appearance` 处理、无自定义箭头、无暗色 `color-scheme` 配套、无 `option` 配色；叠在 `overflow:hidden` + 入场动画 transform 的抽屉容器上，用户实测「仿佛有一半内容展示不出来」。需系统性修复表单下拉控件的视觉。
3. **数据库连接导入导出继续优化**：导入侧已有 `POST /api/v1/bridge/import/connect`（SQLite/PG introspection）；导出侧只有复制/下载 DDL 文本，缺少对称的「连接数据库执行导出」能力。本变更补齐导出侧：新增 `POST /api/v1/bridge/export/execute`，在目标真实库执行生成的 DDL。

## 变更类型

设计级变更（新交互入口 + 新 API 端点 + 样式缺陷修复；room 表结构不变，`archived_at` 字段已存在，无 DB 迁移）

## 变更范围

- 影响的需求文档：无（S04 房间生命周期能力增强，不新增场景编号）
- 影响的功能规格：
  - `core-S04-room-lifecycle-design.md` — 房间生命周期状态机扩展（archived → restored / purged）、回收站交互、删除文案修正
  - `core-01d-import-export.md` — ExportDrawer 新增「导出到数据库」区块；ImportDrawer/ExportDrawer 下拉控件样式条款
  - `core-03-bridge-io.md` — 新增 §14 连接数据库导出执行（端点契约、执行口径、安全边界）
  - `core-09-core-components.md` — 新增表单下拉（select）控件视觉条款
- 影响的业务场景：S04（房间生命周期）、S01 / Phase C（导入导出）
- 影响的部署方案：无
- 影响的 API：
  - `api/rooms.yaml` — `GET /api/v1/rooms` 增加 `archived` 查询参数（回收站列表）；新增 `POST /api/v1/rooms/{roomId}/restore`；新增 `DELETE /api/v1/rooms/{roomId}/permanent`
  - `api/bridge.yaml` — 新增 `POST /api/v1/bridge/export/execute`
- 影响的 DB 表：无结构变更（`room.archived_at` 已存在；硬删除走 `DELETE`，`room_member`/`room_invite` 已有 `ON DELETE CASCADE`，diagram 保留）
- 影响的编排测试：无（非 API 编排链路变更，遵循既有惯例）
- 影响的 smoke 测试：无
- 影响的测试用例文档：
  - `core-S04-test-cases.md` — 新增回收站 UT（恢复 / 硬删除 / 权限 / 恢复冲突 409）与 e2e 锚点用例
  - `core-PC-import-export-test-cases.md` — 新增导出执行 UT（SQLite/PG 真实库）与前端锚点、e2e 用例
  - `core-PE-design-system-test-cases.md` — 下拉控件样式锚点用例（如适用）

## 部署影响

- 是否需要部署：否
- 部署原因：开发阶段功能增强，本地重新构建即生效；无新部署物、无配置变更、无数据迁移（遵循近期后端端点变更惯例）
- 影响环境：无
- 是否涉及数据迁移：否
- 是否需要回滚预案：否
- 是否需要 smoke：否

## 变更概述

**特性 1：房间回收站。** 后端 rooms 模块新增三个能力：① `GET /api/v1/rooms?archived=true` 返回当前用户作为 owner 的已归档房间列表（回收站）；② `POST /api/v1/rooms/{roomId}/restore` 清除 `archived_at` 恢复房间——若该 diagram 已绑定其他 active 房间（`idx_room_diagram_active` 部分唯一索引冲突）返回 409；③ `DELETE /api/v1/rooms/{roomId}/permanent` 物理删除已归档房间（仅 owner、仅 archived 状态可删，防止绕过回收站；`room_member`/`room_invite` 级联删除，diagram 保留）。前端房间列表页新增「回收站」入口与回收站视图：列出已归档房间（名称 + 归档时间），每行提供「恢复」「彻底删除」（二次确认模态，明确不可恢复）；修正 DeleteRoomModal 误导文案为「删除后进入回收站」语义。

**特性 2：下拉列表 UI 修复。** 重做 `.cdb-form-select` / `.cdb-select` 样式：`appearance: none` + 自定义 chevron 箭头（SVG 背景）、显式前景/背景色、`color-scheme` 与 `option` 配色适配暗色模式、合理 `line-height`/`padding` 消除文字裁切感。纯 CSS 修复，覆盖所有原生 select 使用点（ImportDrawer 引擎下拉、ExportDrawer 引擎下拉、创建房间 dbtype 下拉等）。

**特性 3：连接数据库导出执行。** 后端 `phase3_bridge` 新增 `POST /api/v1/bridge/export/execute`：请求携带 `{ engine, source, ddl }`，连接目标库（SQLite 文件 / PG 连接串）并执行 DDL，返回执行统计（执行语句数 / 建表数）；错误语义与 import/connect 对齐（400 参数类、502 连接或执行失败并附引擎错误消息）。前端 ExportDrawer SQL Tab 新增「导出到数据库」区块：引擎下拉 + 连接信息输入 +「在数据库中执行」按钮 + 结果反馈，DDL 复用预览区已生成内容。安全边界与 import/connect 一致：连接信息仅当次请求使用、不持久化、挂既有 auth 中间件。
