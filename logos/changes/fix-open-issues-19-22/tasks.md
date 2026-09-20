# 实现任务

## [delta] 规格变更

- [x] 产出 delta 文件到 `deltas/prd/2-product-design/1-feature-specs/core-01-editor-canvas.md` — #19 拖动后 click 抑制；#20 表宽自适应渲染合同
- [x] 产出 delta 文件到 `deltas/prd/2-product-design/1-feature-specs/core-01a-table-and-field.md` — #20 内容自适应宽度；#22 颜色选择器（预设 + picker / hex）
- [x] 产出 delta 文件到 `deltas/prd/2-product-design/1-feature-specs/core-01b-relationship.md` — #21 线条类型/线型 + 密度降噪；#22 空 color 继承源表色
- [x] 产出 delta 文件到 `deltas/prd/2-product-design/2-page-design/core-01-editor-prototype.html` — #21/#22 原型微对齐
- [x] 产出 delta 文件到 `deltas/api/diagrams.yaml` — #21 Reference `line_type`/`stroke_style`；#22 空 color 语义澄清
- [x] **验证 API YAML** — delta 描述为 OpenAPI 增量指引（merge 时展开为合法 YAML；enum/description 已双引号约定）
- [x] 产出 delta 文件到 `deltas/database/coldrawdb-v1.sql` — #21 reference 新增 `line_type`/`stroke_style`
- [x] 产出 delta 文件到 `deltas/test/core-CR-canvas-test-cases.md` — #19/#20/#22 相关 UT/ST
- [x] 产出 delta 文件到 `deltas/test/core-PB-relationship-test-cases.md` — #21/#22 关系线型与继承色用例

## [code] 代码实现

批次 A — #19 拖动 click 穿透：
- [ ] `frontend-rs/src/editor_render.rs`（+ 必要时 `editor_panels.rs` / styles）：有效拖动后抑制邀请 click；调整 pointer capture 释放时机
- [ ] 批次 A 测试代码 + OpenLogos reporter（含 e2e：拖到 btn-invite 不打开 modal-invite）

批次 B — #20 表宽内容自适应：
- [ ] `frontend-rs/src/editor_render.rs`：`compute_table_render_size` / measure 自适应；`0`/`None` = auto
- [ ] 导入/建表 fit 路径（如适用）+ Set Table Width「0 = auto」语义对齐
- [ ] 批次 B 测试代码 + OpenLogos reporter

批次 C — #21 线型 + 密度降噪：
- [ ] 前端模型 + 后端 entity + migration：`line_type` / `stroke_style`
- [ ] `editor_render.rs`：正交折线 / 直线 / 虚线渲染；选中态非相关线淡化
- [ ] `editor_panels.rs`：关系 Inspector/浮层切换入口
- [ ] 批次 C 测试代码 + OpenLogos reporter

批次 D — #22 表色 picker + 出边跟随：
- [ ] `editor_panels.rs`：表色预设 + color picker / hex
- [ ] `editor_render.rs`：`relation_stroke_color` 空色继承源表色
- [ ] 批次 D 测试代码 + OpenLogos reporter
