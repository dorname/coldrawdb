# 变更提案：p0-fix

> module: core | created: 2026-09-04

## 变更原因
用户反馈 6 项问题（2026-09-04），P0 三项切片：
1. **协作房间无法删除**（问题 6）：后端 DELETE /rooms/{room_id} 已实现（archive_room 返回 204），但前端无 UI 调用入口——RoomPanel 仅成员管理，无「删除房间」按钮
2. **区域（Areas）/ 便签（Notes）不可用**（问题 3）：tool-new-area/tool-new-note 按钮存在但 on:click handler 缺失；Areas/Notes 渲染已有（draw_area/draw_note），store.areas/store.notes 信号已有，但无创建/编辑/删除交互
3. **关系连线交互冗余**（问题 4）：创建关系后弹确认条（RelToolState::Confirm），须点「创建」才落账；用户反馈「没有必要再多加一次创建 one to one 的交互」；画布点击连线无反应（应弹关系列表）；画布不能直接删除连线（应 Delete 键）

## 变更类型
**需求级**（UX 交互补全 + 冗余交互消除）

## 变更范围
- 影响的需求文档：`docs/drawdb-capability-checklist.md`（房间管理/区域便签/关系连线交互）
- 影响的功能规格：`frontend-rs/src/editor_panels.rs`（RoomPanel/RelToolState/AreasTab/NotesTab）
- 影响的业务场景：
  - 协作房间管理（删除房间）
  - 画布编辑（区域/便签创建/编辑/删除）
  - 关系连线（创建简化/点击详情/删除）
- 影响的 API：`DELETE /rooms/{room_id}`（已有，前端未调用）
- 影响的 DB 表：无（archive_room 已有）
- 影响的编排测试：`frontend-rs/scripts/test-spec-parity-d.mjs`（ST-PU-26 已 PASS；新交互待补）

## 部署影响
- 是否需要部署：**否**（纯前端交互补全，无后端变更）
- 部署原因：仅前端 Rust/WASM 代码变更（frontend-rs/src/editor_panels.rs + editor_render.rs + editor_data_access.rs）
- 影响环境：本地 / 测试 / 预发 / 生产（前端静态资源部署）
- 是否涉及数据迁移：**否**
- 是否需要回滚预案：是（前端静态资源可回滚到上一版本）
- 是否需要 smoke：是（部署后 smoke 测试覆盖 P0 三项交互）

## 变更概述

**定点 1：协作房间删除（问题 6）**
- 前端 `RoomClient::delete_room` 方法（DELETE /rooms/{id}）
- RoomPanel 加「删除房间」按钮（Owner 可见）
- `ModalKind::DeleteRoom` 确认模态（显示房间名 + 成员数 + 警告文案）
- 删除后回 rooms 页 + 列表刷新

**定点 2：区域/便签交互（问题 3）**
- `tool-new-area` on:click（画布拖拽创建区域）
- `tool-new-note` on:click（画布点击放置便签）
- Inspector 加 Area/Note 编辑面板（name/comment/color + 删除按钮）
- Delete 键删除 Area/Note（store.areas/store.notes 移除 + dirty + PUT）

**定点 3：关系连线简化（问题 4）**
- 删 `RelToolState::Confirm` 状态（Dragging → 直接落账，无确认条）
- 点击连线 → 弹 `ModalKind::ReferenceDetail` 模态（显示详情 + 删除按钮）
- 画布直接删除连线（选中后 Delete 键）
- 右键菜单（可选 P1：连线右键弹「删除 / 编辑 cardinality」菜单）

## 实现顺序

1. **定点 1**（0.5 天）：RoomClient::delete_room + RoomPanel 按钮 + ModalKind::DeleteRoom
2. **定点 3**（1-2 天）：删 Confirm 状态 + 点击连线命中检测 + ModalKind::ReferenceDetail + Delete 键删除
3. **定点 2**（2-3 天）：tool-new-area/tool-new-note on:click + Inspector 编辑面板 + Delete 键删除

每步独立 commit，commit message 格式 `fix(<module>): ...` 或 `feat(<module>): ...`。

## 验收标准

- **定点 1**：Owner 进房间 → RoomPanel 显示「删除房间」按钮 → 点击 → 弹确认模态 → 确认 → 删除 → 回 rooms 页 → 列表刷新（该房间消失）
- **定点 2**：用户点「添加区域」→ 拖拽画布框选 → 松开 → Area 落账 + 画布显示色块 + Inspector 编辑面板；用户点「添加便签」→ 点击画布 → Note 落账 + 画布显示便签 + Inspector 编辑面板
- **定点 3**：用户拖字段 A.id 到字段 B.user_id 松开 → **直接落账**（无确认条）→ 画布显示连线；用户点击连线 → 弹 ReferenceDetail 模态（显示详情 + 删除按钮）；用户选中连线按 Delete → 连线移除 + 画布刷新