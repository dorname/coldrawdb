## MODIFIED — ST-SP-LIST-03 列表视图表树导航

- 搜索 `user` 后树只剩 users，右网格无需额外点击即切换为 users；点击节点仍保持选中与 Inspector 同步。
- 清空搜索后恢复原选中表，再点击 table_2 验证切表和字段行选中态清空。
- 搜索不存在的关键字时，树节点为 0，右网格显示 `list-view-search-empty`，不得保留 table_2 标题或字段网格；清空后恢复 table_2。
- 画布侧删除当前表后的首表回落与导航不落账保持不变。

## ADDED — ST-SP-LIST-04 列表搜索字段命中提示与网格空态

- **位置**：`frontend-rs/scripts/test-spec-parity-d.mjs`。
- **前置**：预置 users、posts、comments 三表，comments 含字段 `post_id`，表名不含 post。
- **步骤**：进入列表视图，在搜索框输入 `post`，随后输入无命中关键字。
- **断言**：placeholder 明确表名/字段名/类型口径；posts 与 comments 均出现，右网格自动跟随首个匹配 posts；comments 节点显示“字段 post_id 命中”；无命中时 `list-view-search-empty` 可见，旧表标题和字段表格隐藏；清空后恢复原表，全程无新增落账。
- **reporter**：登记 `ST-SP-LIST-03`、`ST-SP-LIST-04` 到 `logos/resources/verify/test-results.jsonl`。

### 附录 A 追加

| ID | 标题 | 对齐实现 |
|---|---|---|
| ST-SP-LIST-04 | 列表搜索字段命中提示与网格空态 | `frontend-rs/scripts/test-spec-parity-d.mjs` |
