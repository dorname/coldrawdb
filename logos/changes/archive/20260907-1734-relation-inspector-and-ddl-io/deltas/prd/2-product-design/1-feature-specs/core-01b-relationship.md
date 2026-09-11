# Delta: core-01b-relationship.md

## MODIFIED — 3. 关系操作

| 操作 | 触发 | 数据变化 |
|---|---|---|
| 创建 | **主要**：关系工具下从字段拖出连线到目标字段；**辅助**：依次点击源字段、目标字段 | 进入确认（生产）或写入 Relationship（原型立即写入）；自连线 = self-reference |
| 选中 / 查看详情 | 单击连线 | 选中该关系（`SelectionKind::Reference`）+ 打开 Inspector 关系面板展示详情（起止字段、cardinality、删除入口）；**不再弹详情模态**（relation-inspector-and-ddl-io：模态与 Inspector 信息重复，已移除） |
| 改 cardinality | 侧栏下拉 | 4 选 1 |
| 改 onUpdate / onDelete | 侧栏下拉 | 5 选 1 |
| 删除 | 选中后 Delete / Backspace，或 Inspector 关系面板「删除关系」按钮 | 从 diagram 移除 |
| 翻转 | 侧栏按钮 | 互换 start/end（不变 cardinality 含义） |

> 点击两点路径必须保留（ST-PB-01 / ST-PU-06）。位移小于 `DRAG_THRESHOLD`（4px）的 pointerdown/up 视为点击，不进入拖线。

## MODIFIED — 渲染参考表「关系线 hover」行

| 关系线 hover / 点击 | 选中 + Inspector 关系面板 | 关系详情：起点、终点、cardinality、onUpdate / onDelete（relation-inspector-and-ddl-io：居中详情模态已移除，详情唯一通路 = Inspector） |
