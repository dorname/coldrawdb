## MODIFIED — §10.5 列表视图（ListView 表树 + 单表字段网格）

在现有布局与交互不变的前提下，调整 ListView 字体层级，提升可读性：

- 字段网格主文字（`.cdb-list-view-table`）由 `12px` 调整为 `14px`。
- 表树节点主文字（`.cdb-list-tree-node`）由 `12px` 调整为 `14px`。
- 字段网格内输入框/下拉框（`.cdb-list-view-table td .cdb-form-input/select`）由 `12px` 调整为 `14px`。
- 表树节点注释（`.cdb-list-tree-comment`）由 `10px` 调整为 `12px`。
- 节点计数（`.cdb-list-tree-node__count`）由 `11px` 调整为 `12px`。
- 分组头（`.cdb-list-tree__group`、`.cdb-list-tree-group`）保持较小字号以维持层级，由 `10px` 调整为 `11px`。

上述字号作为 UI 合同，由 `UT-LV-FONT-01` 单元测试锁定。
