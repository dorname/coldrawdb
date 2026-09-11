# 变更提案：align-v1-areas-notes-store

> module: core | created: 2026-06-21

## 变更原因

前后端对齐诊断 **批次 A**：Inspector 的 Areas / Notes Tab 使用本地 `AreaStub` / `NoteStub`，与画布渲染及 `PUT /api/v1/diagrams/{id}` 使用的 `store.areas` / `store.notes` 双轨运行。用户在侧栏新增的区域/便签不会出现在画布，也不会持久化。

## 变更类型

设计级 + 代码级（前端数据流修复，无 API/DB 变更）

## 变更范围

- 影响的功能规格：`core-04-side-panel-tabs.md` §3/§5、`core-01-editor-canvas.md` areas/notes 数据源
- 影响的测试用例：`core-SP-side-panel-test-cases.md` — UT-SP-10 改用 `Area`/`Note` 类型
- 影响的业务场景：S01（编辑保存含 areas/notes）
- 影响的 API / DB / 编排测试：无

## 部署影响

- 是否需要部署：否
- 是否需要 smoke：否
- 是否涉及数据迁移：否

## 变更概述

1. `AreasTab` / `NotesTab` 改为读写 `EditorStore.areas` / `EditorStore.notes`。
2. 「+ 加区域/便签」写入 store 后 `dirty=true` 并触发 `schedule_save`。
3. 保留 Enums / Types 为 V1 仅前端 state（规格不变）。
