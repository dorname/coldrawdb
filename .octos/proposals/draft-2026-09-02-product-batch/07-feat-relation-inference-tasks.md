# 实现任务 — feat-relation-inference

> 状态：**草案**（落 `.octos/proposals/`，**未**走 `openlogos change`）
> 配套：`06-feat-relation-inference-proposal.md`
> 外环强制约束：禁止改测试断言；tasks 不写 verify/smoke/archive 条目（独立 CLI 节点）；新 UT/ST 编号先 grep 占用情况取下一空闲（UT-MM-18）

## [code] 代码实现

### 确认条状态机（去掉 cardinality 必选，改为推导 + 手动覆盖）

- [ ] `frontend-rs/src/editor_panels.rs`: `RelToolState::Confirm:370-389` 的 `cardinality: String` 字段改为**推导值**（非必选下拉值）：
  - 新增 `infer_cardinality(start_table_id: &str, end_table_id: &str, store: &EditorStore) -> String` 纯函数：
    - 推导规则（operator Q2 裁决）：1+1→"one_to_one"、1+N→"one_to_many"、N+N→"many_to_many"（N = 两端字段数，字段按用户点击顺序）
    - 字段数从 `store.tables.get()` 的对应 table 取 `fields.len()`
    - **向后兼容**：如 table 不存在或字段数为 0，fallback 到 "one_to_many"（与现有默认一致）
  - `RelToolState::Confirm` 状态机构造时调用 `infer_cardinality` 填充 `cardinality` 字段（替代用户必选下拉）
- [ ] `frontend-rs/src/editor_panels.rs`: `build_reference:411-434` 的 `cardinality: &str` 参数改为**推导值**（非用户必选下拉值）
- [ ] `frontend-rs/src/editor_panels.rs`: 确认条 UI 去掉 `CARDINALITY_OPTIONS:411` 4 选 1 必选下拉，改为显示推导结果（如"1:N"）+ 可点击切换为其它 cardinality（手动覆盖）：
  - data-testid 命名：`rel-confirm-inferred-cardinality`（推导结果显示）+ `rel-confirm-cardinality-override`（手动覆盖按钮）
  - 手动覆盖按钮点击后弹出 4 选 1 下拉（与现有 Inspector reference 面板一致）
- [ ] `frontend-rs/src/editor_panels.rs`: `flip_reference_endpoints:439` 翻转后**重新推导 cardinality**（基于翻转后的两端字段数）：
  - 翻转后 `start_field_id`/`end_field_id` 互换，两端字段数可能变化（如 1+N → N+1）
  - 重新推导：`infer_cardinality(flip.end_table_id, flip.start_table_id, store)`（注意翻转后 start/end 互换）
  - 更新 `flip.type_` 字段为重新推导结果

### Inspector reference 面板（保留手动覆盖）

- [ ] `frontend-rs/src/editor_panels.rs`: Inspector reference 面板保留 cardinality 编辑器（允许手动覆盖推导结果）：
  - 现有 `inspector-ref-cardinality` 下拉保留
  - 手动覆盖后 `reference.type_` 字段更新为用户选择值
  - 推导结果与手动覆盖值**不区分**（`type_` 字段统一存储，向后兼容）

### 测试（产出代码，不产出 delta；非 verify/smoke 节点）

> 说明：本 section 列出**代码实现同步产出**的测试用例，**非** verify/smoke/人工验证条目；按 SKILL"禁止在 tasks.md 写 verify/smoke/人工验证类条目"原则，verify/validate-ledger/openlogos-verify 等节点属独立 CLI 操作，**不**列入 tasks。

- [ ] `frontend-rs/src/editor_panels.rs` 单元测试模块（新增 UT-MM-18）：
  - happy: `infer_cardinality("t1", "t2", store)` 两端各 1 字段 → "one_to_one"
  - happy: `infer_cardinality("t1", "t2", store)` 端 A 1 字段 + 端 B N 字段 → "one_to_many"
  - happy: `infer_cardinality("t1", "t2", store)` 端 A N 字段 + 端 B N 字段 → "many_to_many"
  - happy: `infer_cardinality("t1", "t2", store)` 端 A N 字段 + 端 B 1 字段 → "many_to_one"（字段按用户点击顺序，start/end 互换后推导结果不同）
  - edge: `infer_cardinality("t1", "t2", store)` table 不存在 → fallback "one_to_many"
  - edge: `infer_cardinality("t1", "t2", store)` 字段数为 0 → fallback "one_to_many"
- [ ] `frontend-rs/src/editor_panels.rs` 单元测试模块（新增 UT-MM-19）：
  - `test_flip_reference_endpoints_re_infers_cardinality`：翻转后重新推导 cardinality（如 1+N → N+1，推导结果从 "one_to_many" 变为 "many_to_one"）
- [ ] `frontend-rs/src/editor_panels.rs` 单元测试模块（新增 UT-MM-20）：
  - `test_build_reference_uses_inferred_cardinality`：`build_reference` 使用推导值而非用户必选下拉值

## [spec] 规格登记（代码实现同步，非独立 delta 任务）

- [ ] 在 `logos/resources/test/core-UI-modals-2-test-cases.md`（或同类 modals 测试用例 spec 文件）追加 UT-MM-18/19/20 行：
  ```
  | UT-MM-18 | cardinality 推导纯函数测试（1+1→1:1, 1+N→1:N, N+N→N:N） | `editor_panels.rs::infer_cardinality` |
  | UT-MM-19 | flip_reference_endpoints 翻转后重新推导 cardinality | `editor_panels.rs::flip_reference_endpoints` |
  | UT-MM-20 | build_reference 使用推导值而非用户必选下拉值 | `editor_panels.rs::build_reference` |
  ```
- [ ] 确认 UT-MM-18/19/20 已被 reporter 写入 `test-results.jsonl`（cargo test 触发）

## 实现顺序建议

1. `infer_cardinality` 纯函数 + UT-MM-18（6 子用例）
2. `RelToolState::Confirm` 状态机改为推导值（去掉 cardinality 必选下拉）
3. 确认条 UI 显示推导结果 + 手动覆盖按钮
4. `build_reference` 使用推导值 + UT-MM-20
5. `flip_reference_endpoints` 翻转后重新推导 + UT-MM-19
6. Inspector reference 面板保留 cardinality 编辑器（手动覆盖）
7. spec 登记（UT-MM-18/19/20 行）

每步独立 commit，commit message 格式 `feat(<module>): ...`。

## 不在范围（明确排除）

- 不支持多字段连接（`start_field_ids`/`end_field_ids` 数组）——需 operator 裁决后另立提案
- 不修改 `Reference.type_` 字段类型（保持 `String`）
- 不修改 reference 连线布局的端点计算算法
- 不修改 Inspector 其它字段编辑逻辑
- 不修改测试断言（外环强制约束）
- 不写 verify/smoke/archive 条目（独立 CLI 节点）