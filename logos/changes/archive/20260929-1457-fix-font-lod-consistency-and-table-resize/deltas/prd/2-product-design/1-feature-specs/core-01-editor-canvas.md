# Delta — core-01-editor-canvas.md（fix-font-lod-consistency-and-table-resize）

## ADDED — §5.10 表卡内容自适应宽度（续：R-WIDTH-06~08）

| ID | 约束 |
|---|---|
| R-WIDTH-06 | **effective 字号变化时 auto 宽度必须重测**：`estimate_content_width` 需感知 `zoom`、`label_font_scale`、`tier`；当 R-FONT-04 / R-LOD-03 导致实际绘制字号变大时，字符宽度按 effective 字号比例放大，auto 宽度应同步增加（仍夹在 `[TABLE_WIDTH, TABLE_WIDTH_MAX]`）。禁止在字号被钳制/补偿后仍使用 zoom=1.0/label_scale=1.0 的静态宽度，导致文本截断。 |
| R-WIDTH-07 | **表头高与字段行高随 effective 字号自适应**：字段维度下字段行高 = `max(FIELD_ROW_HEIGHT, effective_field_px + 2 × 行内留白)`，表头高 = `max(TABLE_HEADER_HEIGHT, effective_name_px + 2 × 表头留白)`；拓扑档表头高同 R-LOD-09。保证字号放大后行内文本垂直居中、不贴边、不被底部截断。 |
| R-WIDTH-08 | **命中/锚点/精灵缓存与绘制同口径**：`resolve_table_width`、`field_anchor_for_side`、`anchor_for_tier`、`compute_table_render_size`、`table_sprite_fingerprint` 必须使用与 `draw_table_body` / `draw_table_topology_body` 同一组 effective 字号参数（`zoom`、`label_font_scale`、`tier`），禁止绘制宽、命中宽、关系锚点、精灵尺寸四者分叉。 |

**验收口径（补充）**：低 zoom 下（如 zoom=0.2）表名/字段名因 10px 下限被放大后，auto 宽表显著宽于 230px；字段行垂直居中；关系线仍准确连接卡体左右缘；切换 zoom 后离屏表与可见表字体/尺寸一致。

## MODIFIED — §5.12 小缩放 LOD 渲染分档（新增 R-LOD-09）

| ID | 约束 |
|---|---|
| R-LOD-09 | **表维度表头高随表名字号补偿自适应**：拓扑档表头高 = `max(TABLE_HEADER_HEIGHT, scaled_label_font_world(lod_table_font_size(zoom, tier), zoom, label_scale).0 + 2 × 留白)`，与 R-WIDTH-07 同源，确保表维度下放大表名后不被截断。 |

## MODIFIED — §5.16 画布标签字号（新增 R-FONT-07）

| ID | 约束 |
|---|---|
| R-FONT-07 | **字号变化触发尺寸重算与重光栅**：`label_font_scale` 切换或 zoom 变化导致 effective 字号跨越钳制边界时，必须使 `table_sprite_fingerprint` 失效并重新估算表卡尺寸；禁止复用旧精灵缓存或旧 width/height 导致「字体已变、表框未变」的视觉错位。 |
