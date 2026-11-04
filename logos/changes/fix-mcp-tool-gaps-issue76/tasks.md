# 实现任务

## [delta] 规格变更

- [ ] 产出 `deltas/api/mcp-tools.yaml` — ① update_table action 扩为 create|update|delete（delete 禁携带其他字段，description 写明顺带清理触碰 references）；② 新增 update_area / update_note 工具（schema 平铺，遵守客户端兼容约束）；③ update_diagram 新增 allow_mass_delete 守护参数与「缺失 section = 删除」语义说明；④ PAYLOAD_TOO_LARGE 建议文案补充增量工具提示
- [ ] 产出 `deltas/test/core-S06-test-cases.md` — ADDED UT-MCP-51～54 与 ST-MCP-22；MODIFIED UT-MCP-50（14 工具锚点 + 新工具平铺断言）；AC 追溯更新

## [code] 代码实现

- [ ] `mcp-server/src/service.rs` — update_table delete 分支（移表 + 清理触碰 references + 禁字段校验 + 零写入错误）；新增 update_area / update_note 实现；update_diagram 守护（写前 GET base 比较缺失表，>10 或 >50% 需 allow_mass_delete=true）
- [ ] `mcp-server/src/payload_limit.rs` — suggestion 文案补充增量工具提示
- [ ] `mcp-server/tests/c6_write_contract.rs` — 交付 UT-MCP-51～54、扩展 UT-MCP-50（14 工具 + 新工具平铺断言），全部接 reporter
- [ ] merge 后重建 `mcp-server/target/release/coldrawdb-mcp` 并以真实 stdio 探针复核 14 工具全部可见（ST-MCP-22 真实入账）
- [ ] mcp-server 全量测试 0 失败后重跑 `openlogos verify` 全量验收
