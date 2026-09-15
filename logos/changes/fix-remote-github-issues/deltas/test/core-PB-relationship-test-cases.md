# Delta — core-PB-relationship-test-cases.md（#3 字段直连）

> 变更：fix-remote-github-issues / GitHub #3

## ADDED — UT-PB-08 — Idle 下字段拖连（无需激活关系工具）

### UT-PB-08 — 默认模式下字段→字段拖出建关系

- **位置**：`frontend-rs/src/editor_render.rs` / 关系手势状态机
- **前置**：`RelToolState::Idle`；两表各一字段；非 Viewer
- **步骤**：
  1. 源字段 pointerdown → 位移 ≥4px → 目标字段 pointerup
  2. 对照：位移 &lt;4px → 不建关系（字段选中）
- **断言**：
  - `references.len()==1`
  - ToolRail 关系按钮无需预先激活（`tool-relationship` 可保持未 pressed）
  - 写入通路走 `AddReference`（与 UT-KB-03 一致）

## ADDED — ST-PB-05 — 字段直连 e2e

### ST-PB-05 — 不点关系工具即可拖连

- **步骤**：不点击 `tool-relationship`，直接字段拖到另一字段
- **断言**：关系落账；既有 ST-PB-01 / ST-PB-02 仍通过

## MODIFIED — 用例表追加行

| TC ID | Given | When | Then |
|-------|-------|------|------|
| UT-PB-08 | Idle；两表字段 | 字段拖出 ≥4px 到另一字段 | 新增 1 条 reference；无需先点关系工具 |
| ST-PB-05 | 可写编辑器 | 不激活关系工具直接拖连 | 落账成功；ST-PB-01/02 回归通过 |
