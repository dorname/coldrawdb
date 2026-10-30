# 合并指令

## 变更提案
- 提案名称：fix-import-camel-case-compatibility
- 提案目录：logos/changes/fix-import-camel-case-compatibility/

## 提案内容

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


## 需要合并的 Delta 文件

### 1. deltas/api/diagrams.yaml

- Delta 文件：`logos/changes/fix-import-camel-case-compatibility/deltas/api/diagrams.yaml`
- 目标目录：`logos/resources/api/`
- 操作：读取 delta 中的 ADDED / MODIFIED / REMOVED 标记，合并到目标目录中对应的主文档

### 2. deltas/prd/1-product-requirements/core-01-requirements.md

- Delta 文件：`logos/changes/fix-import-camel-case-compatibility/deltas/prd/1-product-requirements/core-01-requirements.md`
- 目标目录：`logos/resources/prd/1-product-requirements/`
- 操作：读取 delta 中的 ADDED / MODIFIED / REMOVED 标记，合并到目标目录中对应的主文档

### 3. deltas/prd/2-product-design/1-feature-specs/core-02-diagram-persistence.md

- Delta 文件：`logos/changes/fix-import-camel-case-compatibility/deltas/prd/2-product-design/1-feature-specs/core-02-diagram-persistence.md`
- 目标目录：`logos/resources/prd/2-product-design/1-feature-specs/`
- 操作：读取 delta 中的 ADDED / MODIFIED / REMOVED 标记，合并到目标目录中对应的主文档

### 4. deltas/test/core-S01-test-cases.md

- Delta 文件：`logos/changes/fix-import-camel-case-compatibility/deltas/test/core-S01-test-cases.md`
- 目标目录：`logos/resources/test/`
- 操作：读取 delta 中的 ADDED / MODIFIED / REMOVED 标记，合并到目标目录中对应的主文档

## 执行要求

1. 逐个 Delta 文件处理，每处理完一个报告修改摘要
2. 对于 ADDED 标记：在主文档的指定位置插入新内容
3. 对于 MODIFIED 标记：替换主文档中同名章节的内容
4. 对于 REMOVED 标记：从主文档中删除对应章节
5. 保持主文档的原有格式和风格
6. 如果主文档有"最后更新"时间戳，同步更新
7. 所有变更完成后，列出修改清单
8. 所有变更合并完成后，自动执行 git commit（告知用户，无需确认）：
   git add -A && git commit -m "docs(fix-import-camel-case-compatibility): merge spec deltas"
   然后提示用户：按更新后的规格实现代码，代码完成后运行 `openlogos verify` 验收，验收通过后明确授权执行 `openlogos archive fix-import-camel-case-compatibility`。
