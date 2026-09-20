# 实现任务

## [delta] 规格变更

- [x] 产出 delta 到 `deltas/prd/2-product-design/1-feature-specs/core-01b-relationship.md` — §4.5 一键整理布局；修正「V1.2 不做」
- [x] 产出 delta 到 `deltas/prd/2-product-design/1-feature-specs/core-0a-code-editor.md` — 命令面板 Action 入口
- [x] 产出 delta 到 `deltas/test/core-PB-relationship-test-cases.md` — UT-PB-16 / ST-PB-09
- [x] 产出 delta 到 `deltas/test/core-CR-canvas-test-cases.md` — UT-CR-LAYOUT-01（登记表行）

## [code] 代码实现

- [x] `frontend-rs/src/layout.rs`：力导向纯函数（对齐 MCP）+ UT
- [x] `command_palette.rs` + `editor_panels.rs`：Action「整理布局」+ 导入后自动整理
- [x] OpenLogos reporter 登记新用例 ID
