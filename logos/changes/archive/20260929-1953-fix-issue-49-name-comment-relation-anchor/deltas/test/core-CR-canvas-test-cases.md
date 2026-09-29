# Delta — core-CR-canvas-test-cases.md（fix-issue-49-name-comment-relation-anchor）

## ADDED — UT 用例（追加）

| ID | 输入/操作 | 断言 |
|---|---|---|
| UT-CR-ANCHOR-01 | 同一表在三种 `CommentDisplay` 模式（`Name` / `NameComment` / `Comment`）下计算字段维度左右锚点与表维度左右表级锚点 | `field_anchor_for_side_with(..., Left)` 的 x 坐标恒等于 `table.x`；`field_anchor_for_side_with(..., Right)` 的 x 坐标恒等于 `table.x + resolve_table_width_for(table, mode, zoom, label_scale, tier)`；`anchor_for_tier_with(..., Left/Right)` 在表维度下同样满足；三种模式之间，有注释时 `NameComment`/`Comment` 的右缘与 `Name` 不同，但锚点仍精确贴齐当前模式宽 |

## ADDED — ST 用例（追加）

| ID | 前置与步骤 | 预期 |
|---|---|---|
| ST-CR-ANCHOR-01 | e2e：建两表并建关系 → 默认 `name+comment` 下截图/探针基线 → 切注释模式到 `name` → 断言关系线端点仍贴齐当前模式卡边（无可见空隙）→ 切到 `comment` → 同样断言 → 切回 `name+comment` 回归 | R-WIDTH-08 / R-LOD-08 在 `comment_mode` 维度上的同口径覆盖；任意注释模式切换后关系线不悬空 |

## MODIFIED — 用例登记（OpenLogos verify 解析用，追加行）

| ID | 层级 | 说明 |
|---|---|---|
| UT-CR-ANCHOR-01 | UT | 三种注释模式下关系锚点与当前 mode 卡宽对齐（#49） |
| ST-CR-ANCHOR-01 | ST | 注释模式切换后关系线贴边 e2e（#49） |
