# 实现任务

## [delta] 规格变更
- [x] 产出 delta 文件到 `deltas/prd/2-product-design/1-feature-specs/core-09-core-components.md` — §14.2 增补分隔条拖动方向语义（MODIFIED）

## [code] 代码实现
- [x] 修复 `frontend-rs/src/editor_render.rs` 网格点阵合批连线 bug（`GridSink::dot` 补 `move_to`）
- [x] 修复 `frontend-rs/src/splitter.rs` Inspector 拖动方向反向（`drag_width` 按实例取符号），同步更新 `mod tests` 断言
- [x] 前端 `cargo test` 通过 + trunk 重建后实机回归（画布无横线、右拖变窄）
