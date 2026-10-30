# 合并指令

## 变更提案
- 提案名称：fix-mcp-export-schema-fidelity
- 提案目录：logos/changes/fix-mcp-export-schema-fidelity/

## 提案内容

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


## 需要合并的 Delta 文件

### 1. deltas/prd/1-product-requirements/core-S06-mcp-service-requirements.md

- Delta 文件：`logos/changes/fix-mcp-export-schema-fidelity/deltas/prd/1-product-requirements/core-S06-mcp-service-requirements.md`
- 目标目录：`logos/resources/prd/1-product-requirements/`
- 操作：读取 delta 中的 ADDED / MODIFIED / REMOVED 标记，合并到目标目录中对应的主文档

### 2. deltas/prd/2-product-design/1-feature-specs/core-S06-mcp-service-design.md

- Delta 文件：`logos/changes/fix-mcp-export-schema-fidelity/deltas/prd/2-product-design/1-feature-specs/core-S06-mcp-service-design.md`
- 目标目录：`logos/resources/prd/2-product-design/1-feature-specs/`
- 操作：读取 delta 中的 ADDED / MODIFIED / REMOVED 标记，合并到目标目录中对应的主文档

### 3. deltas/test/core-S06-test-cases.md

- Delta 文件：`logos/changes/fix-mcp-export-schema-fidelity/deltas/test/core-S06-test-cases.md`
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
   git add -A && git commit -m "docs(fix-mcp-export-schema-fidelity): merge spec deltas"
   然后提示用户：按更新后的规格实现代码，代码完成后运行 `openlogos verify` 验收，验收通过后明确授权执行 `openlogos archive fix-mcp-export-schema-fidelity`。
