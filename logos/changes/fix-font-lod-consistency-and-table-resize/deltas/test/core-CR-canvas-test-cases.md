# Delta — core-CR-canvas-test-cases.md（fix-font-lod-consistency-and-table-resize）

## ADDED — UT 用例（追加）

| ID | 输入/操作 | 断言 |
|---|---|---|
| UT-CR-FONT-02 | `compute_table_render_size_for` / `estimate_content_width_for` 在 effective 字号变化时的自适应（#48 §5.10/5.16） | zoom=1.0、label_scale=1.0 时基线宽/高与现状一致；zoom=0.2 触发 R-FONT-04 下限钳制后，auto 宽度 ≥ 基线宽、字段行高 ≥ `FIELD_ROW_HEIGHT`、表头高 ≥ `TABLE_HEADER_HEIGHT`；手动 `table.width=300` 时宽度固定为 300，但行高仍随字号放大；拓扑档（tier=Topology）下表头高随 `lod_table_font_size` 补偿放大 |

## ADDED — ST 用例（追加）

| ID | 前置与步骤 | 预期 |
|---|---|---|
| ST-CR-FONT-02 | e2e：预置两表（一长表名、一长字段名）→ 默认 zoom 截图/探针基线 → 缩放到 zoom=0.2 → 断言所有表卡（含初始离屏后进入视口的表）effective 字号一致、无斑块式大小差异；断言 auto 宽表框随字号加宽、字段行垂直居中无截断；切 `canvas-font-scale` 到 1.5 后表框再次加宽、高亮 | R-WIDTH-06/07/08、R-FONT-07、R-LOD-09 全覆盖；离屏表与可见表渲染一致 |

## MODIFIED — 用例登记（OpenLogos verify 解析用，追加行）

| ID | 层级 | 说明 |
|---|---|---|
| UT-CR-FONT-02 | UT | effective 字号下表宽/行高自适应（#48） |
| ST-CR-FONT-02 | ST | 缩小全景后字体/表尺寸一致（#48） |
