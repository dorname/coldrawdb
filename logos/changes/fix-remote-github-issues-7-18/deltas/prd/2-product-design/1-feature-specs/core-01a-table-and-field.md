# Delta — core-01a-table-and-field.md（修改）

> 模块：core | 提案：fix-remote-github-issues-7-18
> 关联 issue：#10（画布表/字段中文注释展示）、#12（表头/表边框颜色配置）

## ADDED — §1.5 画布注释展示与显示开关（fix-remote-github-issues-7-18）

> 实现 issue #10。`Table.comment` / `Field.comment` 数据层与 Inspector 编辑入口已存在（§1.4）；本提案补齐**画布渲染**与**显示开关**，渲染细则见 `core-01-editor-canvas.md` §5.8 R-CMT-01～04。

**显示开关**：

| 模式 | 含义 | 备注 |
|---|---|---|
| `name` | 仅英文名（技术标识符） | 与现状一致 |
| `name+comment` | **默认（用户已拍板）**：英文名为主 + 灰色中文注释 | 有 comment 才显示注释部分 |
| `comment` | 仅注释（无 comment 的表/字段回退显示英文名，不留空白） | 浏览业务语义用 |

- 开关位置：画布工具区 / View 菜单（`data-testid="canvas-comment-display"`），三态切换即重绘
- 模式为**视图偏好**：写入 `localStorage`（key `cdb.comment-display`），不写入 diagram 数据、不影响保存链路
- 空 comment 行为与现状完全一致：不渲染注释、不留占位
- hover tooltip：注释被省略截断时 `title` 显示完整 comment

**验收口径**（与 issue #10 对齐）：

1. 表/字段 `comment` 非空时画布可见中文注释（或可切换可见）
2. comment 为空时行为与现状一致，不留空白占位噪音
3. 导出/导入后 comment 仍在，画布刷新后仍可见

## ADDED — §1.6 表颜色配置（fix-remote-github-issues-7-18）

> 实现 issue #12 的表侧。`Table.color` 字段已存在且表头渐变已使用；本提案补齐**表边框着色**与 **Inspector 编辑入口**。

| 规则 | 规格 |
|---|---|
| Inspector 入口 | 「数据表」section 注释输入下方追加「颜色」选择器（`data-testid="inspector-table-color"`）：预设色板（与主原型强调色选项对齐）+ 「默认」项（清空回 `''`）；变更走既有落账链路（blur/change → store → `dirty` + `schedule_save`），进 UndoRedoContext |
| 渲染 | `color` 非空：表头渐变（现状）+ **表边框**跟随该色；为空：保持 `palette` 默认（`header_tint` / `table_border`） |
| 持久化 | `color` 已随 diagram JSON 保存（`diagrams.yaml` `Table.color` 既有）；无需迁移 |
| 导入导出 | JSON 保留；SQL/DBML 降级忽略（现状不变） |
| 兼容 | 存量图 `color=''` → 视觉与现状一致 |

**关系线颜色**入口与合同见 `core-01b-relationship.md` §4.1。

## MODIFIED — 5. 测试用例 ID 索引

> 在原表末尾追加（既有条目保留）：

| TC ID | 描述 |
|---|---|
| UT-CR-COMMENT-01 | 注释渲染口径：三态显示模式 + 空 comment 不留占位（画布绘制参数纯函数） |
| UT-CR-COMMENT-02 | 显示开关读写 `localStorage["cdb.comment-display"]`，默认 `name+comment` |
| UT-CR-COLOR-01 | 表边框用色：`table.color` 非空跟随、为空回退 palette；关系线 `ref.color` 同理 |
| ST-CR-COMMENT-01 | e2e：导入含中文 COMMENT 的 SQL → 画布表头/字段行可见注释 → 刷新仍在 |
| ST-CR-COLOR-01 | e2e：Inspector 改表颜色 → 表头/边框即时更新 → 保存刷新后保留；导出 JSON 再导入颜色仍在 |

> 详细步骤见 `core-CR-canvas-test-cases.md`；Inspector 注释编辑既有用例（UT-PC-29 / UT-PC-30 / ST-PC-08）继续有效。
