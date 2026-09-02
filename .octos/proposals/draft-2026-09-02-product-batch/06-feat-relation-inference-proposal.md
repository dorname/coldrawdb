# 变更提案：feat-relation-inference

> module: core | created: 2026-09-02
> 状态：**草案 v2（外环条目6 判词打回修订后重写）**（落 `.octos/proposals/`，**未**走 `openlogos change`）
> 父批次：[产品优化批次-2026-09-02](./01-current-state.md) D 案（需求 2 独立）
> 上游裁决：黑板条目6 operator 批注（Q2=允许手动覆盖 + 字段按用户点击顺序；推导规则 1+1→1:1、1+N→1:N、N+N→N:N）
> 上游判词：条目6 尾部外环(claude) 主审评审 2026-09-03 — 语义层错误打回修订 v2

## 变更原因

**UX 缺口**（operator 原始需求 2）：连接字段时强制选择 cardinality（4 选 1 下拉：one_to_one / one_to_many / many_to_one / many_to_many），用户期望"连接时不要求选择 1:1/1:N，连接多个字段自然推导为 1:N 或 N:N"。

**现状事实层**（v1 打回教训：行号引用必须实测）：

- `frontend-rs/src/editor_panels.rs:411` `CARDINALITY_OPTIONS: &[&str] = &["one_to_one", "one_to_many", "many_to_one", "many_to_many"]` —— 4 选 1 必选下拉
- `frontend-rs/src/editor_panels.rs:370-389` `RelToolState::Confirm { start_table_id, start_field_id, end_table_id, end_field_id, cardinality: String }` —— 确认条状态机含 cardinality 字段，连接时必选
- `frontend-rs/src/editor_panels.rs:411-434` `build_reference(id, start_table_id, start_field_id, end_table_id, end_field_id, cardinality: &str)` —— 创建 Reference 时传入 cardinality
- `frontend-rs/src/editor_panels.rs:439` `flip_reference_endpoints(r: &Reference)` —— 翻转端点函数存在
- `frontend-rs/scripts/test-spec-parity-d.mjs:525-549` ST-PB-01 测试"点击两点 + 确认条创建关系"；`:551+` ST-PB-02 测试"拖线（≥4px + rubber-band）+ 确认条创建关系"——现有交互为"点击两点"或"拖线"触发确认条，确认条含 cardinality 下拉（必选）
- `frontend-rs/src/editor_core.rs:77-88` `Reference` struct 有 `type_: String` 字段（cardinality 落库字段）+ `start_field_id`/`end_field_id` 单字段（非数组）
- 现有 relation 创建流**仅支持单字段对单字段**（`RelToolState::Confirm` 的 `start_field_id`/`end_field_id` 是单字段）

**operator 裁决（Q2）**：**允许手动覆盖 + 字段按用户点击顺序**。推导规则：**1+1→1:1、1+N→1:N、N+N→N:N**（字段按用户点击顺序排列）。

**外环判词修正（v2 语义层错误）**：草案 v1 误把 Q2 的 1/N 理解为**两端表的总字段数**（`fields.len()`）——这与关系基数**无语义关联**。反例：20 字段表与 3 字段表之间连**一条**单字段关系，草案会推导出 N+N→many_to_many，但用户只连了一对字段，应为 one_to_one。

**Q2 正确语义（外环裁定）**：「1+1→1:1、1+N→1:N、N+N→N:N」中的 1/N 指**该字段已参与的关系计数**（含本次新建），不是表总字段数。推导函数应改为 `infer_cardinality(start_field_id, end_field_id, store)`：
- s = start_field 已参与的关系数（含本条），e = end_field 已参与的关系数（含本条）
- s==1 && e==1 → `one_to_one`；s==1 && e>1 → `one_to_many`；s>1 && e==1 → `many_to_one`；s>1 && e>1 → `many_to_many`
- 「字段按用户点击顺序」落实为先点者为 start（方向由此决定）
- 例：用户先连 A.a→B.x，再连 A.a→B.y——第二条创建时 A.a 参与 2 条（s=2）、B.y 参与 1 条（e=1）→ many_to_one……注意方向：A 端一个字段对 B 端多个字段，语义是 A:B=1:N，即 start=A 时应得 one_to_many。实现时注意 s/e 的计数语义与 start 端为"一"侧的对应关系。

**数据契约**（外环代决）：**不扩展 Reference 数据契约**（不加 `start_field_ids`/`end_field_ids` 数组，复合外键另立案）。operator 需求 2「连接多个字段自然推导」在上述语义下已完整覆盖——多条单字段关系在同一字段上累计即自然形成 1:N/N:N，无需改契约。

## 变更类型

**代码级修复**（参考 `spec/tasks-spec.md` 与 `logos/skills/change-writer/SKILL.md` Step 3 判定）：
- 影响的 PRD/API/DB schema：**无**（`Reference.type_` 字段保持 `String`，向后兼容）
- 影响的功能规格：**无**（cardinality 推导规则不是需求级事实；4 选 1 下拉属实现侧发明）
- 影响的部署方案：**无**（纯前端 WASM）
- 影响的 smoke：**无**

故 `tasks.md` 采用**代码级修复模板**（无 `[delta]`、`[deploy]` section）。

## 变更范围

- 影响的需求文档：**无**（cardinality 推导规则不是需求级事实）
- 影响的功能规格：**无**（grep 既有规格文档无 cardinality 推导字面量断言）
- 影响的业务场景：
  - S01（编辑并保存 diagram）：cardinality 推导后落 store → 保存链路需携带（已是 JSON blob 全量保存，自动适配）
  - S05（OT 实时协作）：cardinality 推导规则变更后，OT 操作需包含推导结果——实现时验证 op 应用器对 `type_` 字段的处理
- 影响的部署方案：**无**
- 影响的 API：**无**（`PUT /api/v1/diagrams/{id}` 契约不变，JSON blob 内字段增减对后端透明）
- 影响的 DB 表：**无 schema 变更**（`reference` 表 `type_` 字段保持 `String`，向后兼容；存量数据原 cardinality 字段保留）
- 影响的编排测试：场景 S01/S05 的下游测试需验证新推导规则不破坏断言
- 影响的 smoke 测试：**无**

**代码影响面**（`frontend-rs/`）：
- `src/editor_panels.rs`：
  - `CARDINALITY_OPTIONS:411` —— 4 选 1 必选下拉（**去除**）
  - `RelToolState::Confirm:370-389` —— 确认条状态机含 `cardinality: String`（**改为推导值**，允许用户手动覆盖）
  - `build_reference:411-434` —— 创建 Reference 时传入 cardinality（**改为推导值**）
  - `flip_reference_endpoints:439` —— 翻转端点函数（**推导后翻转需重新推导 cardinality**，基于翻转后的两端字段已参与关系计数）
  - 关系创建交互流程：点击两点 / 拖线 → 确认条（**去掉 cardinality 下拉**，改为显示推导结果 + 允许手动覆盖）
  - Inspector reference 面板：cardinality 编辑器（**保留**，允许手动覆盖推导结果）
- `src/editor_core.rs`：`Reference` struct `type_: String` 字段保持 `String`（向后兼容）；**不扩展** `start_field_ids`/`end_field_ids` 数组（外环代决否决，复合外键另立案）

## 部署影响

- 是否需要部署：**否**
- 部署原因：纯前端 WASM 代码修复，本地开发环境重新构建即生效；当前项目处于开发阶段，无独立部署节点
- 影响环境：**无**
- 是否涉及数据迁移：**否**（存量 `reference.type_` 字段保留 `String`，向后兼容；新推导规则只影响新建 relation）
- 是否需要回滚预案：**否**（小切片，回滚 = revert commit）
- 是否需要 smoke：**否**

## 变更概述

去掉关系创建确认条的 cardinality 必选下拉，改为**自动推导 + 允许手动覆盖**：
- 推导规则（operator Q2 裁决 + 外环判词修正）：**1/N 指该字段已参与的关系计数（含本次新建）**，不是表总字段数
  - `infer_cardinality(start_field_id, end_field_id, store)`：s = start_field 已参与的关系数（含本条），e = end_field 已参与的关系数（含本条）
  - s==1 && e==1 → `one_to_one`；s==1 && e>1 → `one_to_many`；s>1 && e==1 → `many_to_one`；s>1 && e>1 → `many_to_many`
  - **方向**：start 端为"一"侧时 one_to_many（start 是 1，end 是 N）
- 确认条显示推导结果（如"1:N"），用户可点击切换为其它 cardinality（手动覆盖）
- Inspector reference 面板保留 cardinality 编辑器（允许手动覆盖推导结果）
- `flip_reference_endpoints` 翻转后重新推导 cardinality（基于翻转后的两端字段已参与关系计数，s/e 互换）
- **不扩展** `Reference` struct 数据契约（不加 `start_field_ids`/`end_field_ids` 数组，外环代决否决，复合外键另立案）

## 设计决策记录（ADR-style 摘要）

| 决策 | 选 | 否 | 依据 |
|---|---|---|---|
| 推导依据 | 字段已参与关系计数（含本次新建）| 表总字段数 `fields.len()` | 外环判词修正（v1 语义层错误） |
| 推导规则 | s==1 && e==1 → one_to_one；s==1 && e>1 → one_to_many；s>1 && e==1 → many_to_one；s>1 && e>1 → many_to_many | 固定 one_to_many 默认 / 完全自动 | operator Q2 裁决 + 外环判词修正 |
| 字段顺序 | 按用户点击顺序（先点者为 start） | 按 schema 顺序 | operator Q2 裁决 |
| 手动覆盖 | 允许（Inspector 保留编辑器）| 完全自动不可改 | operator Q2 裁决 |
| 确认条 UI | 显示推导结果 + 可点击切换 | 去掉确认条直接创建 | 保留确认条可减少误操作（现有 ST-PB-01/02 测试覆盖确认条交互）|
| 数据结构 | `type_: String` 保持 | 改为 Enum / 加 `auto_inferred: bool` / 加 `start_field_ids`/`end_field_ids` 数组 | 向后兼容 + 最小改动 + 外环代决否决（复合外键另立案） |

## 真值表（外环判词要求）

| start_field 已参与关系数（s，含本条） | end_field 已参与关系数（e，含本条） | 推导结果 | 语义 |
|---|---|---|---|
| 1 | 1 | `one_to_one` | start:end = 1:1 |
| 1 | N (N>1) | `one_to_many` | start:end = 1:N（start 端为"一"侧） |
| N (N>1) | 1 | `many_to_one` | start:end = N:1（end 端为"一"侧） |
| N (N>1) | N (N>1) | `many_to_many` | start:end = N:N |

**方向对应关系**：start 端为"一"侧时 `one_to_many`（start 是 1，end 是 N）；end 端为"一"侧时 `many_to_one`（start 是 N，end 是 1）。

## 范围外（明确排除）

- 不支持多字段连接（`start_field_ids: Vec<String>`/`end_field_ids: Vec<String>`）——外环代决否决（复合外键另立案）；operator 需求 2「连接多个字段自然推导」在字段已参与关系计数语义下已完整覆盖——多条单字段关系在同一字段上累计即自然形成 1:N/N:N，无需改契约
- 不修改 `Reference.type_` 字段类型（保持 `String`，向后兼容）
- 不修改 reference 连线布局的端点计算算法（仅跟随 cardinality 推导结果；新算法属 `feat-relation-inference` 后续扩展）
- 不修改 Inspector 其它字段编辑逻辑
- 不修改测试断言（外环强制约束）

## 风险点

- **R1**：推导规则与老数据兼容性——存量 `reference.type_` 字段保持 `String`，新推导规则只影响新建 relation，老数据原 cardinality 字段保留（向后兼容）
- **R2**：`flip_reference_endpoints` 翻转后重新推导 cardinality——翻转后 start/end 互换，s/e 互换，推导结果可能不同（如 s=1 && e=2 时 one_to_many → 翻转后 s=2 && e=1 时 many_to_one），需验证语义正确性
- **R3**：确认条去掉 cardinality 下拉后，ST-PB-01/02 测试需更新（现有测试断言确认条含 cardinality 下拉）——**外环强制约束：禁止改测试断言**，需新增测试覆盖推导逻辑
- **R4**：与 feat-table-resize 的端点计算可能耦合（reference 端点位置计算依赖 `table.width`/`table.min_height`，D 案在 A 案之后执行）

## 替代方案否决理由

- **A 固定 one_to_many 默认**：所有新建 relation 默认 1:N，用户手动改——否决（operator Q2 要求推导规则 1+1→1:1 等）
- **B 完全自动不可改**：去掉 cardinality 下拉且不允许手动覆盖——否决（operator Q2 要求允许手动覆盖）
- **C 去掉确认条直接创建**：点击两点/拖线后直接创建 relation 无确认条——否决（保留确认条可减少误操作，现有 ST-PB-01/02 测试覆盖确认条交互）
- **D `type_` 改为 Enum**：破坏向后兼容（存量 `type_` 字段是 `String`，Enum 需 schema migration）——否决
- **E 加 `auto_inferred: bool` 字段**：标记推导值 vs 手动覆盖值——否决（最小改动原则，`type_` 字段已足够，无需额外标记）
- **F 加 `start_field_ids`/`end_field_ids` 数组**：扩展 Reference 数据契约支持多字段连接——否决（外环代决，复合外键另立案；字段已参与关系计数语义下已完整覆盖）

## 关联场景

- **S01（编辑并保存图表）**：cardinality 推导后落 store → 保存链路（已是 JSON blob 全量保存，自动适配）
- **S05（OT 实时协作）**：cardinality 推导规则变更后，OT 操作需包含推导结果——实现时验证 op 应用器对 `type_` 字段的处理（`CommandStack::apply` 直接接收完整 `Reference` 对象，随 Reference struct 序列化自动携带）

## 关联任务清单

见 `07-feat-relation-inference-tasks.md`。

## 不在范围（明确排除）

- 不支持多字段连接（`start_field_ids`/`end_field_ids` 数组）——外环代决否决（复合外键另立案）
- 不修改 `Reference.type_` 字段类型（保持 `String`）
- 不修改 reference 连线布局的端点计算算法
- 不修改 Inspector 其它字段编辑逻辑
- 不修改测试断言（外环强制约束）