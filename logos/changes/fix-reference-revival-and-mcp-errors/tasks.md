# 实现任务

## [delta] 规格变更

- [x] 产出 `deltas/prd/2-product-design/1-feature-specs/core-02-diagram-persistence.md` — 定义软删除 reference 同 ID 重建和活跃行保护
- [x] 产出 `deltas/prd/2-product-design/1-feature-specs/core-S06-mcp-service-design.md` — 更新连接错误码、BASE_URL 诊断与错误 content 契约
- [x] 产出 `deltas/api/mcp-tools.yaml` — 登记 UPSTREAM_UNREACHABLE 与 network_connect 映射
- [x] 产出 `deltas/test/core-S01-test-cases.md` — 新增 UT-S01-16 软删除关系重建
- [x] 产出 `deltas/test/core-S06-test-cases.md` — 新增 UT-MCP-39、ST-MCP-17 并修正错误 structuredContent 旧断言

## [code] 代码实现

- [x] 修改 `backend/src/diagram_persistence.rs`，在保存事务中清理同 ID 软删除 reference 并实现 UT-S01-16
- [x] 修改 `mcp-server/src/api.rs`、`error.rs`、`protocol.rs`，实现连接诊断和 schema 安全错误结果
- [x] 更新 MCP 相关测试，交付 UT-MCP-39、ST-MCP-17 与 OpenLogos reporter，并运行后端/MCP 定向及全量测试（后端 98/98；MCP 全部通过）
