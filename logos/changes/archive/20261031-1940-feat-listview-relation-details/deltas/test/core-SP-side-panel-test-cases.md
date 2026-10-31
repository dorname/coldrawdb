## MODIFIED — ADDED — feat-issue-50-listview-grouping-and-relations / #50 列表视图分组与关联表可视化

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

### UT-SP-LIST-REL-01 — 关联表集合与表关系方向纯函数

- **位置**：`frontend-rs/src/editor_panels.rs`（`related_table_ids` + 表关系方向聚合纯函数）
- **前置**：users → orders、logs → users、users 自环、orders → products，以及无关系表 alone
- **步骤**：分别计算 users、orders、alone 的关联表集合及方向
- **断言**：
  - users 的关联集合含 orders 与 logs，不包含 users 自身；alone 返回空集合
  - users 同时有入边和出边，方向为「双向」；自环同时计入入边/出边且明细不重复
  - 只有入边、只有出边和无关系分别得到「入边」「出边」「无标记」
  - 同对端多条 reference 分别计数，不按对端表去重

### ST-SP-LIST-REL-01 — 列表视图完整关系明细与对端表导航

- **位置**：`frontend-rs/scripts/test-spec-parity-d.mjs`
- **GIVEN**：presetDiagram 预置 users / orders / products 三表及 users → orders、orders → products 两条命名关系，字段、基数、ON DELETE / ON UPDATE 均已知
- **WHEN**：进入列表视图，单击左树 orders 节点
- **THEN**：
  1. 右侧出现 `[data-testid="list-related-tables"]`，每条 reference 独立成行，包含关系名、完整字段对、端点基数及 ON DELETE / ON UPDATE
  2. users 节点显示「出边」，products 节点显示「入边」，orders 节点显示「双向」；标记不依赖当前选中表
  3. 当前表关联节点继续高亮，非关联节点弱化；方向标记与关联高亮可同时存在
  4. 点击对端表 users → 左树定位并选中 users，右侧网格切到 users
  5. 查看和导航前后 PUT 与协作 op 数量不变

### UT-SP-LIST-REL-02 — 逐条关系明细与关系搜索纯函数

- **位置**：`frontend-rs/src/editor_panels.rs` / `frontend-rs/src/editor_core.rs`
- **前置**：包含命名关系、空名关系、同对端多关系、自环，以及四种基数类型、不同 ON DELETE / ON UPDATE 策略
- **步骤**：构造当前表关系明细，并分别按关系名、源字段、目标字段、大小写混合关键字过滤表集合
- **断言**：
  - 每条 reference 独立成行，同对端多关系不合并，自环不重复；空名使用「未命名关系（id）」
  - 字段对完整且 `one_to_one / one_to_many / many_to_one / many_to_many` 分别显示 `1→1 / 1→N / N→1 / N→M` 的端点基数
  - ON DELETE / ON UPDATE 均进入明细；空策略显示「未设置」；未知基数保留原始文案
  - 关系名或任一字段对命中时，该 reference 两端表都可见且给出「关系 `<name>` 命中」；自环只返回一个表节点
  - 原表名/字段名/类型搜索口径不回归；空关键字返回全量，无命中返回空集合

### ST-SP-LIST-REL-02 — 16 表 / 19 关系模型中关系搜索与完整明细

- **位置**：`frontend-rs/scripts/test-spec-parity-d.mjs`（复用 ST-PB-18/19 的电商模型）
- **GIVEN**：16 张表 / 19 条 references；orders 有 users、addresses 两条入边和 order_items、payments、shipments、refunds 四条出边，关系名、字段对、基数和级联策略各异
- **WHEN**：进入列表视图并选中 orders
- **THEN**：
  1. 右侧恰好显示 6 条 orders 关系，`fk_orders_users` 行包含 `users.id (1) → orders.user_id (N)`、`ON DELETE CASCADE`、`ON UPDATE RESTRICT`；其余 5 条也逐条核对名称、字段对、方向化基数和两种策略
  2. orders 树节点为「双向」，users / addresses 为「出边」，payments 为「入边」，无关系节点无方向标记
  3. 搜索 `fk_payments_orders` → 仅 orders / payments 因该关系可见，并显示关系命中原因；搜索 `address_id` → orders / addresses 可见
  4. 清空搜索恢复 16 表；无匹配关键字显示既有空态，不保留搜索前关系明细和字段网格
  5. 搜索、清空及切换对端表均不产生 PUT 或协作 op

### 附录 A 追加

| ID | 标题 | 对齐实现 |
|---|---|---|
| UT-SP-LIST-GROUP-01 | 按 Area 分组纯函数 | `editor_panels.rs::group_tables_by_area` |
| ST-SP-LIST-GROUP-01 | 列表视图按 Area 分组可折叠/搜索 | e2e ListView tree |
| UT-SP-LIST-REL-01 | 关联表集合与表关系方向纯函数 | `editor_panels.rs` 关系集合/方向函数 |
| ST-SP-LIST-REL-01 | 列表视图完整关系明细与对端表导航 | e2e ListView relation details |
| UT-SP-LIST-REL-02 | 逐条关系明细与关系搜索纯函数 | `table_relations_for` / ListView 关系搜索函数 |
| ST-SP-LIST-REL-02 | 16 表 / 19 关系模型中关系搜索与完整明细 | e2e ListView relation search |
