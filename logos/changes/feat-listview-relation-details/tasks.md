# 实现任务

## [delta] 列表视图关系明细与搜索规格
- [x] 产出 `deltas/prd/2-product-design/1-feature-specs/core-04-side-panel-tabs.md` — 明确树节点方向标记、逐条关系明细、关系搜索及测试索引
- [x] 产出 `deltas/test/core-SP-side-panel-test-cases.md` — 修订既有关系用例并新增方向/明细/搜索的 UT/ST 覆盖

## [code] 列表视图关系浏览实现
- [x] 在 `frontend-rs/src/editor_core.rs` 与 `frontend-rs/src/editor_panels.rs` 中复用并扩展统一关系显示模型，实现表方向、方向化端点基数、ON UPDATE 和关系搜索
- [x] 在 `frontend-rs/src/editor_panels.rs` 与 `frontend-rs/src/styles.css` 中实现树节点方向标记和逐条关系明细，保留对端表导航与只读行为
- [x] 在 `frontend-rs/src/editor_panels.rs` 中实现 UT-SP-LIST-REL-01/02，在 `frontend-rs/scripts/test-spec-parity-d.mjs` 中实现 ST-SP-LIST-REL-01/02，并在 `frontend-rs/tests/openlogos_reporter.rs` 登记新用例
- [ ] 使用 16 表 / 19 关系模型执行定向 Rust 与真实浏览器回归；正式验收时限制构建、测试和 Playwright 并发
