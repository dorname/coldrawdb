# Delta — core-KB-shortcut-test-cases.md（fix-issues-38-41-canvas-interaction）

> 模块：core | 提案：fix-issues-38-41-canvas-interaction
> 关联 issue：#38

## ADDED — UT 用例（追加）

| ID | 输入/操作 | 断言 |
|---|---|---|
| UT-KB-07 | pan/zoom 与撤销栈解耦（#38 §5.14） | 纯函数/锚点断言：pan 与 zoom 代码路径不触碰 CommandStack（`editor_render.rs` pan 分支负断言无 `record(` / `Command::` 写入）；undo/redo 栈在模拟 pan 序列后长度不变；键盘处理器在 drag_state 刚清空（pan 松手）后仍正常分发 undo/redo |

## ADDED — ST 用例（追加）

| ID | 前置与步骤 | 预期 |
|---|---|---|
| ST-KB-UNDO-01 | e2e：建表（undoable op）→ 空白拖动 pan 画布（位移 ≥ 50px）→ Ctrl+Z → Ctrl+Y；再 pan 一次 → Ctrl+Z | pan 后 Ctrl+Z 撤销建表（表消失，计数 1→0）；Ctrl+Y 重做（0→1）；二次 pan 后 Ctrl+Z 仍生效（R-PAN-UNDO-02/03/04 回归锚点） |

## MODIFIED — 用例登记（OpenLogos verify 解析用，追加行）

| ID | 层级 | 说明 |
|---|---|---|
| UT-KB-07 | UT | pan/zoom 不进撤销栈且不击穿快捷键链路（#38） |
| ST-KB-UNDO-01 | ST | pan 后 Ctrl+Z / Ctrl+Y 撤销重做正常（#38 e2e 锚点） |
