## MODIFIED — §10.5 列表视图（ListView 表树 + 单表字段网格，PDManer 式）（feat-issue-50-listview-grouping-and-relations / #50 按 Area 分组 + 关联表可视化）

**定位**：字段明细编辑网格（redesign-listview-type-length-canvas-fix 确立行内编辑能力；listview-tree-master-detail 重构布局），对标 PDManer 字段明细表格——列表视图即字段编辑主场，与画布/Inspector 同源同账。**布局为 master-detail 两栏：左侧表树导航 + 右侧当前表字段网格**，解决表多场景下全量堆叠网格的定位困难。

**入口**：AppBar「列表」`[data-testid="btn-view-list"]`，全屏面板 `[data-testid="list-view-panel"]` 占据主区行。

**布局**：

- 左侧表树 `[data-testid="list-tree"]`（固定窄栏，独立纵向滚动）：
  - 顶部搜索框 `[data-testid="list-tree-search"]`：按表名模糊匹配过滤（复用 Tables Tab 搜索口径 `filter_tables`），清空恢复全量
  - **树节点按 Area 分组渲染（fix-issue-50 / #50）**：
    - 分组头 `[data-testid="list-tree-group-{area_id}"]` 显示区域名 + 表数量；无 Area 的表归入「未分组」分组
    - 分组支持折叠/展开：点击分组头切换该分组的展开态；搜索时含匹配表的分组自动展开，无匹配分组自动折叠
    - 分组头样式与现有暗色主题一致，与「数据字典」分组视觉同级
  - 树节点 `[data-testid="list-tree-node-<table_name>"]`：显示表名 + 字段数（`表名（N 字段）`）；当前选中态高亮
  - **关联表高亮（fix-issue-50 / #50）**：当前表存在关联关系时，树中关联表节点追加 `data-testid="list-tree-node-related"` 并高亮；非关联表节点弱化（`data-testid="list-tree-node-unrelated"`）
  - 单击节点 → 选中该表（Inspector 同步，对齐原组头单击语义）；双击节点 → 切回画布并选中该表（对齐原组头双击语义）
  - 默认选中首张表；搜索无命中 → 树内空态提示
- 右侧单表字段网格（占据剩余宽度，独立纵向滚动）：
  - 网格标题：当前表名 `表名（N 字段）`
  - **关联表摘要区（fix-issue-50 / #50）**：网格标题下方、字段行上方展示当前选中表的关联表清单 `[data-testid="list-related-tables"]`，按入边/出边分组，显示关系类型（1:1 / 1:N / N:M）；点击关联表名在左树中定位并切换选中。无关联时隐藏不占位。
  - 工具条：增字段 `[data-testid="list-add-field"]`、删字段、上移、下移（后三者作用于选中字段行，未选中禁用）；右侧依次「导入」`[data-testid="list-btn-import"]`、「导出」`[data-testid="list-btn-export"]`、「返回画布」（fix-overflow-menu-room-delete-listview-io：ListView 补齐 IO 入口）
  - 只渲染当前选中表的字段行（不再按表堆叠组头行）
- 列定义（对齐 PDManer，数据域列不做）：

| 列 | 绑定 | 编辑方式 |
|---|---|---|
| 字段代码 | `field.name` | 文本输入（blur 落账） |
| 显示名称 | `field.comment` | 文本输入（blur 落账） |
| 主键 | `field.primary` | checkbox（即改即账） |
| 不为空 | `field.not_null` | checkbox |
| 唯一 | `field.unique` | checkbox |
| 自增 | `field.increment` | checkbox |
| 数据类型 | `field.type_` 基类型 | 下拉（按引擎过滤，core-01a §2.2 基类型清单） |
| 长度 | `type_` 参数 | 数字输入，仅 VARCHAR 系可编辑，缺省 255 |
| 小数位 | `type_` 参数 | 数字输入，仅 DECIMAL/NUMERIC 系可编辑，缺省 2（精度列并入长度列口径：DECIMAL(长度,小数位)） |
| 说明 | `field.tag` | 文本输入（受控防抖，blur 落账） |

**ListView 导入/导出（fix-overflow-menu-room-delete-listview-io）**：

- 「导入」`list-btn-import` /「导出」`list-btn-export` 打开与画布态**完全相同**的 IoDrawer（Import/Export 面板），列表视图面板让位（与 Inspector 让位同语义），关闭抽屉后恢复。
- 导入提交走本地合并管道（`merge_import_into_store`，口径见 `core-01d` §4.4）：合并进当前模型 → 落账 PUT → Toast `notice-toast`「已导入 N 张表 / M 条关系」；合并后表树刷新（新表出现在树尾部），不跳转、不调 bridge。
- 导出面板预览/复制/下载与画布态一致，数据源为同一 store。
- 只读（Viewer / read_only）：「导入」禁用（hover title 说明只读原因），「导出」可用。

**行为**：

- 行内编辑直接落账（沿用既有 PUT 快照通路；文本类 blur/防抖落账，选择与勾选即改即账）
- 类型/长度/小数位三联：任一变更按 core-01a §2.2 参数化口径重新合成 `type_` 整串（`VARCHAR(32)` / `NUMERIC(10,2)`）
- 增字段：在**当前树选中表**末尾追加默认字段；删字段：删除选中行（无需确认模态，可撤销）；上移/下移：字段在表内排序 ±1
- 选中行高亮（`state.listSel` 语义：{table_id, field_id}）；**切换树选中表时清空字段行选中态**；双击字段行 → 切回画布并选中该表（保留 ST-SP-LIST-01 既有语义）
- 无表 → 整体空态提示，不布局塌陷；当前选中表被删除（如画布侧操作）→ 回落选中首张剩余表

**V1 边界**（明确不做）：

- ❌ 主题域树 / 多表 Tab 条 / 逻辑实体 / 多表透视（PDManer 左树体系——本变更的表树仅为表级扁平列表，不含主题域分组层级）
- ❌ 数据域列、列设置、表格式编辑、入库/标注工具
- ❌ 跨表全量网格模式（「全部表」虚拟节点）/ 分组模式切换 / 批量类型面板
- ❌ 用户自定义分组（本期仅按既有 `Table.area` 分组）

> 注：「数据字典」自 S07（feat-data-dictionary）起纳入，入口形态为 ToolRail `toolrail-dicts` + DictPanel 抽屉（非本文件侧栏 Tab 体系，见 §7A 与 `core-01e-data-dictionary.md`），不再列为排除项；ListView 字段网格「说明」列仅追加只读字典映射摘要，绑定编辑仍在 Inspector。
