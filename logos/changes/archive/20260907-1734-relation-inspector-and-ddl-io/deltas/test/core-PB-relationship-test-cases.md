# Delta: core-PB-relationship-test-cases.md

## MODIFIED — ST-PB-03 用例行（主表）

| ST-PB-03 | 已有一条可见关系（两表拖开不重叠） | 点击连线中点 | 连线选中高亮；**不弹详情模态**（`modal-reference-detail` 不存在）；Inspector 关系面板可见且 label 为 `table_1.id → table_2.id`；点 Inspector「删除关系」→ 落账 `references.len()==0` |

## MODIFIED — ST-PB-04 用例行（主表）

| ST-PB-04 | 已有一条可见关系 | 点击连线选中 → 按 Delete 键；重建关系后再次选中 → 按 Backspace 键 | 两次均落账 `references.len()==0`（Delete 与 Backspace 双键覆盖） |

## ADDED — 变更记录行（附录）

| ST-PB-03/04（MODIFIED，relation-inspector-and-ddl-io） | MODIFIED | 点击连线不再弹详情模态（Inspector 为唯一详情通路）；删除收敛为 Inspector 按钮 + Delete/Backspace 双键 |
