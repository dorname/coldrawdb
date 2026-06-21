# 变更提案：add-v2-collab-spec

> module: core | created: 2026-06-20

## 变更原因

S05 OT 实时协作 Phase 3 时序图（`core-S05-ot-collab.md`）已完成 WS 协议推导，但缺少 OpenAPI 规格与 DDL，阻塞 V2 collab-server 实现与编排测试设计。

## 变更类型

接口级 + 设计级（规格文档）

## 变更范围

- 影响的需求文档：无
- 影响的功能规格：无
- 影响的业务场景：`core-S05-ot-collab.md` §10–§11 引用更新
- 影响的 API：新增 `logos/resources/api/collab.yaml`
- 影响的 DB 表：新增 `operation` / `operation_log` / `room_collab_head`（`coldrawdb-v2-collab.sql`）
- 影响的编排测试：新增 `core-S05-ot-collab.json`（本次产出）

## 部署影响

- 是否需要部署：否（纯规格）
- 部署原因：无代码变更
- 影响环境：无
- 是否涉及数据迁移：否（DDL 规格，实现时追加）
- 是否需要回滚预案：否
- 是否需要 smoke：否

## 变更概述

1. 新增 `collab.yaml`：WebSocket 升级路径 + 2 个 REST 辅助端点（head / ops）+ WS 帧 schema + checkpoint 请求头约定。
2. 新增 `coldrawdb-v2-collab.sql`：`operation`、`operation_log`、`room_collab_head` 三表。
3. 更新场景概览与 `logos-project.yaml` resource_index。
