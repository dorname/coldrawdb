# 变更提案：修复 MCP Schema 导出保真度

> module: core | created: 2026-10-30 | source: GitHub Issue #55

## 变更原因

`export_schema` 当前对 SQL 逐列追加 `PRIMARY KEY`，复合主键表会生成多个列级主键的非法 DDL；同时缺少表级外键、级联动作与注释输出，DBML 也只保留字段类型和单列 pk。现有 UT-MCP-06 / ST-MCP-02 仅检查“非空、确定”，因此非法或语义不完整的导出仍能通过验收。

该问题直接阻断 MCP「设计 → 导出 → 建库 / 评审」主链路。需要把既有 US-S06-03 / FR-S06-05 的“完整 diagram 导出”细化为可验证的 schema 保真契约。

## 变更类型

设计级修复（导出内容契约细化 + 本地序列化实现 + 测试）。

## 变更范围

- 影响的需求文档：`core-S06-mcp-service-requirements.md` FR-S06-05 与 §5 导出验收
- 影响的功能规格：`core-S06-mcp-service-design.md` §4.1，补充 SQL / DBML 的约束、FK、级联和注释口径
- 影响的业务场景：S06 导出辅时序；参与者与 API 调用顺序不变，无需修改时序图
- 影响的部署方案：无
- 影响的 API：`export_schema` 输入 / 输出 envelope 与 `content` 类型不变；`mcp-tools.yaml` 无需变更
- 影响的 DB 表：无 Schema 变更、无数据迁移
- 影响的编排测试：无
- 影响的测试用例：`core-S06-test-cases.md` UT-MCP-06 / ST-MCP-02，增加复合主键、约束、FK、级联、注释与确定性断言
- 影响的 smoke 测试：无

## 部署影响

- 是否需要部署：否
- 部署原因：MCP 本地纯函数与自动化测试修复，不执行环境部署
- 影响环境：无
- 是否涉及数据迁移：否
- 是否需要回滚预案：否
- 是否需要 smoke：否

## 变更概述

SQL 导出以 PostgreSQL 为 Issue 验收主路径：单列主键保持列内声明，复合主键改为唯一表级 `PRIMARY KEY (...)`；字段输出 NOT NULL / UNIQUE / DEFAULT，关系输出表级 FOREIGN KEY 与 ON DELETE / ON UPDATE，表和字段注释按方言输出。其余 SQL 格式共享约束和 FK 修复；MySQL / MariaDB 使用行内及表选项 COMMENT，SQLite / MSSQL 不生成不兼容的 COMMENT ON，不在本期扩展 MSSQL extended property。

DBML 输出单 / 复合主键、NOT NULL、UNIQUE、DEFAULT、字段与表 note，以及带级联动作的 Ref。无法解析端点的悬空关系不得输出 `?` 伪引用，直接跳过并保持确定性。JSON envelope、revision、MIME 类型、纯函数边界与其他工具保持不变；不引入新依赖或数据库 / 网络副作用。

测试沿用 UT-MCP-06 / ST-MCP-02，以含复合主键、约束、注释和关系的完整 fixture 验证 PostgreSQL 与 DBML 语义，并复测七类 SQL 的确定性及非法重复列级主键不再出现。
