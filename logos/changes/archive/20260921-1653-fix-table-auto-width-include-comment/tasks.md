# 实现任务

## [delta] 规格变更

- [x] 产出 delta 文件到 `deltas/prd/2-product-design/1-feature-specs/core-01-editor-canvas.md` — 澄清 §5.10 R-WIDTH-01 并排累加测量口径（#25）
- [x] 产出 delta 文件到 `deltas/prd/2-product-design/1-feature-specs/core-01a-table-and-field.md` — §1.7 交叉引用测量口径
- [x] 产出 delta 文件到 `deltas/test/core-CR-canvas-test-cases.md` — 新增 UT-CR-WIDTH-03（中文注释并排撑宽）

## [code] 代码实现

- [x] 修正 `frontend-rs/src/editor_render.rs`：`estimate_content_width` 在 NameComment 下对表头/字段注释横向累加（含 PK 偏移）
- [x] 新增 UT-CR-WIDTH-03（`frontend-rs/tests/fix_issues_19_22_ut.rs` 或同目录）+ OpenLogos reporter 登记
- [x] 确认 UT-CR-WIDTH-01 / UT-CR-WIDTH-02 不回归
