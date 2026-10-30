# 实现任务

## [delta] 规格变更

- [x] 产出 `deltas/prd/1-product-requirements/core-S06-mcp-service-requirements.md` — 细化 FR-S06-05 与导出验收的 schema 保真要求
- [x] 产出 `deltas/prd/2-product-design/1-feature-specs/core-S06-mcp-service-design.md` — 定义 SQL / DBML 的单复合主键、约束、FK、级联、注释与悬空关系口径
- [x] 产出 `deltas/test/core-S06-test-cases.md` — 强化 UT-MCP-06 / ST-MCP-02 的完整 fixture 与断言

## [code] 代码实现

- [x] 修改 `mcp-server/src/export.rs`，实现 SQL / DBML 保真导出且不改变现有 envelope 和确定性
- [x] 强化 `mcp-server/tests/c2_read_tools.rs` 的 UT-MCP-06 / ST-MCP-02，覆盖 PostgreSQL、DBML、七类 SQL 与 OpenLogos reporter
- [x] 运行 MCP 相关 UT / ST 与格式检查，保留导出样例和测试结果
