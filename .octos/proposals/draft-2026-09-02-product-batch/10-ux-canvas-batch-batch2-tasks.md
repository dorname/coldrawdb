# 实现任务 — ux-canvas-batch 批次 2（细化 tasks，C-1 落实）

> 状态：**细化 tasks**（落 `.octos/proposals/`，**未**走 `openlogos change`）
> 配套：`logos/changes/ux-canvas-batch/proposal.md` + `tasks.md`（commit `be8da4d`）
> 上游裁决链：黑板条目6 operator 批注（Q1=全部 9 项列表视图候选）→ 外环判词 C-1..C-4 → 批次 1 复验采认（commit `039c93d`）→ 外环 steer 派发批次 2 前置任务（C-1：先起草批次 2 细化 tasks）
> 外环强制约束：禁止改测试断言；tasks 不写 verify/smoke/archive 条目（独立 CLI 节点）；新 UT/ST 编号先 grep 占用情况取下一空闲（UT-MM-23 起）
> **语义记一笔**（外环判词）：`SortColumn::Type` 实现取**首个字段类型**做表级排序键（`a.fields.first()`），空表回退 `""`——批次 2 过滤规则与 SortColumn::Type 首字段类型口径对齐

## [code] 代码实现（批次 2：过滤 + 批量重命名）

### 过滤（按名称模糊匹配/按类型/按是否有索引）

- [ ] `frontend-rs/src/editor_panels.rs`: `ListViewState` 新增过滤字段：
  - `filter_query: RwSignal<String>`（按名称模糊匹配——表名/字段名/类型）
  - `filter_type: RwSignal<String>`（按类型过滤——与 `SortColumn::Type` 首字段类型口径对齐：取**首个字段类型**做表级过滤键，空表回退 `""`）
  - `filter_has_index: RwSignal<Option<bool>>`（按是否有索引过滤——`Some(true)` = 仅有索引，`Some(false)` = 仅无索引，`None` = 不过滤）
- [ ] `frontend-rs/src/editor_panels.rs`: 新增 `filter_tables` 纯函数（UT-MM-23）：
  - 按名称模糊匹配：表名/字段名/类型含 `filter_query` 子串（大小写不敏感）
  - 按类型过滤：表级过滤键 = `table.fields.first().map(|f| f.type_.as_str()).unwrap_or("")`（与 `SortColumn::Type` 首字段类型口径对齐——**外环判词语义记一笔**）
  - 按是否有索引过滤：`table.indices.is_empty()` 反向（`Some(true)` = `!indices.is_empty()`，`Some(false)` = `indices.is_empty()`）
  - 组合过滤：三条件 AND（同时满足）
- [ ] `frontend-rs/src/editor_panels.rs`: 过滤 UI（ListView 组件内）：
  - 搜索框（按名称模糊匹配）：data-testid `list-view-filter-query`
  - 类型下拉（按类型过滤）：data-testid `list-view-filter-type`；选项从现有 tables 的首个字段类型去重派生
  - 索引复选框（按是否有索引过滤）：data-testid `list-view-filter-has-index`；三态（不过滤/仅有索引/仅无索引）

### 批量重命名（多表或多字段一次性改名）

- [ ] `frontend-rs/src/editor_panels.rs`: 新增 `batch_rename_tables` 纯函数（UT-MM-24）：
  - 输入：`tables: &mut Vec<Table>`、`rename_map: HashMap<String, String>`（旧名 → 新名）
  - 规则（**重名冲突处理真值表**，外环判词 C-1 强制 + B2-S1 补充规则）：

    | 旧名 | 新名 | 新名是否已存在（改名前快照） | 结果 |
    |---|---|---|---|
    | A | B | 否 | A → B（改名成功） |
    | A | B | 是（另一表已用 B） | A 跳过（不改名，保持原名 A） |
    | A | A | 是（自身） | A 跳过（不改名，保持原名 A） |
    | A | "" | — | A 跳过（不改名，保持原名 A） |
    | A | B（含非法字符） | — | A 跳过（不改名，保持原名 A） |
    | A | B | 否（但另一旧名也映射到 B） | 字典序靠前者得名，其余跳过（B2-S1 ③） |

  - **B2-S1 补充规则**（外环 steer 强制）：
    - ①冲突判定以**改名前快照**为准（`{A→B, B→C}` 全跳过——B→C 时 B 仍存在于改名前快照，C 冲突）
    - ②处理顺序按**旧名字典序**（`{B→D, A→D}` → A 先处理，A→D 成功，B→D 跳过）
    - ③同一新名多旧名映射（`{A→C, B→C}`）→ 字典序靠前者得名（A→C 成功），其余跳过（B→C 跳过）

  - **实例推演**（外环判词 C-1 强制 + B2-S1 补充）：
    - 场景 1：tables = [A, B, C]，rename_map = {A→D} → A→D（改名成功），B/C 不变
    - 场景 2：tables = [A, B, C]，rename_map = {A→B} → A 跳过（新名 B 已存在，保持原名 A）
    - 场景 3：tables = [A, B, C]，rename_map = {A→A} → A 跳过（新名 = 原名，保持原名 A）
    - 场景 4：tables = [A, B, C]，rename_map = {A→""} → A 跳过（新名为空，保持原名 A）
    - 场景 5：tables = [A, B, C]，rename_map = {A→"A-B"} → A→A-B（合法字符，改名成功）
    - 场景 6（B2-S1 ①）：tables = [A, B, C]，rename_map = {A→B, B→C} → A 跳过（新名 B 已存在），B 跳过（新名 C 已存在）——**冲突判定以改名前快照为准**
    - 场景 7（B2-S1 ②）：tables = [A, B, C]，rename_map = {B→D, A→D} → A→D（字典序靠前，改名成功），B 跳过（新名 D 已被 A 占用）——**处理顺序按旧名字典序**
    - 场景 8（B2-S1 ③）：tables = [A, B, C]，rename_map = {A→C, B→C} → A→C（字典序靠前，改名成功），B 跳过（新名 C 已被 A 占用）——**同一新名多旧名映射，字典序靠前者得名其余跳过**
  - 批量改名后 `store.dirty.set(true)`（标记脏，触发自动保存）
- [ ] `frontend-rs/src/editor_panels.rs`: 批量重命名 UI（ListView 组件内）：
  - 复选框（多选表）：data-testid `list-view-select-{table_id}`
  - 批量改名按钮：data-testid `list-view-batch-rename`；点击后弹出批量改名模态
  - 批量改名模态：data-testid `modal-batch-rename`；输入框（旧名 → 新名，支持多行）；data-testid `modal-input-batch-rename`
  - 批量改名模态 Apply 按钮：data-testid `modal-submit-batch-rename`；调用 `batch_rename_tables` 纯函数写入 store

### 测试（产出代码，不产出 delta；非 verify/smoke 节点）

> 说明：本 section 列出**代码实现同步产出**的测试用例，**非** verify/smoke/人工验证条目；按 SKILL"禁止在 tasks.md 写 verify/smoke/人工验证类条目"原则，verify/validate-ledger/openlogos-verify 等节点属独立 CLI 操作，**不**列入 tasks。

- [ ] `frontend-rs/src/editor_panels.rs` 单元测试模块（新增 UT-MM-23）：
  - happy: `filter_tables(tables, "users", "", None)` → 表名/字段名/类型含 "users" 子串的表
  - happy: `filter_tables(tables, "", "INT", None)` → 首个字段类型为 INT 的表（与 `SortColumn::Type` 首字段类型口径对齐——**外环判词语义记一笔**）
  - happy: `filter_tables(tables, "", "", Some(true))` → 仅有索引的表
  - happy: `filter_tables(tables, "", "", Some(false))` → 仅无索引的表
  - happy: `filter_tables(tables, "users", "INT", Some(true))` → 表名/字段名/类型含 "users" 子串 + 首个字段类型为 INT + 有索引的表（三条件 AND）
  - edge: `filter_tables(tables, "nonexistent", "", None)` → 空结果
  - edge: `filter_tables(tables, "", "", None)` → 全部表（不过滤）
- [ ] `frontend-rs/src/editor_panels.rs` 单元测试模块（新增 UT-MM-24，**8 子用例**——B2-S1 ③ 同一新名多旧名映射补第 8 子用例）：
  - happy: `batch_rename_tables(tables, {A→D})` → A→D（改名成功），B/C 不变
  - happy: `batch_rename_tables(tables, {A→B})` → A 跳过（新名 B 已存在，保持原名 A）
  - happy: `batch_rename_tables(tables, {A→A})` → A 跳过（新名 = 原名，保持原名 A）
  - happy: `batch_rename_tables(tables, {A→""})` → A 跳过（新名为空，保持原名 A）
  - happy: `batch_rename_tables(tables, {A→"A-B"})` → A→A-B（合法字符，改名成功）
  - edge: `batch_rename_tables(tables, {})` → 全部不变（空 rename_map）
  - edge: `batch_rename_tables(tables, {D→E})` → 全部不变（旧名 D 不存在）
  - **B2-S1 ③**: `batch_rename_tables(tables, {A→C, B→C})` → A→C（字典序靠前，改名成功），B 跳过（新名 C 已被 A 占用）——**同一新名多旧名映射，字典序靠前者得名其余跳过**

## [spec] 规格登记（代码实现同步，非独立 delta 任务）

- [ ] 在 `logos/resources/test/core-UI-modals-2-test-cases.md`（或同类 modals 测试用例 spec 文件）追加 UT-MM-23/24 行：
  ```
  | UT-MM-23 | 列表视图过滤纯函数测试（按名称模糊匹配/按类型/按是否有索引；与 SortColumn::Type 首字段类型口径对齐） | `editor_panels.rs::filter_tables` |
  | UT-MM-24 | 列表视图批量重命名纯函数测试（重名冲突处理：新名已存在→跳过/新名=原名→跳过/新名为空→跳过） | `editor_panels.rs::batch_rename_tables` |
  ```
- [ ] 确认 UT-MM-23/24 已被 reporter 写入 `test-results.jsonl`（cargo test 触发）

## 实现顺序建议（批次 2）

1. `ListViewState` 新增过滤字段（filter_query/filter_type/filter_has_index）
2. `filter_tables` 纯函数 + UT-MM-23（7 子用例）
3. 过滤 UI（ListView 组件内：搜索框/类型下拉/索引复选框）
4. `batch_rename_tables` 纯函数 + UT-MM-24（7 子用例）
5. 批量重命名 UI（ListView 组件内：复选框/批量改名按钮/批量改名模态）
6. spec 登记（UT-MM-23/24 行）

每步独立 commit，commit message 格式 `feat(<module>): ...`。

## 不在范围（明确排除）

- 批量改类型（多字段一次性改类型）——批次 3
- 双击跳到画布对应表——批次 3
- 导出 CSV/Excel——批次 3
- 列宽可调——批次 4
- 表/字段分组（按 schema/按 tag）——批次 4
- 样式优化（字体回退栈补思源黑体/苹方 + Canvas 文本离屏缓存 + 关键交互帧率 < 16ms（**C-2：降为代码审查项 + 可选基准脚本，不作为 verify 门禁断言**）+ rAF 统一调度）——批次 4
- 大图（>200 表）虚拟化（operator Q5 裁决暂缓）
- 不修改 `Reference`/`Table` struct 数据契约
- 不修改 reference 连线布局的端点计算算法
- 不修改 Inspector 其它字段编辑逻辑
- 不修改测试断言（外环强制约束）
- 不写 verify/smoke/archive 条目（独立 CLI 节点）

## 外环判词 C-1 落实（批次 2 细化 tasks 留白规则）

- **批量重命名的重名冲突处理**：真值表 + 实例推演已给出（见 `batch_rename_tables` 纯函数段）
- **过滤规则与 SortColumn::Type 首字段类型口径对齐**：`filter_tables` 纯函数的按类型过滤取**首个字段类型**做表级过滤键（`table.fields.first().map(|f| f.type_.as_str()).unwrap_or("")`），空表回退 `""`——与 `SortColumn::Type` 首字段类型口径对齐（**外环判词语义记一笔**）
- **新 UT 编号**：UT-MM-23/24（grep 确认 UT-MM-10..22 全部占用，UT-MM-23/24 空闲）