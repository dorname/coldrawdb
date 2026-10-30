# 变更提案：修复导入实体 ID 冲突与错误诊断

> module: core | created: 2026-10-30 | source: GitHub Issue #62

## 变更原因

`POST /api/v1/diagrams/import` 当前仅给缺失 ID 的实体补值，已有 table / field / reference / area / note ID 会原样写入全局唯一表。重复导入同一文件，或不同图使用相同实体 ID 时，INSERT 触发 UNIQUE 冲突并整体返回 500，导致新图无法创建。

同时，嵌套 DTO 反序列化使用 `.ok().unwrap_or_default()`，任一字段类型错误会把整组实体静默变成空数组；handler 又丢弃 `SaveDiagramError`，返回 `details: null`，用户无法判断是 ID 冲突还是 payload 类型错误。

## 变更类型

接口级修复（导入 ID 隔离语义 + 400/500 错误诊断 + 共享持久化实现与测试）。

## 变更范围

- 影响的需求文档：`core-01-requirements.md` FR-12，明确重复导入创建独立新图且错误可诊断
- 影响的功能规格：`core-02-diagram-persistence.md` §5.1，定义全局实体 ID 重映射、关系端点回填和错误行为
- 影响的业务场景：S01 导入辅链路；端点、参与者和调用顺序不变，无需修改时序图
- 影响的 API：`diagrams.yaml` `/api/v1/diagrams/import` 的 200/400/500 描述；请求与成功响应 Schema 不变
- 影响的 DB 表：无 Schema 变更、无数据迁移；仅改变新导入实体的 ID 写入值
- 影响的编排测试：无
- 影响的测试用例：`core-S01-test-cases.md` 新增 UT-S01-14、UT-S01-15、ST-S01-06
- 影响的 smoke 测试：无

## 部署影响

- 是否需要部署：否
- 部署原因：后端导入逻辑与自动化测试修复，不执行环境部署
- 影响环境：无
- 是否涉及数据迁移：否
- 是否需要回滚预案：否
- 是否需要 smoke：否

## 变更概述

导入始终代表“创建独立新图”。共享 `persist_import_payload` 在现有逐实体规范化之后，为保留的 table / field / reference / area / note 统一生成新的 `auto-` 全局唯一 ID，并用 table / field 的旧新映射回填 references 四类端点；不再把调用方提供的全局实体 ID 原样写入。关系、字段归属和各数组顺序保持不变，不修改既有图数据。

嵌套 tables / references / areas / notes / dictionaries 反序列化失败改为 `BadRequest`，import 返回 400 且 details 指明失败数组和 Serde 原因，不再出现“成功但 0 表”。其他持久化异常保留空图回收，记录脱敏 warning 日志，并在 500 details 中返回可诊断的数据库错误信息。

本提案不处理 #54 的 camelCase / notNull 兼容，也不处理 #57 的既有图 PUT 软删除同 ID 重写；两者保持独立 Issue。
