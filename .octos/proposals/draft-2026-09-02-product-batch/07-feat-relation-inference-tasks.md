# 实现任务 — feat-relation-inference

> 状态：**草案 v2（外环条目6 判词打回修订后重写）**（落 `.octos/proposals/`，**未**走 `openlogos change`）
> 配套：`06-feat-relation-inference-proposal.md` v2
> 外环强制约束：禁止改测试断言；tasks 不写 verify/smoke/archive 条目（独立 CLI 节点）；新 UT/ST 编号先 grep 占用情况取下一空闲（UT-MM-18/19/20）
> 外环判词修正（v2 语义层错误）：推导依据从「表总字段数 `fields.len()`」改为「字段已参与关系计数（含本次新建）」——Q2 的 1/N 指字段参与的关系数，非表字段数

## [code] 代码实现

### 确认条状态机（去掉 cardinality 必选，改为推导 + 手动覆盖）

- [ ] `frontend-rs/src/editor_panels.rs`: `RelToolState::Confirm:370-389` 的 `cardinality: String` 字段改为**推导值**（非必选下拉值）：
  - 新增 `infer_cardinality(start_field_id: &str, end_field_id: &str, store: &EditorStore) -> String` 纯函数：
    - **推导依据（外环判词修正）**：**字段已参与关系计数（含本次新建）**，不是表总字段数 `fields.len()`
    - s = start_field 已参与的关系数（含本条），e = end_field 已参与的关系数（含本条）
    - 从 `store.references.get()` 统计 `start_field_id`/`end_field_id` 出现的次数（作为 start 或 end 端均可）
    - 推导规则（operator Q2 裁决 + 外环判词修正）：
      - s==1 && e==1 → `"one_to_one"`
      - s==1 && e>1 → `"one_to_many"`（start 端为"一"侧）
      - s>1 && e==1 → `"many_to_one"`（end 端为"一"侧）
      - s>1 && e>1 → `"many_to_many"`
    - **向后兼容**：如字段不存在或计数为 0，fallback 到 `"one_to_many"`（与现有默认一致）
  - `RelToolState::Confirm` 状态机构造时调用 `infer_cardinality` 填充 `cardinality` 字段（替代用户必选下拉）
- [ ] `frontend-rs/src/editor_panels.rs`: `build_reference:411-434` 的 `cardinality: &str` 参数改为**推导值**（非用户必选下拉值）
- [ ] `frontend-rs/src/editor_panels.rs`: 确认条 UI 去掉 `CARDINALITY_OPTIONS:411` 4 选 1 必选下拉，改为显示推导结果（如"1:N"）+ 可点击切换为其它 cardinality（手动覆盖）：
  - data-testid 命名：`rel-confirm-inferred-cardinality`（推导结果显示）+ `rel-confirm-cardinality-override`（手动覆盖按钮）
  - 手动覆盖按钮点击后弹出 4 选 1 下拉（与现有 Inspector reference 面板一致）
- [ ] `frontend-rs/src/editor_panels.rs`: `flip_reference_endpoints:439` 翻转后**重新推导 cardinality**（基于翻转后的两端字段已参与关系计数）：
  - 翻转后 `start_field_id`/`end_field_id` 互换，s/e 互换
  - 重新推导：`infer_cardinality(flip.end_field_id, flip.start_field_id, store)`（注意翻转后 start/end 互换）
  - 更新 `flip.type_` 字段为重新推导结果
  - **真值表验证**：如翻转前 s=1 && e=2（one_to_many）→ 翻转后 s=2 && e=1（many_to_one）

### Inspector reference 面板（保留手动覆盖）

- [ ] `frontend-rs/src/editor_panels.rs`: Inspector reference 面板保留 cardinality 编辑器（允许手动覆盖推导结果）：
  - 现有 `inspector-ref-cardinality` 下拉保留
  - 手动覆盖后 `reference.type_` 字段更新为用户选择值
  - 推导结果与手动覆盖值**不区分**（`type_` 字段统一存储，向后兼容）

### 测试（产出代码，不产出 delta；非 verify/smoke 节点）

> 说明：本 section 列出**代码实现同步产出**的测试用例，**非** verify/smoke/人工验证条目；按 SKILL"禁止在 tasks.md 写 verify/smoke/人工验证类条目"原则，verify/validate-ledger/openlogos-verify 等节点属独立 CLI 操作，**不**列入 tasks。

- [ ] `frontend-rs/src/editor_panels.rs` 单元测试模块（新增 UT-MM-18）：
  - **推导依据**：字段已参与关系计数（含本次新建），不是表总字段数
  - happy: `infer_cardinality("f1", "f2", store)` 两端字段均参与 0 条既有关系 → `"one_to_one"`（s=1, e=1）
  - happy: `infer_cardinality("f1", "f2", store)` start 字段参与 0 条既有关系 + end 字段参与 1 条既有关系 → `"one_to_many"`（s=1, e=2）
  - happy: `infer_cardinality("f1", "f2", store)` start 字段参与 1 条既有关系 + end 字段参与 0 条既有关系 → `"many_to_one"`（s=2, e=1）
  - happy: `infer_cardinality("f1", "f2", store)` start 字段参与 1 条既有关系 + end 字段参与 1 条既有关系 → `"many_to_many"`（s=2, e=2）
  - happy: `infer_cardinality("f1", "f2", store)` start 字段参与 2 条既有关系 + end 字段参与 0 条既有关系 → `"many_to_one"`（s=3, e=1）
  - edge: `infer_cardinality("f1", "f2", store)` 字段不存在 → fallback `"one_to_many"`
  - edge: `infer_cardinality("f1", "f2", store)` 字段计数为 0 → fallback `"one_to_many"`
- [ ] `frontend-rs/src/editor_panels.rs` 单元测试模块（新增 UT-MM-19）：
  - `test_flip_reference_endpoints_re_infers_cardinality`：翻转后重新推导 cardinality（如翻转前 s=1 && e=2（one_to_many）→ 翻转后 s=2 && e=1（many_to_one））
- [ ] `frontend-rs/src/editor_panels.rs` 单元测试模块（新增 UT-MM-20）：
  - `test_build_reference_uses_inferred_cardinality`：`build_reference` 使用推导值而非用户必选下拉值

## [spec] 规格登记（代码实现同步，非独立 delta 任务）

- [ ] 在 `logos/resources/test/core-UI-modals-2-test-cases.md`（或同类 modals 测试用例 spec 文件）追加 UT-MM-18/19/20 行：
  ```
  | UT-MM-18 | cardinality 推导纯函数测试（字段已参与关系计数：s==1&&e==1→1:1, s==1&&e>1→1:N, s>1&&e==1→N:1, s>1&&e>1→N:N） | `editor_panels.rs::infer_cardinality` |
  | UT-MM-19 | flip_reference_endpoints 翻转后重新推导 cardinality（s/e 互换） | `editor_panels.rs::flip_reference_endpoints` |
  | UT-MM-20 | build_reference 使用推导值而非用户必选下拉值 | `editor_panels.rs::build_reference` |
  ```
- [ ] 确认 UT-MM-18/19/20 已被 reporter 写入 `test-results.jsonl`（cargo test 触发）

## 实现顺序建议

1. `infer_cardinality` 纯函数 + UT-MM-18（7 子用例：字段已参与关系计数语义）
2. `RelToolState::Confirm` 状态机改为推导值（去掉 cardinality 必选下拉）
3. 确认条 UI 显示推导结果 + 手动覆盖按钮
4. `build_reference` 使用推导值 + UT-MM-20
5. `flip_reference_endpoints` 翻转后重新推导 + UT-MM-19（s/e 互换）
6. Inspector reference 面板保留 cardinality 编辑器（手动覆盖）
7. spec 登记（UT-MM-18/19/20 行）

每步独立 commit，commit message 格式 `feat(<module>): ...`。

## 不在范围（明确排除）

- 不支持多字段连接（`start_field_ids`/`end_field_ids` 数组）——外环代决否决（复合外键另立案）；operator 需求 2「连接多个字段自然推导」在字段已参与关系计数语义下已完整覆盖——多条单字段关系在同一字段上累计即自然形成 1:N/N:N，无需改契约
- 不修改 `Reference.type_` 字段类型（保持 `String`）
- 不修改 reference 连线布局的端点计算算法
- 不修改 Inspector 其它字段编辑逻辑
- 不修改测试断言（外环强制约束）
- 不写 verify/smoke/archive 条目（独立 CLI 节点）

## v1 → v2 修订点速查（外环判词反馈）

| v1 错误 | v2 修正 |
|---|---|
| `infer_cardinality(start_table_id, end_table_id, store)` 以**两端表的总字段数**（`fields.len()`）作为推导依据 | `infer_cardinality(start_field_id, end_field_id, store)` 以**该字段已参与的关系计数（含本次新建）**作为推导依据 |
| 推导规则：1+1→1:1、1+N→1:N、N+N→N:N（N = 表总字段数） | 推导规则：s==1&&e==1→1:1, s==1&&e>1→1:N, s>1&&e==1→N:1, s>1&&e>1→N:N（s/e = 字段已参与关系计数） |
| 反例：20 字段表与 3 字段表之间连一条单字段关系 → 推导出 many_to_many | 反例修正：20 字段表与 3 字段表之间连一条单字段关系 → s==1 && e==1 → one_to_one（字段已参与关系计数为 1） |
| `flip_reference_endpoints` 翻转后重新推导（基于翻转后的两端字段数） | `flip_reference_endpoints` 翻转后重新推导（基于翻转后的两端字段已参与关系计数，s/e 互换） |
| UT-MM-18 六子用例（表总字段数语义） | UT-MM-18 七子用例（字段已参与关系计数语义） |
| 真值表未给出 | 真值表给出（s/e 与 one_to_many 方向对应关系：start 端为"一"侧时 one_to_many） |
| 多字段契约扩展待定 | 多字段契约扩展外环代决否决（复合外键另立案），范围外排除保留 |