# 变更提案：refresh-editor-prototype

> module: core | created: 2026-06-20

## 变更原因

用户要求按 **product-designer** Skill 重新设计项目原型，并以 **drawdb `origin/main` 分支**（`Workspace.jsx` / `ControlPanel.jsx` / `EditorSidePanel`）为 UI 事实锚点。现有 `core-01-editor-prototype.html` 存在 delta merge 残留（`</html>` 后追加片段），无法作为可交互原型使用；且缺少按场景（S01/S02）组织的交互设计文档。

## 变更类型

设计级变更（Phase 2 产品设计层）

## 变更范围

- 影响的需求文档：无（需求不变，仅细化交互）
- 影响的功能规格：无结构性变更（引用既有 `core-00` ~ `core-0c`）
- 影响的业务场景：**S01**（编辑保存）、**S02**（分享加载）、**S03**（鉴权，V2）、**S04**（协作房间，V2）、**S05**（OT 实时协作，V2）
- 影响的 API：无
- 影响的 DB 表：无
- 影响的编排测试：无

## 部署影响

- 是否需要部署：**否**（仅规格/原型文档）
- 部署原因：文档变更，不涉及运行时
- 影响环境：无
- 是否涉及数据迁移：否
- 是否需要回滚预案：否
- 是否需要 smoke：否

## 变更概述

1. **重建** `core-01-editor-prototype.html`：单一完整 HTML，对齐 drawdb main 布局（ControlPanel + SidePanel 7 Tab + Canvas 表渲染）与 coldrawdb V2 栅格（AppBar / ToolRail / Inspector / IO Drawer / StatusBar），使用 `core-07` token（`#175e7a` 主色）。
2. **新增** S01/S02 场景交互设计文档，含交互级 GIVEN/WHEN/THEN 与原型锚点（`data-testid`）。
3. **更新** `logos-project.yaml` `resource_index` 索引新文档。
4. **新增** S03 鉴权交互设计 + HTML 原型（登录 / 注册 / Token 续期演示；drawdb main 无鉴权，为 coldrawdb V2 net-new）。
5. **新增** S04 协作房间交互设计 + HTML 原型（创建 / 邀请 / 加入 / 成员管理；依赖 S03，为 S05 OT 前置）。
6. **新增** S05 OT 实时协作交互设计 + HTML 原型（WS 连接 / presence / 远端 op / 重连；依赖 S04）。
7. **新增** S03 Phase 3 时序图 + S03 API（auth.yaml）+ DB（coldrawdb-v2-auth.sql）。
8. **新增** S04 Phase 3 时序图 + DB 预埋（coldrawdb-v2-rooms.sql）+ `rooms.yaml` + 编排测试 `core-S04-room-lifecycle.json`。
9. **新增** S05 Phase 3 时序图 `core-S05-ot-collab.md`（WS/OT/重连/checkpoint）。
