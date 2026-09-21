# Delta — core-01a-table-and-field.md（修改）

> 模块：core | 提案：fix-table-auto-width-include-comment
> 关联 issue：#25

## MODIFIED — ADDED — §1.7 表宽内容自适应（fix-open-issues-19-22 / #20）

> 实现 issue #20；#25 补齐 NameComment 并排注释计入宽度。渲染细则见 `core-01-editor-canvas.md` §5.10 R-WIDTH-01～05。

| 规则 | 规格 |
|---|---|
| 数据语义 | `Table.width`：`None` / `0` = auto；正整数 = 固定 CSS/世界坐标宽度（px） |
| 测量输入 | 随注释显示模式（`name` / `name+comment` / `comment`）取表头主文本、字段名、类型、注释中**实际并排绘制**的字符串；`name+comment` 下注释与名称/类型横向累加（见 R-WIDTH-01），不得仅取 max |
| 夹紧 | `[230, 480]`（与 ListView `auto_calc_column_width` 上限一致） |
| Inspector / 模态 | Set Table Width：「0 = auto」写入 auto 语义；正数写入固定宽 |
| 导入 | SQL/DBML/JSON 导入新建表默认 auto，首次展示按内容 fit |

## MODIFIED — 5. 测试用例 ID 索引

> 在原表末尾追加（既有条目保留）：

| TC ID | 描述 |
|---|---|
| UT-CR-COMMENT-01 | 注释渲染口径：三态显示模式 + 空 comment 不留占位（画布绘制参数纯函数） |
| UT-CR-COMMENT-02 | 显示开关读写 `localStorage["cdb.comment-display"]`，默认 `name+comment` |
| UT-CR-COLOR-01 | 表边框用色：`table.color` 非空跟随、为空回退 palette；关系线 `ref.color` 同理 |
| ST-CR-COMMENT-01 | e2e：导入含中文 COMMENT 的 SQL → 画布表头/字段行可见注释 → 刷新仍在 |
| ST-CR-COLOR-01 | e2e：Inspector 改表颜色 → 表头/边框即时更新 → 保存刷新后保留；导出 JSON 再导入颜色仍在 |
| UT-CR-WIDTH-01 | `compute_table_render_size`：`None`/`0` 按内容夹紧；正数 width 原样 |
| UT-CR-WIDTH-02 | 长表名测量宽 > 230 且 ≤ 480 |
| ST-CR-WIDTH-01 | e2e：长表名表卡明显宽于默认 230；Inspector 名称完整可读对照 |
| UT-CR-COLOR-02 | picker/hex 写入任意合法色并参与边框渲染 |
| ST-CR-COLOR-02 | e2e：picker 改色 → 保存刷新保留；出边未设色时跟随表色 |
| UT-CR-COLOR-03 | 表头前景相对有效背景亮度自适应（浅色表头 → 深色字；深色表头 → 浅色字） |
| UT-CR-WIDTH-03 | NameComment 下短英文名 + 中文注释并排撑宽（#25）：有注释估算 > 无注释；近似 ≥ 名+注释+类型 |

> 详细步骤见 `core-CR-canvas-test-cases.md`；Inspector 注释编辑既有用例（UT-PC-29 / UT-PC-30 / ST-PC-08）继续有效。
