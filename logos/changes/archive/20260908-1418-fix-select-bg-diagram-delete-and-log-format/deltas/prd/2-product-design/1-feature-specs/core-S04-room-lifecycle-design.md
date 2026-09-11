# Delta: core-S04-room-lifecycle-design.md

> 提案：fix-select-bg-diagram-delete-and-log-format

## MODIFIED — Create Room 模态字段表（diagram 行）

diagram 行改为：

| diagram | ✅ | 下拉已有 diagram 或当前 diagram；选「新建空白模型」时先创建 diagram 再绑定房间。选项数据源 `GET /diagrams/queryAll` **只返回未软删的 diagram**（`is_deleted=0`，fix-select-bg-diagram-delete-and-log-format）；选中既有图时选择器下方显示「删除此图表」按钮 `[data-testid="create-room-diagram-delete"]`，点击弹二次确认模态 `[data-testid="modal-delete-diagram"]`（文案：图表及其画布内容将被删除、已绑定房间的图不可在此删除——选项本身不含已绑定图），确认调 `DELETE /api/v1/diagrams/{id}`，成功后刷新选项、选中态回退「新建空白模型」并 Toast「已删除图表」 |
