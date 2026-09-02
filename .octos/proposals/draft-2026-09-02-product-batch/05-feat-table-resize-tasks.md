# 实现任务 — feat-table-resize

> 状态：**草案**（落 `.octos/proposals/`，**未**走 `openlogos change`）
> 配套：`04-feat-table-resize-proposal.md`

## [code] 代码实现

- [ ] `frontend-rs/src/editor_core.rs`: `Table` struct 新增 `min_height: Option<u32>` 字段（向后兼容，serde 默认 `None`）
- [ ] `frontend-rs/src/editor_panels.rs`: 新增 `pub fn parse_table_height(input: &str) -> Result<u32, String>`，对称 `parse_table_width`（行 7473）的解析规则：接受正整数 u32；拒绝 0/空/非数字
- [ ] `frontend-rs/src/editor_panels.rs`: Inspector 在 `width` 输入框旁（或下方）加 `min_height` 输入框，data-testid 命名仿 `inspector-table-min-height`
- [ ] `frontend-rs/src/editor_panels.rs`: Canvas 渲染表时计算 `render_height = max(min_height.unwrap_or(DEFAULT_MIN_HEIGHT), fields.len() * ROW_HEIGHT)`，其中 `DEFAULT_MIN_HEIGHT` 与 `ROW_HEIGHT` 为新常量（值在实现时定，需回归测视觉）
- [ ] `frontend-rs/src/editor_panels.rs`: 表 record 在 store 更新路径上传播 `min_height`（与 `width` 同样的 setter 模式）
- [ ] `frontend-rs/src/editor_panels.rs`: OT 操作（如有）在新建/编辑表时携带 `min_height` 字段（参考 `width` 在 OT op 序列化里的处理）

## [test] 测试

- [ ] `frontend-rs/tests/tokens.rs`（或新建 `tests/table_height.rs`）: 新增 UT-MM-12
  - happy: `parse_table_height("200") → Ok(200)`
  - happy: `parse_table_height("100") → Ok(100)`
  - edge: `parse_table_height("0") → Err(...)`（0 被拒绝，与 width 一致）
  - edge: `parse_table_height("abc") → Err(...)`
  - edge: `parse_table_height("") → Err(...)`
  - edge: `parse_table_height("-5") → Err(...)`（负数被拒绝，与 width 一致）
- [ ] `frontend-rs/src/editor_panels.rs` 单元测试模块（如有）：`test_render_height_min_overrides_fields` —— 验证 `render_height = max(min_height, fields×row_height)`
- [ ] 视觉回归：通过现有 Playwright spec-parity-* 跑套，验证未破坏既有视觉断言（ST-FE-ALIGN-*、ST-PU-*）

## [verify] 验收

- [ ] `cd frontend-rs && cargo test --lib`：全绿（含 UT-MM-12 新增 6 个用例）
- [ ] `cd backend && cargo test`：全绿（确认后端零改动）
- [ ] `cd mcp-server && cargo test`：全绿
- [ ] `cd frontend-rs && npm run test:spec-parity-a` / `-b` / `-c` / `-d`：全绿
- [ ] `cd frontend-rs && npm run test:unified-prototype`：全绿
- [ ] `bash scripts/run-verify-tests-clean.sh` 完整跑通
- [ ] `node scripts/validate-openlogos-ledger.mjs --report ST-MM-12` PASS（UT-MM-12 在 ledger 登记）
- [ ] `openlogos verify` Gate 3.5 PASS（包含新增 UT-MM-12 与既有所有用例）

## [spec] 规格登记

- [ ] 在 `logos/resources/test/core-CR-canvas-test-cases.md`（或同类 canvas 测试用例 spec 文件）追加 UT-MM-12 行：
  ```
  | UT-MM-12 | parse_table_height 纯函数测试 | `frontend-rs/tests/tokens.rs` |
  ```
- [ ] 确认 UT-MM-12 已被 reporter 写入 `test-results.jsonl`（cargo test 触发）

## [archive] 归档

- [ ] `openlogos archive feat-table-resize`（待 verify Gate 3.5 PASS + 外环独立复验后；operator 授权后执行）

## 完成记录

> 实现完成后填入 commit hash 与验证证据。

---

## 实现顺序建议

1. 数据结构（`editor_core.rs::Table.min_height`）+ serde 默认值
2. 纯函数 `parse_table_height` + UT-MM-12（与宽度同模板）
3. Inspector UI（`min_height` 输入框，复用宽度 UI 组件）
4. Canvas 渲染逻辑（`max(min_height, fields×row_height)`）
5. OT op 携带字段（参考 `width`）
6. spec 登记（UT-MM-12 行）
7. 全量 verify

每步独立 commit，commit message 格式参考 `fix-auth-register-redact` 系列的 `feat(<module>): ...` 风格。