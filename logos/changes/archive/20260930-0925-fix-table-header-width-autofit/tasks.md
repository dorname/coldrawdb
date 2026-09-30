# 实现任务

## [delta] 规格变更
- [x] 产出 delta 文件到 `deltas/prd/2-product-design/1-feature-specs/core-01-editor-canvas.md` — §5.12 R-LOD 规则表新增 R-LOD-11：表头行宽度需求不受 Tier 上限钳制（绝对上限 720px）
- [x] 产出 delta 文件到 `deltas/test/core-CR-canvas-test-cases.md` — 新增 UT-CR-HEADER-WIDTH-01 / ST-CR-HEADER-WIDTH-01

## [code] 代码实现
- [x] `frontend-rs/src/editor_render.rs`：新增 `TABLE_WIDTH_MAX_HEADER = 720` 常量；`estimate_content_width_for` 拆分表头行需求与字段行需求（新增 `estimate_content_width_parts_for`）；`resolve_table_width_for` auto 宽改为 `max(clamp(字段行需求), min(表头行需求, 720))`
- [x] `frontend-rs/src/editor_render.rs`：新增 UT-CR-HEADER-WIDTH-01 单元测试（表头需求超 480 时 Detail 宽度突破上限、短表头仍回 480 内、超 720 钳制、显式 width 不受影响）
- [x] `frontend-rs/scripts/test-spec-parity-d.mjs`：新增 ST-CR-HEADER-WIDTH-01 e2e（长中英文注释表头完整显示不截断）
- [x] `frontend-rs/tests/openlogos_reporter.rs`：注册 UT-CR-HEADER-WIDTH-01 / ST-CR-HEADER-WIDTH-01

## [勘误] R-LOD-11 联动修订（实现验证期发现，同提案范围内）
- [x] UT-CR-HEADER-WIDTH-01 断言 3 勘误：zoom=1.0 时表维度 effective 字号（表名 11/注释 10）低于 Detail 表名 13，Topology 估算值可小于 Detail——删去「步骤 2 ≥ 步骤 1」跨维度大小比较，改为「Topology 表头需求 ∈ (640, 720] 时突破 640 + 同表 Detail 超 720 钳上限」（主文档与 Rust UT 已同步）
- [x] UT-CR-TOPO-WIDTH-01 联动修订：R-LOD-11 后 Tier 上限仅约束字段行需求——测试主体改由长字段行驱动（长字段名+长类型），验证 Topology（640）> Detail（480）钳制结构；表头需求驱动场景归 UT-CR-HEADER-WIDTH-01（主文档与 Rust UT 已同步）
- [x] ST-CR-TOPO-WIDTH-01 e2e 联动修订：同上改长字段行驱动（asset_object_registry + 长字段行），保证「表维度卡宽 > 字段维度」断言在 R-LOD-11 后仍可观测
