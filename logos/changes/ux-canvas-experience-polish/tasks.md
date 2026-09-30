# 实现任务

## [delta] 规格变更
- [x] 更新 `core-01-editor-canvas.md` §3.3 滚轮缩放因子说明
- [x] 更新 `core-01-editor-canvas.md` §5 关系线视觉层级说明

## [code] 代码实现
- [x] 提取 `CANVAS_WHEEL_ZOOM_FACTOR = 1.15` 并替换 `editor_render.rs` 中硬编码 `1.1`
- [x] 调整 `draw_relation` 中关系线主线/光晕宽度
- [x] 新增 `UT-CR-ZOOM-02` 单元测试；更新 `UT-PE-HL-01` 源码断言
- [x] 在 `frontend-rs/tests/openlogos_reporter.rs` 注册 `UT-CR-ZOOM-02`

## [verify] 验证
- [ ] 运行 `cargo test` 通过
- [ ] 运行 `openlogos verify` 通过
