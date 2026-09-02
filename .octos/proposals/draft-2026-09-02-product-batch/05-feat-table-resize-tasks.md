# 实现任务 — feat-table-resize

> 状态：**草案 v2（外环条目6 切片2 判词打回修订后重写）**（落 `.octos/proposals/`，**未**走 `openlogos change`）
> 配套：`04-feat-table-resize-proposal.md`

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

### UI：高度入口（待定）

- [ ] `frontend-rs/src/editor_panels.rs`: 新增 `SetTableMinHeightModal` 组件（或扩展现有 `SetTableWidthModal` 为 `SetTableSizeModal` 含 width + min_height）—— **实现时定**：单模态 vs 双模态需 UI 决策
- [ ] 对应 `ModalKind` 新增 `SetTableMinHeight` 变体（如独立模态方案）
- [ ] 数据 testid 命名：`modal-title-set-min-height` / `modal-input-min-height` / `modal-submit-set-min-height` / `modal-cancel-set-min-height-btn`

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
- [ ] `frontend-rs/src/editor_render.rs`: reference 端点计算（:935 处 `table.x + TABLE_WIDTH`）改为 `table.x + width`（即渲染宽度的同一变量；R1 风险伴随修复）
- [ ] `frontend-rs/src/editor_render.rs`: **复用现有常量** `TABLE_WIDTH=230.0` / `TABLE_HEADER_HEIGHT=43.0` / `FIELD_ROW_HEIGHT=35.0`（:18-20），**不新建** `DEFAULT_MIN_HEIGHT` / `ROW_HEIGHT` 等常量

### Store 传播与 OT

- [ ] `frontend-rs/src/editor_panels.rs`: store 更新路径传播 `width` / `min_height`（参考其它字段的 setter 模式）
- [ ] `frontend-rs/src/editor_panels.rs`: OT 操作（如有）在新建/编辑表时携带 `width` / `min_height` 字段（参考其它字段在 OT op 序列化里的处理）

## [test] 测试

- [ ] `frontend-rs/tests/tokens.rs`（或新建 `tests/table_size.rs`）: 新增 **UT-MM-17**（**编号从 UT-MM-12/13/14 已被 `validate_language`/`custom_type`/`import_source` 占用的事实跳过**）
  - happy: `parse_table_height("200") → Ok(200)`
  - happy: `parse_table_height("100") → Ok(100)`
  - **edge: `parse_table_height("0") → Ok(0)`（"0 = auto"，对称 width）**
  - edge: `parse_table_height("abc") → Err(...)`
  - edge: `parse_table_height("") → Err(...)`
  - edge: `parse_table_height("-5") → Err(...)`（负数被拒绝）
- [ ] `frontend-rs/src/editor_panels.rs` 单元测试模块（如有）：
  - `test_set_table_width_modal_apply_closes_store_loop`：验证 Apply on:click 把 width 写入 store（可 mock store 验证调用）
  - `test_draw_table_uses_table_width_over_default`：验证 draw_table 读取 `table.width` 字段
  - `test_draw_table_min_height_overrides_auto`：验证 `render_height = max(min_height, auto_height)`
  - `test_draw_table_min_height_none_uses_auto`：验证 None 走 auto_height
- [ ] 视觉回归：通过现有 Playwright spec-parity-* 跑套，验证未破坏既有视觉断言（ST-FE-ALIGN-*、ST-PU-*、ST-CR-*）

## [verify] 验收

- [ ] `cd frontend-rs && cargo test --lib`：全绿（含 UT-MM-17 新增 6 个用例 + draw_table 新增 4 个用例）
- [ ] `cd backend && cargo test`：全绿（确认后端零改动）
- [ ] `cd mcp-server && cargo test`：全绿
- [ ] `cd frontend-rs && npm run test:spec-parity-a` / `-b` / `-c` / `-d`：全绿
- [ ] `cd frontend-rs && npm run test:unified-prototype`：全绿
- [ ] `bash scripts/run-verify-tests-clean.sh` 完整跑通
- [ ] `node scripts/validate-openlogos-ledger.mjs --report ST-MM-17` PASS（UT-MM-17 在 ledger 登记）
- [ ] `openlogos verify` Gate 3.5 PASS（包含新增 UT-MM-17 与既有所有用例）

## [spec] 规格登记

- [ ] 在 `logos/resources/test/core-CR-canvas-test-cases.md`（或同类 canvas 测试用例 spec 文件）追加 UT-MM-17 行：
  ```
  | UT-MM-17 | parse_table_height 纯函数测试（对称 parse_table_width, 0=auto） | `frontend-rs/tests/tokens.rs` |
  ```
- [ ] 确认 UT-MM-17 已被 reporter 写入 `test-results.jsonl`（cargo test 触发）

## [archive] 归档

- [ ] `openlogos archive feat-table-resize`（待 verify Gate 3.5 PASS + 外环独立复验后；operator 授权后执行）

## 完成记录

> 实现完成后填入 commit hash 与验证证据。

---

## 实现顺序建议

1. 数据结构（`editor_core.rs::Table.width` + `min_height`）+ serde 默认值
2. 纯函数 `parse_table_height` + UT-MM-17（与宽度同模板，**0=auto 对称**）
3. `SetTableWidthModal` Apply on:click 闭环 + draw_table 消费 `table.width`
4. `SetTableMinHeightModal`（或扩展 SetTableSizeModal） + draw_table 消费 `table.min_height`
5. reference 端点计算跟随 width（R1 风险伴随修复）
6. OT op 携带 width / min_height（参考其它字段）
7. spec 登记（UT-MM-17 行）
8. 全量 verify

每步独立 commit，commit message 格式参考 `fix-auth-register-redact` 系列的 `feat(<module>): ...` 风格。

---

## v1 → v2 修订点速查（外环 R2 反馈）

| v1 错误 | v2 修正 |
|---|---|
| `Table` 已有 `pub width: Option<u32>`（既有） | `Table` struct **无 width 字段**，本次为新增 |
| `width` 复用既有路径 | `width` 是**新链路**：补 Apply on:click + draw_table 消费 |
| `parse_table_height("0") → Err` | `parse_table_height("0") → Ok(0)`（**对称 width 0=auto**）|
| UT-MM-12/13/14 编号 | **UT-MM-17**（前序编号已被 `validate_language`/`custom_type`/`import_source` 占用）|
| 渲染消费新常量 `DEFAULT_MIN_HEIGHT` / `ROW_HEIGHT` | **复用** `TABLE_HEADER_HEIGHT=43.0` / `FIELD_ROW_HEIGHT=35.0`（`editor_render.rs:18-20`），不新建常量 |
| 渲染逻辑在 `editor_panels.rs` | 渲染逻辑在 `editor_render.rs:1166 draw_table`，所有渲染改动落此文件 |
| Inspector 加高度输入 | 模态（SetTableMinHeight 或扩展 SetTableSizeModal），非 Inspector |