# Delta 索引与合并顺序

> proposal: align-prototype-docs-implementation | module: core | 2026-08-19

## 1. 合并顺序

1. `scenario/core-S03-user-auth.json`：先补齐 S03 API 编排测试，锁定 auth register/login/me/refresh/logout 的真实链路。
2. `test/core-V2-production-frontend-test-cases.md`：新增生产前端接入测试矩阵，建立代码批次与 OpenLogos reporter 的用例来源。
3. `reference/implementation/core-implementation-checklist.md`：把 V2 生产前端待接入项拆成 A/B/C/D 四个实现批次。
4. `reference/implementation/core-frontend-alignment-acceptance.md`：新增验收说明，明确统一原型是视觉交互基线，生产验收必须调用真实 API/WS。

## 2. 关键决策

- 不重开 S03/S04/S05 的需求、产品设计、API 或数据库设计；本变更只把生产实现补齐到已合并规格。
- 统一主原型仍为 `core-01-editor-prototype.html`；历史 S03/S04/S05 独立原型不作为验收入口。
- 后端 auth/rooms/collab 已有实现，源码阶段只允许修契约兼容、错误码、CORS/WS 握手和测试缺口。
- 前端实现必须保持 S01/S02 单人保存、分享加载、409 冲突、导入导出与设计系统不回退。
- 每个代码批次必须先列出覆盖用例 ID，再同时交付业务代码、测试和 OpenLogos reporter。

## 3. Delta 文件统计

| 类别 | 文件 |
|---|---:|
| 编排测试 | 1 |
| 测试矩阵 | 1 |
| 实现清单/验收参考 | 2 |
| Delta 索引 | 1 |

## 4. 合并后代码批次

| 批次 | 范围 | 主要用例 |
|---|---|---|
| A | S03 鉴权生产接入 | `UT-S03-01`～`UT-S03-07`、`ST-S03-01`、`ST-FE-S03-01`～`ST-FE-S03-05` |
| B | S04 房间与邀请生产接入 | `UT-S04-01`～`UT-S04-10`、`ST-S04-01`、`ST-FE-S04-01`～`ST-FE-S04-06` |
| C | S05 WS/OT/presence 生产接入 | `UT-C-01`～`UT-C-05`、`ST-C-01`、`ST-FE-S05-01`～`ST-FE-S05-06` |
| D | 全链路回归与状态收口 | `ST-FE-V2-01`～`ST-FE-V2-04`、S01/S02/PU 回归 |
