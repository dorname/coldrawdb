# 实现任务

## [delta] 规格变更

- [x] 产出 `deltas/prd/2-product-design/1-feature-specs/core-04-side-panel-tabs.md` — 更新搜索过滤与 master-detail 联动、命中提示规则
- [x] 产出 `deltas/prd/2-product-design/2-page-design/core-01-editor-prototype.html` — 同步唯一主原型的搜索和右侧空态
- [x] 产出 `deltas/test/core-SP-side-panel-test-cases.md` — 修改 ST-SP-LIST-03 并新增 ST-SP-LIST-04
- [x] 产出 `deltas/test/core-UI-modals-2-test-cases.md` — 修改 UT-MM-38 并新增 UT-MM-39

## [code] 代码实现

- [x] 修改 `frontend-rs/src/editor_panels.rs`，让当前表跟随可见结果、无命中显示网格空态并展示命中原因
- [x] 更新 `frontend-rs/scripts/test-spec-parity-d.mjs`，交付 ST-SP-LIST-03/04 与 OpenLogos reporter
- [x] 交付 UT-MM-38/39，运行前端定向测试、相关 Playwright 与全量测试
