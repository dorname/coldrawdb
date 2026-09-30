# 实现任务

## [delta] 规格变更
- [x] 产出 delta 文件到 `deltas/prd/2-product-design/1-feature-specs/core-01b-relationship.md` — §3.1 拆分为“创建关系”与“选中关系”两种工具模式
- [x] 产出 delta 文件到 `deltas/prd/2-product-design/1-feature-specs/core-04-side-panel-tabs.md` — §1 Tool Rail 按钮清单新增 `tool-relationship-create` / `tool-relationship-select`
- [x] 产出 delta 文件到 `deltas/prd/2-product-design/1-feature-specs/core-08-icon-library.md` — 新增 `IconRelationshipCreate` / `IconRelationshipSelect` 图标条目
- [x] 产出 delta 文件到 `deltas/test/core-PB-relationship-test-cases.md` — 新增 UT-PB-27 / ST-PB-16

## [code] 代码实现
- [ ] `frontend-rs/src/editor_panels.rs`：`ActiveTool` 枚举新增 `RelationshipSelect`
- [ ] `frontend-rs/src/editor_panels.rs`：`ToolRail` 新增两个关系按钮（拆分原有 `tool-relationship` 按钮行为）
- [ ] `frontend-rs/src/editor_panels.rs`：`RoomEditor` 事件分发：在 `RelationshipSelect` 模式下点击关系线命中时写入 `SelectionKind::Reference`，不进入 `RelToolState`
- [ ] `frontend-rs/src/icons.rs`：新增 `IconRelationshipCreate` / `IconRelationshipSelect` 图标（V1 先用样式区分；SVG 可占位）
- [ ] `frontend-rs/src/editor_panels.rs`：新增 UT-PB-27 单元测试（ActiveTool 切换与 RelationshipSelect 选择态）
- [ ] `frontend-rs/tests/openlogos_reporter.rs`：注册 UT-PB-27 / ST-PB-16
