# 实现任务

## [delta] 规格变更
- [x] 产出 delta 文件到 `deltas/prd/2-product-design/1-feature-specs/core-04-side-panel-tabs.md` — 更新 §10.5 列表视图为按 Area 分组 + 关联表摘要
- [x] 产出 delta 文件到 `deltas/test/core-SP-side-panel-test-cases.md` — 新增 UT-SP-LIST-GROUP-01 / ST-SP-LIST-GROUP-01 / UT-SP-LIST-REL-01 / ST-SP-LIST-REL-01

## [code] 代码实现
- [ ] `frontend-rs/src/editor_panels.rs`：新增按 Area 分组纯函数与关联表集合纯函数
- [ ] `frontend-rs/src/editor_panels.rs`：重写 ListView 左侧表树为分组渲染（折叠/搜索/空态）
- [ ] `frontend-rs/src/editor_panels.rs`：在右侧字段网格上方新增「关联表」摘要区，并在树中高亮关联节点
- [ ] `frontend-rs/src/editor_panels.rs`：新增对应 UT 测试
- [ ] `frontend-rs/scripts/test-spec-parity-d.mjs`：新增 ST-SP-LIST-GROUP-01 / ST-SP-LIST-REL-01 e2e 测试
- [ ] `frontend-rs/tests/openlogos_reporter.rs`：注册新增 UT/ST ID
