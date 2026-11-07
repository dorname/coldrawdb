# 实现任务

## [delta] 规格变更

- [x] 产出 `deltas/test/core-CR-canvas-test-cases.md` — ADDED UT-CR-FOCUS-03（fit_content_transform 纯函数三态）+ UT-CR-FOCUS-04（三加载挂点自动 fit 与 btn-fit-canvas 锚点），附录登记
- [x] 产出 `deltas/test/core-S04-test-cases.md` — ADDED ST-S04-UI-19（进房后表无需平移即可见 e2e）+ ST-S04-UI-20（btn-fit-canvas 点击 fit e2e），变更记录登记

## [code] 代码实现

- [x] `frontend-rs/src/editor_render.rs` — 新增 `content_bounds` / `fit_content_transform` 纯函数（zoom clamp [ZOOM_MIN, 1.0]、空内容 None）+ UT-CR-FOCUS-03 单测
- [x] `frontend-rs/src/editor_panels.rs` — 三个加载挂点（on_enter_room / on_open_personal_diagram / share 冷启动）store.load 成功后双 rAF 自动 fit；FloatingControls 新增 btn-fit-canvas（复用 fit 纯函数）+ UT-CR-FOCUS-04 锚点（实现另覆盖 invite accept 进房第四挂点，同根因面）
- [x] `frontend-rs/scripts/test-spec-parity-d.mjs` — ST-S04-UI-19/20 e2e（mock 表坐标远离原点；最终口径：`__cdb_fit_probe`/`__cdb_transform` 常驻探针数学断言 + fillText hook 记录 CTM 变换后屏幕坐标的视觉断言，`paintViaWheel` 双 wheel 补帧越过 headless 按需帧语义；UI-19 含空图冷启动回归）

## [code] verify 迭代（2026-11-07，方案 A 收窄）

- [x] `frontend-rs/src/editor_render.rs` — 新增 `content_offscreen` 纯函数（screen AABB 与视口无交集判定）+ UT-CR-FOCUS-05 五态单测
- [x] `frontend-rs/src/editor_panels.rs` — `fit_to_content` 参数化 `only_if_offscreen`（挂点 true / 按钮 false）+ fit 探针全路径写入（空内容/收窄拒绝同登记）+ UT-CR-FOCUS-04 双路径接线锚点
- [x] `frontend-rs/tests/openlogos_reporter.rs` — 白名单补登记 UT-CR-FOCUS-03/04/05（首轮 verify 未覆盖根因）
- [x] 回归：lib UT 375 全过；ST-S04-UI-19/20 收窄版 e2e 双 PASS
