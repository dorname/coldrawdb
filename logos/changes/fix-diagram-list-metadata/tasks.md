# 实现任务

## [delta] 规格变更

- [x] 产出 `deltas/prd/1-product-requirements/core-S06-mcp-service-requirements.md` — 更新 FR-S06-02 的摘要元数据验收
- [x] 产出 `deltas/prd/2-product-design/1-feature-specs/core-S06-mcp-service-design.md` — 定义 queryAll 与 list_diagrams 的真实 revision/updated_at 规则
- [x] 产出 `deltas/api/mcp-tools.yaml` — 将 DiagramSummary 的 revision/updated_at 键列为必需并保持值可空兼容
- [x] 产出 `deltas/test/core-S04-test-cases.md` — 新增 UT-S04-20，验证 queryAll 返回真实元数据且仍过滤软删除
- [x] 产出 `deltas/test/core-S06-test-cases.md` — 新增 UT-MCP-40，验证 MCP 摘要保留真实元数据

## [code] 代码实现

- [x] 修改 `backend/src/diagrams/mod.rs` 与 `backend/src/entity/vo/diagram_vo.rs`，读取并序列化真实 revision/updated_at
- [x] 交付 UT-S04-20、UT-MCP-40 与 OpenLogos reporter，保持 lastModified 回退及列表过滤排序兼容
- [x] 运行后端与 MCP 定向测试和全量测试，确认软删除过滤与 MCP 读链路无回归
