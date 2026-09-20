# Delta — core-01-editor-canvas.md（修改）

> 模块：core | 提案：fix-table-header-contrast
> 关联 issue：#24（表头/浅色背景与浅色字体对比度不足）

## MODIFIED — ADDED — §5.8 表/字段注释与颜色渲染（V1.2，fix-remote-github-issues-7-18）

> 实现 issue #10 的画布渲染侧与 issue #12 的表/关系渲染侧；交互入口与显示开关见 `core-01a-table-and-field.md` §1.5/§1.6，关系侧见 `core-01b-relationship.md` §4.1。
> 对比度自适应（issue #24）见 R-COLOR-04。

| ID | 约束 |
|---|---|
| R-CMT-01 | `Table.comment` 非空且显示模式含注释时，表头在表名下方（或同行右侧）渲染注释文本，字号小于表名；**表头**主文本与旁注/字段计数颜色取自 R-COLOR-04 选定的强/次前景对（不得在浅色表头上固定使用暗色主题浅色字）；超出单行省略，hover tooltip（title）显示完整 comment |
| R-CMT-02 | `Field.comment` 非空且显示模式含注释时，字段行在类型文本旁/后渲染注释（灰色、单行省略 + title 全文）；字段行宽度内按 名称 > 类型 > 注释 的优先级截断。字段行绘于表体底色之上，取色仍跟当前主题 `palette`（本变更不改字段行） |
| R-CMT-03 | comment 为空时不渲染、不留占位，行高/表高与现状一致；注释参与 R-PERF-07 精灵指纹（内容变化触发重光栅） |
| R-CMT-04 | 显示模式三态：`name`（仅英文名）/ `name+comment`（默认，用户已拍板）/ `comment`（仅注释）；模式为画布级视图设置，切换即重绘，不落库到 diagram 数据 |
| R-COLOR-01 | 表边框色：`Table.color` 非空时表边框/选中外环基色跟随 `table.color`（现状仅表头渐变使用）；为空保持 `palette.table_border` |
| R-COLOR-02 | 关系线按 `Reference.color` 渲染；空 `Reference.color` 时的解析合同见 `core-01b-relationship.md` §4.1 / §4.2（可继承源表色） |
| R-COLOR-03 | 自定义色在暗/亮主题下与选中高亮并存时可辨识；选中态外环仍用 `palette.selected` |
| R-COLOR-04 | **表头前景自适应**：以表头有效背景（`table.color` 非空则用该色，否则用 `palette.header_tint`）相对 `palette.table_bg` 做 alpha 合成后计算相对亮度；亮度 ≥ 阈值（实现默认 0.55）时表头文字使用深色强/次文本（对齐 `PALETTE_LIGHT.text_strong` / `text_muted`），否则使用浅色强/次文本（对齐 `PALETTE_DARK` 对应字段）。覆盖：暗主题 + 浅表色、亮主题 + 浅/深表色、自定义色接近主题字色。默认半透明主题 tint 合成后必须仍可读。 |

**验收口径**：有 comment 的表/字段在默认模式下画布可见中文注释；空 comment 无占位噪音；导入导出（JSON）后 comment 与颜色保留，画布刷新仍可见；**浅色表头（含自定义浅色）上表名与旁注清晰可读，不得近白字压浅底**。
