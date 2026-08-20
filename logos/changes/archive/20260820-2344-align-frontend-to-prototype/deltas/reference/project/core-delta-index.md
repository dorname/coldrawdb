# Delta Index — align-frontend-to-prototype

> module: core | proposal: align-frontend-to-prototype

## 1. 本次 delta 文件

1. `reference/implementation/core-frontend-alignment-acceptance.md`：新增 FEUX-AC-01～FEUX-AC-08，定义生产前端继续对齐主原型的页面流验收标准。
2. `test/core-V2-production-frontend-test-cases.md`：新增 `UT-FE-PROTO-01`～`UT-FE-PROTO-09` 与 `ST-FE-PROTO-01`～`ST-FE-PROTO-08`。
3. `reference/implementation/core-implementation-checklist.md`：新增 7.6 生产前端原型对齐收口项，并区分“API 接入已完成”和“体验对齐待收口”。

## 2. 不变项

- 不新增需求文档。
- 不新增 API、DB 或后端场景编排 JSON。
- 不修改统一主原型 `core-01-editor-prototype.html`；它继续作为视觉与交互基线。
- 不改变 S03/S04/S05 已合并 API/DDL/后端语义。

## 3. 批次与测试映射

| 批次 | 范围 | 用例 |
|---|---|---|
| A | Auth 与 Invite 页面流 | `UT-FE-PROTO-01`、`UT-FE-PROTO-02`、`ST-FE-PROTO-01`、`ST-FE-PROTO-02` |
| B | Rooms 列表页 | `UT-FE-PROTO-03`、`UT-FE-PROTO-04`、`ST-FE-PROTO-03`、`ST-FE-PROTO-04` |
| C | Collab Editor 可见状态与响应式 | `UT-FE-PROTO-05`、`UT-FE-PROTO-06`、`ST-FE-PROTO-05`、`ST-FE-PROTO-06`、`ST-FE-PROTO-07` |
| D | 全链路回归 | `ST-FE-PROTO-08`、`ST-FE-V2-01`～`ST-FE-V2-04`、ST-PU 回归 |
