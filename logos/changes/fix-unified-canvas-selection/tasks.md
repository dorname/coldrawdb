# 实现任务

## [delta] 规格变更

- [x] 产出 `deltas/prd/2-product-design/1-feature-specs/core-01-editor-canvas.md` — 新增统一选择、拖动与空白平移合同
- [x] 产出 `deltas/prd/2-product-design/1-feature-specs/core-01b-relationship.md` — 修订选中关系工具的回落与禁止创建语义
- [x] 产出 `deltas/prd/2-product-design/1-feature-specs/core-04-side-panel-tabs.md` — 默认工具命名为「选择 / 平移」
- [x] 产出 `deltas/test/core-PB-relationship-test-cases.md` — 新增 UT-PB-36 与 ST-PB-20，并关联既有回归

## [code] 代码实现

- [x] 修改 `frontend-rs/src/editor_render.rs`，统一选择类工具的指针路由并交付 UT-PB-36
- [x] 修改 `frontend-rs/src/editor_panels.rs`，更新 ToolRail 名称、可访问名称与模式锚点
- [x] 更新 `frontend-rs/scripts/test-spec-parity-d.mjs`，交付 ST-PB-20 真实浏览器交互与 OpenLogos reporter
- [x] 更新 `frontend-rs/tests/openlogos_reporter.rs`，登记 UT-PB-36 并运行定向及限资源回归
