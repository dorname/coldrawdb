# 变更提案：fix-diagram-entity-persistence

> module: core | created: 2026-06-21

## 变更原因

编辑器 V2 布局已就绪，但核心数据流仍为 stub：保存/加载/分享/导入后画布变空；冲突弹窗、撤销/重做、New 模态未接线；Inspector 7 Tab 与 IO 抽屉 testid 与原型不一致。

## 变更类型

代码级（含部分接口行为对齐 OpenAPI 已有规格）

## 变更范围

- 影响的需求文档：Phase 2 编辑器原型、Phase 3 diagrams/bridge 场景
- 影响的功能规格：diagram 持久化、IO 导入、冲突处理、撤销栈
- 影响的业务场景：S02 分享加载、bridge 导入、自动保存
- 影响的 API：`GET/PUT /diagrams/{id}`、`POST /bridge/import-local-draft`
- 影响的 DB 表：diagram / table / field / reference / area / note / diagram_link
- 影响的编排测试：导入抽屉 E2E、保存/冲突相关 ST

## 部署影响

- 是否需要部署：是
- 部署原因：前后端行为变更，需重启本地/测试环境服务
- 影响环境：本地
- 是否涉及数据迁移：否（SQLite schema 不变）
- 是否需要回滚预案：否（可 revert 代码）
- 是否需要 smoke：是（导入 + 分享链接 smoke）

## 变更概述

**Batch 1**：后端 `diagram_persistence` 模块实现 GET/PUT/import 完整嵌套实体往返；前端 `DiagramForSave` 发送完整 diagram。

**Batch 2**：bridge 导入在 INSERT diagram 后调用 `persist_import_payload`；前端 SQL/DBML import payload 含 `tables` 数组。

**Batch 3**：冲突弹窗 force/reload 接线；CommandStack undo/redo 接 AppBar + 快捷键；New/Rename 模态接 create/rename。

**Batch 4**：AppBar 原型顺序；Inspector 挂载 LeftPanel 7 Tab + RightPanel；统一 `io-drawer` / `share-url` / `side-search` / `field-editor` testid。
