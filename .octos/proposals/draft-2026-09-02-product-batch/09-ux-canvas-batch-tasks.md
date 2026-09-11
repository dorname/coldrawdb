# 实现任务 — ux-canvas-batch

> 状态：**草案**（落 `.octos/proposals/`，**未**走 `openlogos change`）
> 配套：`08-ux-canvas-batch-proposal.md`
> 上游裁决：黑板条目6 operator 批注（Q1=全部 9 项列表视图候选；Q5=外环代决样式子集：字体回退栈+子像素抗锯齿+中文字体思源黑体/苹方+Canvas 文本离屏缓存+关键交互<16ms+rAF 统一调度；大图虚拟化暂缓）
> 上游判词：条目6 尾部 feat-relation-inference 归档通报（cb56c39 已 push）
> 外环强制约束：禁止改测试断言；tasks 不写 verify/smoke/archive 条目（独立 CLI 节点）；新 UT/ST 编号先 grep 占用情况取下一空闲（UT-MM-21 起）
> 分批建议（8 天档）：4 批次（基础表格化 → 过滤/批量重命名 → 批量改类型/导出 → 列宽/分组/样式优化）

## [code] 代码实现（批次 1：基础表格化 + 排序）

### SidePanelTab 新增 ListView tab

- [ ] `frontend-rs/src/editor_panels.rs`: `SidePanelTab:215-249` 新增 `ListView` 变体：
  - testid: `"tab-list-view"`
  - label: `"列表视图"`
  - 枚举变体插入到 `Tables` 之后（顺序：Tables → ListView → Areas → ...）
- [ ] `frontend-rs/src/editor_panels.rs`: 侧栏 tab 切换逻辑（ListView tab 激活时显示列表视图）：
  - `active_tab.set(SidePanelTab::ListView)` 时渲染列表视图组件
  - data-testid 命名：`list-view-panel`（列表面板）

### 列表视图组件（基础表格化展示）

- [ ] `frontend-rs/src/editor_panels.rs`: 新增 `ListView` 组件（表结构列表视图）：
  - 表名/字段名/类型表格化展示（`<table>` 元素，列：表名/字段名/类型）
  - 数据派生：从 `store.tables.get()` 取所有 table，遍历 `table.fields` 生成行
  - data-testid 命名：`list-view-table`（表格元素）
  - **D 案教训**：涉及推导/状态机必须给真值表+实例推演（见 proposal 真值表段）

### 排序（按任意列）

- [ ] `frontend-rs/src/editor_panels.rs`: 新增 `ListViewState`（排序列/排序方向）：
  - `sort_column: RwSignal<SortColumn>`（枚举：TableName/FieldCount/Type/HasIndex）
  - `sort_direction: RwSignal<SortDirection>`（枚举：Ascending/Descending）
  - 排序规则（真值表见 proposal）：
    - 表名：字典序升序/降序
    - 字段数：少→多 / 多→少
    - 类型：字典序升序/降序（如 INT < VARCHAR）
    - 是否有索引：无→有 / 有→无
- [ ] `frontend-rs/src/editor_panels.rs`: 表头点击切换排序列/排序方向：
  - 点击表头 → 切换排序列（如当前排序列 = 表名，则切换排序方向；否则切换排序列）
  - data-testid 命名：`list-view-sort-table-name`（表名表头）、`list-view-sort-field-count`（字段数表头）等

### 测试（产出代码，不产出 delta；非 verify/smoke 节点）

> 说明：本 section 列出**代码实现同步产出**的测试用例，**非** verify/smoke/人工验证条目；按 SKILL"禁止在 tasks.md 写 verify/smoke/人工验证类条目"原则，verify/validate-ledger/openlogos-verify 等节点属独立 CLI 操作，**不**列入 tasks。

- [ ] `frontend-rs/src/editor_panels.rs` 单元测试模块（新增 UT-MM-21）：
  - happy: `sort_tables(tables, SortColumn::TableName, SortDirection::Ascending)` → 按表名字典序升序
  - happy: `sort_tables(tables, SortColumn::TableName, SortDirection::Descending)` → 按表名字典序降序
  - happy: `sort_tables(tables, SortColumn::FieldCount, SortDirection::Ascending)` → 按字段数升序（少→多）
  - happy: `sort_tables(tables, SortColumn::FieldCount, SortDirection::Descending)` → 按字段数降序（多→少）
  - happy: `sort_tables(tables, SortColumn::Type, SortDirection::Ascending)` → 按类型字典序升序（如 INT < VARCHAR）
  - happy: `sort_tables(tables, SortColumn::HasIndex, SortDirection::Ascending)` → 无索引 → 有索引
  - happy: `sort_tables(tables, SortColumn::HasIndex, SortDirection::Descending)` → 有索引 → 无索引
  - edge: `sort_tables(tables, SortColumn::TableName, SortDirection::Ascending)` 空 tables → 空结果
- [ ] `frontend-rs/src/editor_panels.rs` 单元测试模块（新增 UT-MM-22）：
  - `test_list_view_tab_switch`：ListView tab 激活时显示列表视图组件

## [spec] 规格登记（代码实现同步，非独立 delta 任务）

- [ ] 在 `logos/resources/test/core-UI-modals-2-test-cases.md`（或同类 modals 测试用例 spec 文件）追加 UT-MM-21/22 行：
  ```
  | UT-MM-21 | 列表视图排序纯函数测试（按任意列：表名/字段数/类型/是否有索引） | `editor_panels.rs::sort_tables` |
  | UT-MM-22 | 列表视图 tab 切换测试 | `editor_panels.rs::ListView` |
  ```
- [ ] 确认 UT-MM-21/22 已被 reporter 写入 `test-results.jsonl`（cargo test 触发）

## 实现顺序建议（批次 1）

1. SidePanelTab 新增 `ListView` tab（testid/label 补全）
2. `ListView` 组件基础表格化展示（表名/字段名/类型）
3. `ListViewState`（排序列/排序方向）
4. `sort_tables` 纯函数 + UT-MM-21（7 子用例）
5. 表头点击切换排序列/排序方向
6. UT-MM-22（ListView tab 切换测试）
7. spec 登记（UT-MM-21/22 行）

每步独立 commit，commit message 格式 `feat(<module>): ...`。

## 不在范围（明确排除）

- 过滤（按名称模糊匹配/按类型/按是否有索引）——批次 2
- 批量重命名（多表或多字段一次性改名）——批次 2
- 批量改类型（多字段一次性改类型）——批次 3
- 双击跳到画布对应表——批次 3
- 导出 CSV/Excel——批次 3
- 列宽可调——批次 4
- 表/字段分组（按 schema/按 tag）——批次 4
- 样式优化（字体回退栈补思源黑体/苹方 + Canvas 文本离屏缓存 + 关键交互帧率 < 16ms + rAF 统一调度）——批次 4
- 大图（>200 表）虚拟化（operator Q5 裁决暂缓）
- 不修改 `Reference`/`Table` struct 数据契约
- 不修改 reference 连线布局的端点计算算法
- 不修改 Inspector 其它字段编辑逻辑
- 不修改测试断言（外环强制约束）
- 不写 verify/smoke/archive 条目（独立 CLI 节点）