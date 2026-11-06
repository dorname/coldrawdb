# 实现任务

## [delta] 规格变更

- [x] 产出 `deltas/test/core-CR-canvas-test-cases.md` — ADDED UT-CR-FOCUS-03（fit_content_transform 纯函数三态）+ UT-CR-FOCUS-04（三加载挂点自动 fit 与 btn-fit-canvas 锚点），附录登记
- [x] 产出 `deltas/test/core-S04-test-cases.md` — ADDED ST-S04-UI-19（进房后表无需平移即可见 e2e）+ ST-S04-UI-20（btn-fit-canvas 点击 fit e2e），变更记录登记

## [code] 代码实现

- [x] `frontend-rs/src/editor_render.rs` — 新增 `content_bounds` / `fit_content_transform` 纯函数（zoom clamp [ZOOM_MIN, 1.0]、空内容 None）+ UT-CR-FOCUS-03 单测
- [x] `frontend-rs/src/editor_panels.rs` — 三个加载挂点（on_enter_room / on_open_personal_diagram / share 冷启动）store.load 成功后双 rAF 自动 fit；FloatingControls 新增 btn-fit-canvas（复用 fit 纯函数）+ UT-CR-FOCUS-04 锚点（实现另覆盖 invite accept 进房第四挂点，同根因面）
- [x] `frontend-rs/scripts/test-spec-parity-d.mjs` — ST-S04-UI-19/20 e2e（mock 表坐标远离原点，断言进房/点击按钮后表框在视口内可见）
