## MODIFIED — 10.5 列表视图（ListView 表树 + 单表字段网格，PDManer 式）（feat-issue-50-listview-grouping-and-relations / #50 按 Area 分组 + 关联表可视化）

**定位**：字段明细编辑网格（redesign-listview-type-length-canvas-fix 确立行内编辑能力；listview-tree-master-detail 重构布局），对标 PDManer 字段明细表格——列表视图即字段编辑主场，与画布/Inspector 同源同账。**布局为 master-detail 两栏：左侧表树导航 + 右侧当前表字段网格**，解决表多场景下全量堆叠网格的定位困难。

**入口**：AppBar「列表」`[data-testid="btn-view-list"]`，全屏面板 `[data-testid="list-view-panel"]` 占据主区行。

**布局**：

- 左侧表树 `[data-testid="list-tree"]`（固定窄栏，独立纵向滚动）：
  - 顶部搜索框 `[data-testid="list-tree-search"]`：占位文案为「按表/字段/类型/关系搜索…」。除原有表名、字段名和首字段类型模糊匹配外，按关系显示名及完整字段对做大小写不敏感包含匹配；关系命中时该 reference 的两端表均进入可见集合，自环只保留一个表节点。清空恢复全量。
  - 非表名命中的节点显示具体命中原因：字段名命中显示「字段 `<name>` 命中」，首字段类型命中显示「类型 `<type>` 命中」；表名已命中时不追加冗余提示
  - 非表名/字段/类型命中的节点显示具体关系命中原因：「关系 `<name>` 命中」；空名关系使用与画布/Inspector 相同的「未命名关系（id）」稳定回退。表名、字段或类型已命中时不追加冗余关系提示。
  - **树节点按 Area 分组渲染（fix-issue-50 / #50）**：
    - 分组头 `[data-testid="list-tree-group-{area_id}"]` 显示区域名 + 表数量；无 Area 的表归入「未分组」分组
    - 分组依据为画布 Area 与表卡的几何包含关系：表卡中心落入某 Area 矩形内即归该组；未落入任何 Area 的表归入「未分组」
    - 分组支持折叠/展开：点击分组头切换该分组的展开态；搜索时含匹配表的分组自动展开，无匹配分组自动折叠
    - 分组头样式与现有暗色主题一致，与「数据字典」分组视觉同级
  - 树节点 `[data-testid="list-tree-node-<table_name>"]`：显示表名 + 字段数（`表名（N 字段）`）；当前选中态高亮
  - **关系方向标记（feat-listview-relation-details / #61）**：每张表根据自身全部 references 独立显示 `[data-testid="list-tree-relation-direction-<table_name>"]`，只有入边显示「入边」，只有出边显示「出边」，两者兼有显示「双向」；自环同时计入入边和出边，无关系不渲染标记。标记不得受当前选中表、搜索或关联高亮影响，不得再用无方向语义的 `⇄` 代替。title/aria-label 同时给出入边和出边数量，便于区分同对端多关系。
  - **关联表高亮（fix-issue-50 / #50）**：当前表存在关联关系时，树中关联表节点追加 `data-testid="list-tree-node-related"` 并高亮；非关联表节点弱化（`data-testid="list-tree-node-unrelated"`）。该高亮仅表达「与当前选中表相关」，与上述每张表自身的方向标记互不替代。
  - 单击节点 → 选中该表（Inspector 同步，对齐原组头单击语义）；双击节点 → 切回画布并选中该表（对齐原组头双击语义）
  - 当前表基于搜索后的可见表集合解析：原选中表仍命中则保持；未命中则回落首个匹配表；无匹配则为 None。清空搜索后恢复原选中表解析，不因搜索修改 diagram 或产生落账
  - 无匹配时左树显示空态，右侧显示 `[data-testid="list-view-search-empty"]`，并隐藏旧表标题、关系明细和字段网格，不得继续展示搜索前的表
- 右侧单表字段网格（占据剩余宽度，独立纵向滚动）：
  - 网格标题：当前表名 `表名（N 字段）`
  - **逐条关系明细区（feat-listview-relation-details / #61）**：网格标题下方、字段行上方展示当前选中表的全部关系 `[data-testid="list-related-tables"]`，按入边/出边/自环分组。每条 reference 独立渲染 `[data-testid="list-relation-<reference_id>"]`，不得按对端表合并；同对端多条关系均保留，自环只显示一条。每行必须显示：
    - 关系显示名（空名回退「未命名关系（id）」）与相对当前表的方向；
    - 完整字段对及方向化端点基数，例如 `users.id (1) → orders.user_id (N)`；`one_to_one / one_to_many / many_to_one / many_to_many` 分别映射为 `1→1 / 1→N / N→1 / N→M`，未知类型保留原始基数文案，不臆造端点基数；
    - `ON DELETE <策略>` 与 `ON UPDATE <策略>`；空策略显示「未设置」，不得省略整项；
    - 可点击的对端表导航（保留 `[data-testid="list-related-table-<table_name>"]`），点击后在左树定位、展开所在分组并切换右侧网格。自环不提供重复跳转按钮。
    - 无关系时隐藏明细区不占位；该面板只读，不提供关系编辑，也不因查看/搜索/导航落账。
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

**字体层级（ux-canvas-listview-font-size）**：

- 字段网格主文字（`.cdb-list-view-table`）为 `14px`。
- 表树节点主文字（`.cdb-list-tree-node`）为 `14px`。
- 字段网格内输入框/下拉框（`.cdb-list-view-table td .cdb-form-input/select`）为 `14px`。
- 表树节点注释（`.cdb-list-tree-comment`）、节点计数（`.cdb-list-tree-node__count`）为 `12px`。
- 分组头（`.cdb-list-tree__group`、`.cdb-list-tree-group`）保持 `11px` 以维持层级。

**V1 边界**（明确不做）：

- ❌ 主题域树 / 多表 Tab 条 / 逻辑实体 / 多表透视（PDManer 左树体系——本变更的表树仅为表级扁平列表，不含主题域分组层级）
- ❌ 数据域列、列设置、表格式编辑、入库/标注工具
- ❌ 跨表全量网格模式（「全部表」虚拟节点）/ 分组模式切换 / 批量类型面板
- ❌ 用户自定义分组（本期仅按既有 `Table.area` 分组）

> 注：「数据字典」自 S07（feat-data-dictionary）起纳入，入口形态为 ToolRail `toolrail-dicts` + DictPanel 抽屉（非本文件侧栏 Tab 体系，见 §7A 与 `core-01e-data-dictionary.md`），不再列为排除项；ListView 字段网格「说明」列仅追加只读字典映射摘要，绑定编辑仍在 Inspector。

## MODIFIED — 11. 测试用例 ID 索引

| TC ID | 描述 |
|---|---|
| UT-SP-01 | 单击 Tables Tab 表项 → 画布高亮 + 滚动 |
| UT-SP-02 | 搜索 "user" → 列表过滤只含 user* |
| UT-SP-03 | 双击枚举名 → 进入重命名 |
| UT-SP-04 | 引用检查：删除被引用的枚举 → 弹确认 |
| UT-SP-05 | Issues Tab：表名重复 → 错误项出现 |
| UT-SP-06 | Issues Tab：单击错误 → 跳转 + 画布闪烁 |
| UT-SP-07 | DBML Editor：编辑后 Apply → 解析成功 → Diagram 更新 |
| UT-SP-08 | DBML Editor：编辑非法 DBML → 错误消息 + 不应用 |
| ST-SP-01 | 端到端：编辑 5 表 → Issues Tab 显示 0 error |
| UT-SP-09 | 6 业务 Tab 切换（点击 Tab A→B→C，验证激活态 + 内容区切换）— B2 范围 |
| UT-SP-10 | 全局搜索跨 Tab 过滤（spec §10，搜索框过滤 Tables/Areas/Enums 等多 Tab 列表）— B2 范围 |
| UT-SP-11 | 字典面板开合（ToolRail `toolrail-dicts` + DictPanel 抽屉）+ 字典 CRUD/绑定/导出（详细用例见 `core-01e-data-dictionary.md` §8） |
| UT-SP-LIST-REL-01 | 当前表关联集合与每张表自身入边/出边/双向方向聚合 |
| ST-SP-LIST-REL-01 | 列表视图关系明细、树方向标记与对端表导航 |
| UT-SP-LIST-REL-02 | 逐条关系完整字段对、方向化基数、级联策略及关系搜索纯函数 |
| ST-SP-LIST-REL-02 | 16 表 / 19 关系中完整明细与关系名/字段对搜索真实浏览器回归 |
| UT-LV-FONT-01 | ListView 字号合同：主文字 14px，辅助文字 12px，分组头 11px（`frontend-rs/tests/listview_font_ut.rs`） |
