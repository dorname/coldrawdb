# 实现任务

## [delta] 规格变更
- [x] 更新 `core-08-icon-library.md` ToolRail 图标清单
- [x] 更新 `core-04-side-panel-tabs.md` §1 ToolRail 按钮清单图标引用

## [code] 代码实现
- [x] 重绘 `frontend-rs/src/icons.rs` 中 ToolRail 图标组件（保持 testid/行为不变）
- [x] 更新 `frontend-rs/src/editor_panels.rs` ToolRail 中图标组件引用（若组件名变化）
- [x] 更新 `UT-PB-27` 源码锚点断言

## [verify] 验证
- [ ] 运行 `cargo test` 通过
- [ ] 运行 `openlogos verify` 通过
