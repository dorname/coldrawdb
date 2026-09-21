# Delta — core-01-editor-canvas.md（修改）

> 模块：core | 提案：fix-table-auto-width-include-comment
> 关联 issue：#25（表宽自适应未计入中文注释长度）

## MODIFIED — ADDED — §5.10 表卡内容自适应宽度（fix-open-issues-19-22 / #20）

> 补齐 `TABLE_WIDTH` 硬编码缺口；与 R-CMT-01/02「单行省略」并存——自适应先算最小可读宽，再在该宽内省略。
> #25：澄清 NameComment 下注释为**并排累加**测量，禁止用各文本段取 max 代替横向之和。

| ID | 约束 |
|---|---|
| R-WIDTH-01 | `table.width == None` 或 `Some(0)` 表示 **auto**：渲染宽按当前注释显示模式下**实际并排绘制**的文本测量，取各行需求最大值，夹在 `[TABLE_WIDTH, TABLE_WIDTH_MAX]`（`TABLE_WIDTH=230`，`TABLE_WIDTH_MAX=480`，对齐 ListView 列宽上限）。**测量口径（#25）**：`name+comment` 且 secondary 非空时，表头行 = 主文本宽 + 间距 + 注释宽 + 字段计数预留 + 内边距；字段行 = PK 徽章水平偏移（若有）+ 主文本宽 + 间距 + 注释宽 + 间距 + 类型宽 + 内边距。禁止用「仅注释宽」与「名+类型宽」取 `max` 代替并排累加。`name` / `comment`（无 secondary）仍按主文本 + 类型（字段行）测量。 |
| R-WIDTH-02 | `table.width` 为正整数时尊重用户手动宽度（Set Table Width / 拖拽改宽），不因内容自动覆盖 |
| R-WIDTH-03 | `compute_table_render_size`、hit-test、关系锚点、精灵缓存指纹必须共用同一有效宽，禁止绘制宽与命中宽分叉 |
| R-WIDTH-04 | 导入或新建表且宽度为 auto 时，允许一次性计算并可选写回 `table.width`（持久化 fit）；之后用户改名为更长文本时，auto 模式应在下次重绘按内容再算（不强制每次写库） |
| R-WIDTH-05 | Set Table Width 文案「0 = auto」语义必须与 R-WIDTH-01 一致（禁止把 `0` 当成 0px 窄表） |

**验收口径**：长表名（如 `asset_v2.virtualization_cluster_profile`）在 auto 下表头完整可辨（或至少显著宽于 230px 且逼近测量宽）；手动设宽后不再被内容强制撑开；**短英文名 + 非空中文注释（`name+comment`）时估算宽严格大于无注释对照（未触达 480 上限前）**。
