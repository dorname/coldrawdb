# Delta — core-01a-table-and-field.md（修改）

> 模块：core | 提案：fix-table-header-contrast
> 关联 issue：#24

## MODIFIED — ADDED — §1.6 表颜色配置（fix-remote-github-issues-7-18）

> 实现 issue #12 的表侧。`Table.color` 字段已存在且表头渐变已使用；本提案补齐**表边框着色**与 **Inspector 编辑入口**。
> issue #24：表头文字对比度见 `core-01-editor-canvas.md` §5.8 R-COLOR-04。

| 规则 | 规格 |
|---|---|
| Inspector 入口 | 「数据表」section 注释输入下方追加「颜色」选择器（`data-testid="inspector-table-color"`）：预设色板（与主原型强调色选项对齐）+ 「默认」项（清空回 `''`）；变更走既有落账链路（blur/change → store → `dirty` + `schedule_save`），进 UndoRedoContext |
| 自由取色（#22） | 在预设快捷色之外提供 native color picker（`<input type="color" data-testid="inspector-table-color-picker">`）或等价 hex 输入；可选色空间不限于命名预设 |
| 即时落账 | picker/`change`/`input` 变更走既有 store → dirty → schedule_save，进 UndoRedoContext |
| 出边跟随 | 表色变更后，所有**未显式设色**的出边关系线渲染色应即时跟随（合同见 `core-01b` §4.2）；不静默改写已保存的 `Reference.color` |
| 渲染 | `color` 非空：表头渐变（现状）+ **表边框**跟随该色；为空：保持 `palette` 默认（`header_tint` / `table_border`） |
| 表头文字对比度（#24） | 表头主文本 / 旁注 / 字段计数必须按 `core-01-editor-canvas.md` §5.8 **R-COLOR-04** 相对表头有效背景自适应深/浅前景；禁止仅按全局主题固定浅色字绘制在浅色表头上 |
| 持久化 | `color` 已随 diagram JSON 保存（`diagrams.yaml` `Table.color` 既有）；无需迁移 |
| 导入导出 | JSON 保留；SQL/DBML 降级忽略（现状不变） |
| 兼容 | 存量图 `color=''` → 视觉与现状一致（默认 tint + R-COLOR-04 合成后仍可读） |

**关系线颜色**入口与合同见 `core-01b-relationship.md` §4.1 / §4.2。

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

> 详细步骤见 `core-CR-canvas-test-cases.md`；Inspector 注释编辑既有用例（UT-PC-29 / UT-PC-30 / ST-PC-08）继续有效。
