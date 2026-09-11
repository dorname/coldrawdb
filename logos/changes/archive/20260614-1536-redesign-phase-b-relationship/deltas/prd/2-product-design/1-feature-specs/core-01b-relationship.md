# Delta — core-01b-relationship.md (Phase B)

## ADDED — §3.1 关系工具模式（Phase B）

> merge 时在 §3「关系操作」之后插入。

### 3.1 关系工具模式（Tool Rail `🔗`）

**激活**：点击 Tool Rail `tool-relationship` 或快捷键 `R`；按钮进入 `cdb-is-active` 态。

**状态机**：

| 状态 | 用户操作 | 下一状态 |
|------|----------|----------|
| `Idle` | 激活关系工具 | `PickSource` |
| `PickSource` | 点击源字段 | `PickTarget` |
| `PickTarget` | 点击目标字段 | `Confirm` |
| `Confirm` | 点「创建」 | `Idle`（关系写入 store） |
| `Confirm` | 点「取消」 | `PickSource` |
| 任意 | 按 `Esc` 或切回选择工具 | `Idle` |

**画布提示**（`data-testid="rel-tool-hint"`）：
- `PickSource`：「选择源字段」
- `PickTarget`：「选择目标字段」

### 3.2 关系确认条（非模态）

**位置**：画布底部居中（`z-index: L2`，不遮挡 AppBar / Inspector）。

```
┌──────────────────────────────────────────────────────┐
│ users.id → orders.user_id   [1:N ▼]  [创建] [取消]   │
└──────────────────────────────────────────────────────┘
```

- testid：`rel-confirm-bar` / `rel-confirm-create` / `rel-confirm-cancel` / `rel-confirm-cardinality`
- 默认 cardinality：`one_to_many`
- 创建后：写入 `Reference`，`type_` = cardinality，`on_delete`/`on_update` 默认 `RESTRICT`

## ADDED — §8 Phase B 测试 ID

| TC ID | 描述 |
|-------|------|
| UT-PB-01 | `hit_test_field` 命中字段行 |
| UT-PB-02 | `build_reference` 默认 RESTRICT |
| UT-PB-03 | `flip_reference_endpoints` 互换端点 |
| UT-PB-04 | `toggle_field_primary` 单表唯一 PK |
| ST-PB-01 | e2e：关系工具创建 1 条 reference |
