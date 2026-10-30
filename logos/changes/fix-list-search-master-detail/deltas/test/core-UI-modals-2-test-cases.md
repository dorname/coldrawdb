## MODIFIED — UT-MM-38 表树过滤与当前表解析纯函数测试

保留 `resolve_list_current_table` 对输入数组的命中/首表/None 规则。ListView 组合测试改为：先将搜索后的数组传给 resolver；原选中不在结果中时回落首个匹配，无匹配数组返回 None。删除“搜索无命中不影响当前表”的旧预期。

## ADDED — UT-MM-39 列表搜索命中原因纯函数

- **位置**：`frontend-rs/src/editor_panels.rs`。
- **操作**：对表名命中、字段名命中、首字段类型命中、无命中及空关键字分别调用搜索命中原因 helper。
- **断言**：表名命中不显示冗余提示；字段命中返回包含具体字段名的提示；类型命中返回具体类型；无命中和空关键字返回 None；大小写口径与 `filter_tables` 一致。
- **reporter**：登记 `UT-MM-38`、`UT-MM-39` 到 `logos/resources/verify/test-results.jsonl`。

### 附录 A 追加

| ID | 标题 | 对齐实现 |
|---|---|---|
| UT-MM-39 | 列表搜索字段/类型命中原因 | `editor_panels.rs::list_tree_search_match` |
