## ADDED — UT-ALIGN-A01 — Areas/Notes Tab 与 store 同源

**步骤**：
1. `EditorStore` 初始 `areas`/`notes` 为空
2. 模拟 AreasTab 新增逻辑向 `store.areas` push 默认 `Area`
3. `snapshot()` 断言 `areas.len() == 1` 且 `name` 一致
4. 模拟 NotesTab 新增逻辑向 `store.notes` push 默认 `Note`
5. `snapshot()` 断言 `notes.len() == 1`

**预期**：侧栏与保存 payload 使用同一 store 信号
