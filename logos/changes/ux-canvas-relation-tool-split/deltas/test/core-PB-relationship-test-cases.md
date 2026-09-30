## ADDED — ux-canvas-relation-tool-split 关系工具拆分为创建/选中两个图标

### UT-PB-27 — 关系工具创建与选中模式切换

- **位置**：`frontend-rs/src/editor_panels.rs`（`ActiveTool` / `ToolRail` / 事件分发）
- **前置**：非只读 Viewer 的编辑器上下文
- **步骤**：
  1. 点击 `tool-relationship-create`，断言 `active_tool == ActiveTool::Relationship`
  2. 点击 `tool-relationship-select`，断言 `active_tool == ActiveTool::RelationshipSelect` 且 `rel_tool_state == RelToolState::Idle`
  3. 在 `RelationshipSelect` 模式下点击画布上的关系线，断言 `selection == SelectionKind::Reference(ref_id)`
  4. 在 `RelationshipSelect` 模式下点击空白处，断言 `selection == SelectionKind::None`
- **断言**：
  - `Relationship` 模式下仍可通过拖字段/点击两点创建关系
  - `RelationshipSelect` 模式下不进入 `PickSource / PickTarget / Dragging`

### ST-PB-16 — e2e：通过 ToolRail 切换到“选中关系”工具并点击关系线

- **位置**：`frontend-rs/scripts/test-spec-parity-d.mjs`
- **GIVEN**：presetDiagram 预置两张表与一条关系 `r1`
- **WHEN**：点击 `tool-relationship-select`，再点击画布上关系 `r1` 的可见路径
- **THEN**：
  1. `active_tool === "relationship-select"`（探针暴露）
  2. `sel_ref_id === "r1"`
  3. Inspector 显示关系详情面板
  4. 切换回 `tool-relationship-create` 后点击关系线不选中（不触发创建）
- **reporter**：`ST-PB-16` 写入 `logos/resources/verify/test-results.jsonl`

### 附录 A 追加

| ID | 标题 | 对齐实现 |
|---|---|---|
| UT-PB-27 | 关系工具创建/选中模式切换 | `editor_panels.rs::ActiveTool` / `ToolRail` |
| ST-PB-16 | e2e 选中关系工具点击关系线 | `test-spec-parity-d.mjs` canvas hit |
