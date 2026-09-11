# 实现任务

> 配套：`proposal.md`
> 内容基线：`.octos/proposals/draft-2026-09-02-product-batch/05-feat-table-resize-tasks.md` v2（commit `db57087`）
> 外环强制补充（条目6 steer 第 3 项）：`editor_render.rs:1427` `hit_test_field` 与 `:1450` `hit_test` 也消费 `TABLE_WIDTH`，须同步改为消费 `table.width`（tasks 的 `[code]` 段加此项）

## [code] 代码实现

### 数据结构

- [ ] `frontend-rs/src/editor_core.rs`: `Table` struct 新增 `width: Option<u32>` 字段（serde 默认 `None`，向后兼容）
- [ ] `frontend-rs/src/editor_core.rs`: `Table` struct 新增 `min_height: Option<u32>` 字段（serde 默认 `None`，向后兼容）

### 纯函数

- [ ] `frontend-rs/src/editor_panels.rs`: 新增 `pub fn parse_table_height(input: &str) -> Result<u32, String>`，**严格对称 `parse_table_width`（:7473）的语义**：
  - `""` → `Err("高度不能为空")`
  - `"0"` → **`Ok(0)`（"0 = auto"，与 width 一致）**
  - `"abc"` / `"-5"` → `Err("高度必须是非负整数")`
  - `"200"` → `Ok(200)`

### UI：宽度链路闭环

- [ ] `frontend-rs/src/editor_panels.rs`: `SetTableWidthModal`（:8138+）的 `data-testid="modal-submit-set-width"` Apply 按钮**补 `on:click` handler**：
  - 读 `width_input` 当前值（已通过 `parse_table_width` 校验）
  - 找到模态传入的 `target_table_ids`（批量设宽场景下为多个，否则为单个）
  - 对每个 table，store setter 写入 `width: Some(value)`
  - 关闭模态 `kind_close.set(None)`
- [ ] `frontend-rs/src/editor_panels.rs`: 检查 `ModalKind::SetTableWidth` 是否携带 target table id(s)，如未携带需补（实现时定具体传参机制）

### UI：高度入口（实现时定）

- [ ] `frontend-rs/src/editor_panels.rs`: 新增 `SetTableMinHeightModal` 组件（或扩展现有 `SetTableWidthModal` 为 `SetTableSizeModal` 含 width + min_height）—— **实现时定**：单模态 vs 双模态需 UI 决策
- [ ] 对应 `ModalKind` 新增 `SetTableMinHeight` 变体（如独立模态方案）
- [ ] data-testid 命名：`modal-title-set-min-height` / `modal-input-min-height` / `modal-submit-set-min-height` / `modal-cancel-set-min-height-btn`

### 渲染消费

- [ ] `frontend-rs/src/editor_render.rs`: `draw_table` 函数（:1166）把硬编码 `TABLE_WIDTH = 230.0` 替换为：
  ```rust
  let width = table.width.map(|w| w as f64).unwrap_or(TABLE_WIDTH);
  ```
- [ ] `frontend-rs/src/editor_render.rs`: 高度计算改为：
  ```rust
  let auto_height = TABLE_HEADER_HEIGHT + FIELD_ROW_HEIGHT * field_count as f64;
  let total_height = table.min_height
      .map(|h| h as f64)
      .map(|min| min.max(auto_height))   // 用户最小值生效，否则自动撑高
      .unwrap_or(auto_height);
  ```
- [ ] `frontend-rs/src/editor_render.rs`: reference 端点计算（:935 处 `table.x + TABLE_WIDTH`）改为 `table.x + width`（R1 风险伴随修复）
- [ ] `frontend-rs/src/editor_render.rs`: **复用现有常量** `TABLE_WIDTH=230.0` / `TABLE_HEADER_HEIGHT=43.0` / `FIELD_ROW_HEIGHT=35.0`（:18-20），**不新建** `DEFAULT_MIN_HEIGHT` / `ROW_HEIGHT` 等常量

### 渲染：命中测试同步消费 `table.width`（外环强制补充）

- [ ] `frontend-rs/src/editor_render.rs`: `hit_test_field` 函数（:1427）将命中宽度判定 `table.x + TABLE_WIDTH` 改为消费 `table.width`，避免新宽度下命中区域错位：
  ```rust
  let width = table.width.map(|w| w as f64).unwrap_or(TABLE_WIDTH);
  if x < table.x || x > table.x + width { continue; }
  ```
  高度方向判定保持消费 `TABLE_HEADER_HEIGHT` + `FIELD_ROW_HEIGHT`（这两常量在命中测试中仍代表字段行高，与 `min_height` 无关；`min_height` 不影响字段级命中位置）
- [ ] `frontend-rs/src/editor_render.rs`: `hit_test` 函数（:1450）将表级命中判定 `table.x + TABLE_WIDTH` 改为消费 `table.width`，高度判定使用 `auto_height`（命中用 `min_height` 会让用户拖动到 `min_height` 之上仍命中，与 draw_table 渲染行为一致；保守做法是用 `auto_height`，实现时可讨论是否同步 `min_height`）

### Store 传播与 OT

- [ ] `frontend-rs/src/editor_panels.rs`: store 更新路径传播 `width` / `min_height`（参考其它字段的 setter 模式）
- [ ] `frontend-rs/src/editor_panels.rs`: OT 操作（如有）在新建/编辑表时携带 `width` / `min_height` 字段（参考其它字段在 OT op 序列化里的处理）

## [test] 测试实现（产出代码，不产出 delta；非 verify/smoke 节点）

> 说明：本 section 列出**代码实现同步产出**的测试用例，**非** verify/smoke/人工验证条目；按 SKILL"禁止在 tasks.md 写 verify/smoke/人工验证类条目"原则，verify/validate-ledger/openlogos-verify 等节点属独立 CLI 操作，**不**列入 tasks。

- [ ] `frontend-rs/tests/tokens.rs`（或新建 `tests/table_size.rs`）: 新增 **UT-MM-17**（**编号从 UT-MM-12/13/14 已被 `validate_language`/`custom_type`/`import_source` 占用的事实跳过**）
  - happy: `parse_table_height("200") → Ok(200)`
  - happy: `parse_table_height("100") → Ok(100)`
  - **edge: `parse_table_height("0") → Ok(0)`（"0 = auto"，对称 width）**
  - edge: `parse_table_height("abc") → Err(...)`
  - edge: `parse_table_height("") → Err(...)`
  - edge: `parse_table_height("-5") → Err(...)`（负数被拒绝）
- [ ] `frontend-rs/src/editor_panels.rs` 单元测试模块（如有）：
  - `test_set_table_width_modal_apply_closes_store_loop`：验证 Apply on:click 把 width 写入 store
  - `test_draw_table_uses_table_width_over_default`：验证 draw_table 读取 `table.width` 字段
  - `test_draw_table_min_height_overrides_auto`：验证 `render_height = max(min_height, auto_height)`
  - `test_draw_table_min_height_none_uses_auto`：验证 None 走 auto_height
  - `test_hit_test_field_uses_table_width`：验证 `hit_test_field` 命中宽度跟随 `table.width`（外环强制补充）
  - `test_hit_test_uses_table_width`：验证 `hit_test` 命中宽度跟随 `table.width`（外环强制补充）

## [spec] 规格登记（代码实现同步，非独立 delta 任务）

- [ ] 在 `logos/resources/test/core-CR-canvas-test-cases.md`（或同类 canvas 测试用例 spec 文件）追加 UT-MM-17 行：
  ```
  | UT-MM-17 | parse_table_height 纯函数测试（对称 parse_table_width, 0=auto） | `frontend-rs/tests/tokens.rs` |
  ```
- [ ] 确认 UT-MM-17 已被 reporter 写入 `test-results.jsonl`（cargo test 触发）

## 实现顺序建议

1. 数据结构（`editor_core.rs::Table.width` + `min_height`）+ serde 默认值
2. 纯函数 `parse_table_height` + UT-MM-17（与宽度同模板，**0=auto 对称**）
3. `SetTableWidthModal` Apply on:click 闭环 + draw_table 消费 `table.width`
4. `SetTableMinHeightModal`（或扩展 SetTableSizeModal）+ draw_table 消费 `table.min_height`
5. reference 端点计算跟随 width + **`hit_test_field` / `hit_test` 跟随 width**（R1 风险 + 外环强制）
6. OT op 携带 width / min_height（参考其它字段）
7. spec 登记（UT-MM-17 行）
8. 全量 verify（独立 CLI 节点，**非本 tasks 跟踪项**）

每步独立 commit，commit message 格式参考 `fix-auth-register-redact` 系列的 `feat(<module>): ...` 风格。

## v1 → v2 → 正式版 修订点速查

| v1 错误 | v2 修正 | 正式版（按 SKILL） |
|---|---|---|
| `Table` 已有 `pub width: Option<u32>` | `Table` 无 width 字段 | 明确为**新增**字段 + 列出 R3 风险 |
| `width` 复用既有路径 | `width` 是新链路 | 闭环 + Apply on:click 列为 `[code]` 子任务 |
| `parse_table_height("0") → Err` | `"0" → Ok(0)` | happy + edge 各列 |
| UT-MM-12/13/14 | UT-MM-17 | 同 v2 |
| 新常量 `DEFAULT_MIN_HEIGHT` | 复用 `TABLE_HEADER_HEIGHT`/`FIELD_ROW_HEIGHT` | 明确"复用" |
| 渲染在 `editor_panels.rs` | 在 `editor_render.rs:1166` | 同 v2 |
| Inspector 加高度输入 | 模态 | 同 v2 |
| **（v1/v2 漏）** | — | **`hit_test_field` / `hit_test` 同步消费 `table.width`**（外环强制补充） |
| 写"全量 verify"等条目 | 同 v2 | **删除 verify/smoke/人工验证条目**（SKILL 强制禁止）|