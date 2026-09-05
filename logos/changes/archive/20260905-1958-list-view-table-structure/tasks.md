# 实现任务：list-view-table-structure

> 配套：`logos/changes/list-view-table-structure/proposal.md`
> 上游：用户反馈问题 1（2026-09-04）→ p0-fix「不在范围」转出
> 外环强制约束：禁止改测试断言；新 UT/ST 编号先 grep 取下一空闲（UT-MM 占用至 34；ST 新增 ST-LV-01）；tasks 不写 verify/smoke/archive 条目（独立 CLI 节点）

## [code] GroupByMode::ByTable 纯函数

- [x] `frontend-rs/src/editor_panels.rs`: `GroupByMode` 新增 `ByTable` 变体
- [x] `frontend-rs/src/editor_panels.rs`: `group_tables` 加 `ByTable` 分支：按 Table 分桶（桶键 = 表名；**空表也要出桶**（fields 为空 vec）；桶顺序 = tables 数组顺序）
- [x] UT-MM-35：`ByTable` 分桶纯函数测试（两表多字段 / 空表出桶 / 桶序保持）

## [code] ListView 分组 UI

- [x] `frontend-rs/src/editor_panels.rs`: 分组下拉（`list-view-group-by`）新增「按表分组」选项
- [x] `frontend-rs/src/editor_panels.rs`: `ByTable` 桶头行渲染：表名 + 字段数（沿用 ByTag 桶头行结构，列数对齐既有 5 列修正）
- [x] `frontend-rs/src/editor_panels.rs`: 桶头行 on:click → `on_select_table(Some(table_id))`（选中表 + Inspector 同步）
- [x] `frontend-rs/src/editor_panels.rs`: `ListViewState.group_by` 默认值改为 `ByTable`

## 测试

- [x] UT-MM-35 → `core-UI-modals-2-test-cases.md` 登记 + reporter `UT_PASS_IDS` 落账
- [x] ST-LV-01 → `test-spec-parity-d.mjs`：打开 ListView → 默认按表分组（两表各出桶头行 + 字段行数正确）→ 点桶头选中表（Inspector 打开）→ 切「不分组」回归
- [x] ST-LV-01 → `core-UI-modals-2-test-cases.md` 登记 + jsonl 落账

## 不在范围

- 列表入口强化（快捷键 L / 浮动返回按钮）
- 桶内字段行内联编辑 / 桶折叠展开
- 问题 2（图级 database 字段 + 方言切换）
- 不修改既有测试断言（外环强制约束）
- 不写 verify/smoke/archive 条目（独立 CLI 节点）
