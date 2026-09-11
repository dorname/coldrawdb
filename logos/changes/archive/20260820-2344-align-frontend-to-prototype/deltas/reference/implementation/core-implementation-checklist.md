# Delta — core-implementation-checklist.md（生产前端原型对齐收口）

> module: core | proposal: align-frontend-to-prototype

## ADDED — 7.6 生产前端继续对齐主原型

- [ ] 批次 A：Auth 与 Invite 页面流对齐，覆盖 `UT-FE-PROTO-01`、`UT-FE-PROTO-02`、`ST-FE-PROTO-01`、`ST-FE-PROTO-02`。
- [ ] 批次 B：Rooms 列表页对齐，覆盖 `UT-FE-PROTO-03`、`UT-FE-PROTO-04`、`ST-FE-PROTO-03`、`ST-FE-PROTO-04`。
- [ ] 批次 C：Collab Editor 可见状态与响应式对齐，覆盖 `UT-FE-PROTO-05`、`UT-FE-PROTO-06`、`ST-FE-PROTO-05`、`ST-FE-PROTO-06`、`ST-FE-PROTO-07`。
- [ ] 批次 D：全链路回归与状态收口，覆盖 `ST-FE-PROTO-08`、`ST-FE-V2-01`～`ST-FE-V2-04` 回归、ST-PU 回归。

## CHANGED — 9.2 V2 文档与实现状态说明

将“V2 生产前端实现（auth / rooms / collab REST + OT 状态 + presence，按 A/B/C 批次交付）”扩展为：

- [x] V2 生产前端 API 接入（auth / rooms / collab REST + OT 状态 + presence，`align-prototype-docs-implementation` 已完成）
- [ ] V2 生产前端体验对齐统一主原型页面流（auth → rooms → editor、invite 独立页、协作状态可见性，`align-frontend-to-prototype` 收口）
