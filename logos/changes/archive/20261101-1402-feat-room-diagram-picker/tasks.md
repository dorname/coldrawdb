# 实现任务

## [delta] 图表选择器与个人建模规格
- [x] 产出 `deltas/prd/2-product-design/1-feature-specs/core-S04-room-lifecycle-design.md` — 可搜索选择器、元数据、危险区和个人入口
- [x] 产出 `deltas/prd/3-technical-plan/2-scenario-implementation/core-S04-room-lifecycle.md` — 摘要查询与个人新建/打开时序
- [x] 产出 `deltas/test/core-S04-test-cases.md` — 新增摘要计数、筛选排序与个人/房间真实浏览器用例，修订删除回归

## [code] 图表选择器与个人建模实现
- [x] 扩展 `backend/src/entity/vo/diagram_vo.rs` 与 `backend/src/diagrams/mod.rs` 的 queryAll 聚合摘要，新增 UT-S04-21
- [x] 扩展 `frontend-rs/src/editor_data_access.rs` 摘要模型，在 `frontend-rs/src/editor_panels.rs` 实现搜索/排序/元数据纯函数与 UT-S04-UI-19
- [x] 在 `frontend-rs/src/editor_panels.rs` 与 `frontend-rs/src/styles.css` 实现共享图表选择器、独立危险区及个人建模入口，复用加载守卫与私有保存通路
- [x] 在 `frontend-rs/scripts/test-spec-parity-d.mjs` 实现 ST-S04-UI-18、修订 ST-S04-UI-16，并在 `frontend-rs/tests/openlogos_reporter.rs` 登记新用例
- [ ] 串行执行定向 Rust 与 350+ 图真实浏览器回归；正式验收限制 Cargo、Rust 测试和 Playwright 并发
