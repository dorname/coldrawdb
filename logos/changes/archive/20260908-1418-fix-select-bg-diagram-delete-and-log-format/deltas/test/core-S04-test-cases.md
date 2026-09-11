# Delta: core-S04-test-cases.md

> 提案：fix-select-bg-diagram-delete-and-log-format

## ADDED — 汇总表行（UI / 页面流用例表后）

| UT-S04-15 | DB 含 active 与 is_deleted=1 的 diagram | `GET /diagrams/queryAll` | 200；返回列表不含已软删 diagram |
| UT-S04-UI-16 | — | `include_str!` 锚点 | `editor_panels.rs` 含 `create-room-diagram-delete` 与 `modal-delete-diagram`；删除接线 `client.delete(`；仅选中既有图（非 `__new__`）时渲染 |
| ST-S04-UI-16 | 已登录，queryAll 返回 1 张未绑定孤儿图 | 打开创建房间模态 → 选中该图 → 删除此图表 → 确认 | `modal-delete-diagram` 含图名；确认调 `DELETE /api/v1/diagrams/{id}`；选项列表刷新不再含该图；选中回退「新建空白模型」；Toast「已删除图表」 |

## ADDED — 用例详情

### UT-S04-15 — queryAll 过滤软删 diagram

- **位置**：`diagrams::tests::ut_s04_15_query_all_excludes_deleted`
- **断言**：插入两张 diagram（一张 is_deleted=1）→ `query_all_diagrams` 返回仅含未删图

### UT-S04-UI-16 — 创建房间模态删除图入口锚点

- **位置**：`editor_panels::tests::test_create_room_diagram_delete_anchor_ut_s04_ui_16`
- **断言**（`include_str!` 口径）：
  1. `data-testid="create-room-diagram-delete"` 存在且渲染条件含 `__new__` 排除
  2. `data-testid="modal-delete-diagram"` 二次确认模态存在
  3. 删除接线 `client.delete(`（DELETE /api/v1/diagrams/{id}）

### ST-S04-UI-16 — 创建房间模态删除孤儿图 e2e

- **位置**：`frontend-rs/scripts/test-spec-parity-d.mjs`
- **前置**：mock `GET /diagrams/queryAll` 返回 1 张未绑定图「孤儿图」；mock `DELETE /api/v1/diagrams/{id}` 204
- **步骤**：`/rooms` → 创建房间 → diagram 下拉选「孤儿图」→ 点「删除此图表」→ 模态确认
- **断言**：模态含图名；发出 DELETE；选项刷新后下拉不再含「孤儿图」；选中回退 `__new__`；Toast「已删除图表」

## ADDED — 变更记录行

| UT-S04-15 / UT-S04-UI-16 / ST-S04-UI-16 | ADDED（fix-select-bg-diagram-delete-and-log-format） | queryAll 过滤软删 + 创建房间模态删除孤儿图入口 |
