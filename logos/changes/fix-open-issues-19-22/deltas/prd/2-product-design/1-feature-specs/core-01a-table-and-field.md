# Delta — core-01a-table-and-field.md（修改）

> 模块：core | 提案：fix-open-issues-19-22
> 关联 issue：#20（表宽自适应）、#22（表颜色选择器）

## ADDED — §1.7 表宽内容自适应（#20）

> 实现 issue #20。渲染细则见 `core-01-editor-canvas.md` §5.10 R-WIDTH-01～05。

| 规则 | 规格 |
|---|---|
| 数据语义 | `Table.width`：`None` / `0` = auto；正整数 = 固定 CSS/世界坐标宽度（px） |
| 测量输入 | 随注释显示模式（`name` / `name+comment` / `comment`）取表头主文本、字段名、类型、注释中实际绘制的字符串 |
| 夹紧 | `[230, 480]`（与 ListView `auto_calc_column_width` 上限一致） |
| Inspector / 模态 | Set Table Width：「0 = auto」写入 auto 语义；正数写入固定宽 |
| 导入 | SQL/DBML/JSON 导入新建表默认 auto，首次展示按内容 fit |

## MODIFIED — §1.6 表颜色配置（#22 扩展）

在既有预设色板合同上追加：

| 规则 | 规格 |
|---|---|
| 自由取色 | `data-testid="inspector-table-color"` 区域在预设快捷色之外提供 native color picker（`<input type="color">`）或等价 hex 输入；可选色空间不限于命名预设 |
| 即时落账 | picker/`change`/`input` 变更走既有 store → dirty → schedule_save，进 UndoRedoContext |
| 预设并存 | 保留命名预设下拉或色板快捷项；「默认」清空回 `''` |
| 出边跟随 | 表色变更后，所有**未显式设色**的出边关系线渲染色应即时跟随（合同见 `core-01b` §4.2）；不静默改写已保存的 `Reference.color` |

## MODIFIED — 5. 测试用例 ID 索引（追加）

| TC ID | 描述 |
|---|---|
| UT-CR-WIDTH-01 | `compute_table_render_size`：`None`/`0` 按内容夹紧；正数 width 原样 |
| UT-CR-WIDTH-02 | 长表名测量宽 > 230 且 ≤ 480 |
| ST-CR-WIDTH-01 | e2e：长表名表卡明显宽于默认 230；Inspector 名称完整可读对照 |
| UT-CR-COLOR-02 | picker/hex 写入任意合法色并参与边框渲染 |
| ST-CR-COLOR-02 | e2e：picker 改色 → 保存刷新保留；出边未设色时跟随表色 |
