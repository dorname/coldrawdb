# 实现任务

## [delta] 规格变更
- [x] 产出 delta 文件到 `deltas/prd/2-product-design/1-feature-specs/core-04-side-panel-tabs.md` — 重写 §10.5：左侧表树 + 右侧单表字段网格交互定义
- [x] 产出 delta 文件到 `deltas/prd/2-product-design/2-page-design/core-01-editor-prototype.html` — renderListView 重构为树+单表布局（样式 + 交互 + testid 锚点）
- [x] 产出 delta 文件到 `deltas/test/core-SP-side-panel-test-cases.md` — ST-SP-LIST-01/02 MODIFIED + 新增 ST-SP-LIST-03（树搜索/切表/空态）
- [x] 产出 delta 文件到 `deltas/test/core-UI-modals-2-test-cases.md` — ST-LV-01 / UT-MM-22 MODIFIED + 新增表树相关 UT

## [code] 代码实现
- [x] 重构 `frontend-rs/src/editor_panels.rs::ListView` 为树+单表网格（新增表树子组件，复用 ListFieldRow/filter_tables；移除 ListGroupHeader 与 Header 行变体）
- [x] 更新 `frontend-rs/src/styles.css` 列表视图布局样式（左右两栏 grid + 树面板样式）
- [x] 编写/更新对应 UT（表树过滤、选中表状态）与 ST 断言对齐代码，含 OpenLogos reporter 写入
