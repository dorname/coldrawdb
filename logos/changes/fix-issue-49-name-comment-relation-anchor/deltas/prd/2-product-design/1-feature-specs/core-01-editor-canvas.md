# Delta — core-01-editor-canvas.md（fix-issue-49-name-comment-relation-anchor）

## MODIFIED — §5.10 R-WIDTH-08

| ID | 约束 |
|---|---|
| R-WIDTH-08 | **命中/锚点/精灵缓存与绘制同口径**：`resolve_table_width`、`field_anchor_for_side`、`anchor_for_tier`、`compute_table_render_size`、`table_sprite_fingerprint` 必须使用与 `draw_table_body` / `draw_table_topology_body` 同一组 **effective 字号参数（`zoom`、`label_font_scale`、`tier`）以及同一 `comment_mode`**，禁止绘制宽、命中宽、关系锚点、精灵尺寸四者分叉。特别地，`field_anchor_for_side`、`anchor_for_tier`、`pick_port_sides` 在 `Name` / `NameComment` / `Comment` 任一模式下均须按该模式解析的有效宽取缘，不得写死 `NameComment`（fix-issue-49）。 |

## MODIFIED — §5.12 R-LOD-08

| ID | 约束 |
|---|---|
| R-LOD-08 | 表维度关系线**表级锚定**（#35）：字段行隐藏时，关系线两端锚点收敛到表级端口——纵坐标 = 表头纵向中线（`table.y + TABLE_HEADER_HEIGHT / 2`），横坐标 = 卡体左/右缘（按当前 `comment_mode` 下的有效宽计算），选侧沿用 §3.4 `pick_port_sides` 并传入当前 `comment_mode`（bezier / orthogonal / straight 三线型同口径）；同一表多条关系线在表级端口汇聚，不再按隐藏字段的纵向位置扇形散开；切回字段维度恢复字段维度锚定。crow's foot 端点记号随表级锚点绘制（fix-issue-49）。 |
