# 变更提案：兼容 drawDB camelCase 导入字段

> module: core | created: 2026-10-31 | issue: #54

## 变更原因

Issue #54 指出 drawDB 风格 JSON 使用 `startTableId` / `endTableId` / `startFieldId` / `endFieldId` 时，导入规范化只检查 snake_case，导致全部关系以 warnings 形式被丢弃但 HTTP 仍成功；字段的 `notNull: true` 也未映射到持久化 DTO 的 `not_null`，刷新后非空约束丢失。

snake_case 带关系导入 500 的全局 ID 冲突与空 details 已由 `fix-import-id-remapping-and-errors`（#62）修复，本提案不重复修改该部分，只保留回归验证。

## 变更类型

接口级变更（导入兼容性修复，无响应结构变化）。

## 变更范围

- 影响的需求文档：`core-01-requirements.md` FR-12。
- 影响的功能规格：`core-02-diagram-persistence.md` §5.1、§5.2。
- 影响的业务场景：S01 / JSON 导入。
- 影响的 API：`POST /api/v1/diagrams/import` 的 payload 字段兼容性。
- 影响的 DB 表：`reference`、`field`（仅正确映射既有列，不改 Schema）。
- 影响的编排测试：`core-S01-test-cases.md`。
- 不影响：snake_case 现有行为、ID 重映射、全局唯一约束、成功响应 envelope、其他 API。

## 部署影响

- 是否需要部署：否。
- 部署原因：本地后端兼容逻辑与回归测试，无部署或数据迁移。
- 影响环境：无。
- 是否涉及数据迁移：否。
- 是否需要回滚预案：否。
- 是否需要 smoke：否。

## 变更概述

导入规范化在检查关系完整性前，将四个 camelCase 端点字段补写到既有 snake_case 键；snake_case 已提供有效值时优先保留，缺失或无效时才使用 camelCase 别名。后续 ID 重映射、端点回填和 DTO 反序列化继续只走统一内部表示。

字段 DTO 为 `not_null` 增加 `notNull` 别名，关系 DTO 同时声明四个 camelCase 别名，保证规范化函数与直接 DTO 构造的契约一致。两种命名风格均持久化相同的关系和非空约束；两端点命名都缺失时仍按现有 warnings 逐条丢弃，不扩大静默成功范围。
