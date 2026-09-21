# 实现任务

## [delta] 规格变更
- [x] 产出 delta 文件到 `deltas/prd/2-product-design/1-feature-specs/` — 更新 `core-01-editor-canvas.md`：§5.8 增补 R-COLOR-04（表头前景相对背景自适应），修订 R-CMT-01 表头取色口径
- [x] 产出 delta 文件到 `deltas/prd/2-product-design/1-feature-specs/` — 更新 `core-01a-table-and-field.md`：§1.6 表色渲染交叉引用 R-COLOR-04（如章节已有颜色合同则 MODIFIED）
- [x] 产出 delta 文件到 `deltas/test/` — 更新 `core-CR-canvas-test-cases.md`：新增 UT-CR-COLOR-03（表头对比度自适应）并登记索引表

## [code] 代码实现
- [x] 在 `frontend-rs/src/editor_render.rs` 实现表头有效背景亮度解析 + 前景色选择纯函数，并在 `draw_table_body` 表头文字路径消费
- [x] 新增/扩展 UT（`canvas_comment_color_ut.rs` 或同级）：覆盖 UT-CR-COLOR-03；接入 OpenLogos reporter
- [x] 将 `UT-CR-COLOR-03` 登记到 `frontend-rs/tests/openlogos_reporter.rs`（如该清单为权威枚举）
