# Delta — core-PB-relationship-test-cases.md（新文件）

## ADDED — Phase B 关系工具测试用例

| TC ID | Given | When | Then |
|-------|-------|------|------|
| UT-PB-01 | 表含字段 (100,130) | `hit_test_field` | `Some((table_id, field_id))` |
| UT-PB-02 | draft 两端字段 | `build_reference` | `type_==one_to_many`, on_delete==RESTRICT |
| UT-PB-03 | reference A→B | `flip_reference_endpoints` | start/end 互换 |
| UT-PB-04 | 表两字段 f1 PK | `toggle_field_primary(f2)` | f2.primary=true, f1.primary=false |
| UT-PB-05 | 确认条可见 | 点 create | `references.len()+1` |
| ST-PB-01 | 两张表各一字段 | 关系工具双点+确认 | Inspector 可编辑关系 |
