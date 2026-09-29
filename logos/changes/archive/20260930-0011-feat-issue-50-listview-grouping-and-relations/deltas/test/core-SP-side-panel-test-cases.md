## ADDED — feat-issue-50-listview-grouping-and-relations / #50 列表视图分组与关联表可视化

### UT-SP-LIST-GROUP-01 — 按 Area 分组纯函数

- **位置**：`frontend-rs/src/editor_panels.rs`（新增纯函数 `group_tables_by_area`）
- **前置**：store 中存在 tables: `[t1(area="核心域"), t2(area="核心域"), t3(area=""), t4(area="扩展域")]`
- **步骤**：调用 `group_tables_by_area(tables)`
- **断言**：
  - 返回 3 个分组：「核心域」含 t1/t2，「扩展域」含 t4，「未分组」含 t3
  - 分组内保持原始表序
  - 空输入返回空 Vec

### ST-SP-LIST-GROUP-01 — 列表视图按 Area 分组可折叠/搜索

- **位置**：`frontend-rs/scripts/test-spec-parity-d.mjs`
- **GIVEN**：presetDiagram 预置 4 张表：t1/t2 属于 area="核心域"，t3 无 area，t4 属于 area="扩展域"
- **WHEN**：进入列表视图
- **THEN**：
  1. 左侧表树显示分组头「核心域（2）」、「扩展域（1）」、「未分组（1）」
  2. 点击「核心域」分组头后该分组折叠，节点 `list-tree-node-t1`、`list-tree-node-t2` 不可见
  3. 在搜索框输入 "t4" → 仅「扩展域」分组自动展开并显示 t4，其他分组折叠/无匹配
  4. 清空搜索 → 所有分组恢复展开

### UT-SP-LIST-REL-01 — 关联表集合纯函数

- **位置**：`frontend-rs/src/editor_panels.rs`（新增纯函数 `related_table_ids`）
- **前置**：store 中存在 tables: `[users, orders, products]`，references: `[orders.user_id → users.id（1:N）, orders.product_id → products.id（N:1）]`
- **步骤**：调用 `related_table_ids("orders", tables, refs)`
- **断言**：
  - 返回集合含 `users`（出边）与 `products`（出边）
  - 不包含 `orders` 自身
  - 无关系表返回空集合

### ST-SP-LIST-REL-01 — 选中表时展示关联表清单并高亮树节点

- **位置**：`frontend-rs/scripts/test-spec-parity-d.mjs`
- **GIVEN**：presetDiagram 预置 users / orders / products 三表及上述 references
- **WHEN**：进入列表视图，单击左树 orders 节点
- **THEN**：
  1. 右侧字段网格上方出现 `[data-testid="list-related-tables"]`，清单含 users、products 及关系类型
  2. 左树中 `list-tree-node-users`、`list-tree-node-products` 带 `data-testid="list-tree-node-related"` 高亮属性
  3. `list-tree-node-users` 等非关联表无高亮属性
  4. 点击关联表清单中的 users → 左树定位并选中 users，右侧网格切到 users

### 附录 A 追加

| ID | 标题 | 对齐实现 |
|---|---|---|
| UT-SP-LIST-GROUP-01 | 按 Area 分组纯函数 | `editor_panels.rs::group_tables_by_area` |
| ST-SP-LIST-GROUP-01 | 列表视图按 Area 分组可折叠/搜索 | e2e ListView tree |
| UT-SP-LIST-REL-01 | 关联表集合纯函数 | `editor_panels.rs::related_table_ids` |
| ST-SP-LIST-REL-01 | 选中表时展示关联表清单 | e2e ListView related panel |
