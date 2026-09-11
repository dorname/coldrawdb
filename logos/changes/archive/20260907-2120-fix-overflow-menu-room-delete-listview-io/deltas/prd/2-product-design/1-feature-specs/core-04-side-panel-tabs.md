# Delta: core-04-side-panel-tabs.md

> 提案：fix-overflow-menu-room-delete-listview-io

## MODIFIED — §10.5 布局·工具条条款

> 锚点：替换 §10.5「布局」中右侧单表字段网格的「工具条」一行。

- 右侧单表字段网格（占据剩余宽度，独立纵向滚动）：
  - 网格标题：当前表名 `表名（N 字段）`
  - 工具条：增字段 `[data-testid="list-add-field"]`、删字段、上移、下移（后三者作用于选中字段行，未选中禁用）；右侧依次「导入」`[data-testid="list-btn-import"]`、「导出」`[data-testid="list-btn-export"]`、「返回画布」（fix-overflow-menu-room-delete-listview-io：ListView 补齐 IO 入口）
  - 只渲染当前选中表的字段行（不再按表堆叠组头行）

## ADDED — §10.5 列表视图 IO 入口

> 锚点：追加于 §10.5 列定义表之后。

**ListView 导入/导出（fix-overflow-menu-room-delete-listview-io）**：

- 「导入」`list-btn-import` /「导出」`list-btn-export` 打开与画布态**完全相同**的 IoDrawer（Import/Export 面板），列表视图面板让位（与 Inspector 让位同语义），关闭抽屉后恢复。
- 导入提交走本地合并管道（`merge_import_into_store`，口径见 `core-01d` §4.4）：合并进当前模型 → 落账 PUT → Toast `notice-toast`「已导入 N 张表 / M 条关系」；合并后表树刷新（新表出现在树尾部），不跳转、不调 bridge。
- 导出面板预览/复制/下载与画布态一致，数据源为同一 store。
- 只读（Viewer / read_only）：「导入」禁用（hover title 说明只读原因），「导出」可用。
