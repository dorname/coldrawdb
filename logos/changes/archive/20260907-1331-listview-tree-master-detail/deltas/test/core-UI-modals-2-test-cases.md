# Delta: core-UI-modals-2-test-cases.md

## MODIFIED — UT-MM-22 列表视图 tab 切换测试

附录 A 中 UT-MM-22 行更新为：

| UT-MM-22 | 列表视图 tab 切换测试（切列表视图出树+单表网格布局：左树节点 + 右网格仅当前表字段行；切回画布恢复） | `editor_panels.rs::ListView` |

## MODIFIED — ST-LV-01 ListView e2e

附录 A 中 ST-LV-01 行更新为：

| ST-LV-01 | ListView 表树+单表网格 e2e（左树出全部表节点且首表默认选中；点树节点切换右网格为对应表字段行并 Inspector 同步；字段行内嵌编辑控件；双击树节点/字段行跳画布选中该表；组头行锚点 `list-view-group-*` 移除，树节点锚点 `list-tree-node-*` 生效） | `frontend-rs/scripts/test-spec-parity-d.mjs` |

## ADDED — UT-MM-38 表树过滤与选中状态纯函数测试

### UT-MM-38 — 表树过滤与当前表解析纯函数测试

- **位置**：`frontend-rs/src/editor_panels.rs`（`filter_tables` 复用口径 + 当前表解析纯函数 `resolve_list_current_table`）
- **步骤与预期**：
  1. `filter_tables(tables, "user")` → 仅表名含 user（大小写不敏感）的表；空关键字 → 全量；无命中 → 空数组
  2. `resolve_list_current_table(tables, list_table_id)`：`list_table_id` 命中 → 该表；为 None 或 id 已不存在（被删）→ 首张表；空表数组 → None
  3. 过滤不影响当前表解析：搜索无命中时当前表仍按规则 2 解析（导航与过滤解耦）

附录 A 新增行：

| UT-MM-38 | 表树过滤与当前表解析纯函数测试（filter_tables 大小写不敏感包含/空关键字全量/无命中空数组；resolve_list_current_table 命中/回落首表/空数组 None/过滤与导航解耦） | `editor_panels.rs::filter_tables` + `resolve_list_current_table` |
