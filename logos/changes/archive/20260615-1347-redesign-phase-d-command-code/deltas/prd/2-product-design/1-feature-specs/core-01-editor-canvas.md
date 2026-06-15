# Delta — core-01-editor-canvas.md

## ADDED — §7 视图模式切换（Phase D）

> merge 时在 §6 EmptyGuide 之后追加。

### 7.1 Canvas ↔ Code

| 事件 | Canvas 视图 | Code 视图 |
|------|-------------|-----------|
| 进入 | 默认 | `view_mode = Code` |
| 可见区域 | Tool Rail + Canvas + Inspector/IO | 仅 CodeView 面板 |
| 选中态 | 正常 | 保留内存，返回 Canvas 时恢复 |
| 缩放 / 平移 | 有效 | 暂停（transform 不丢） |

### 7.2 Command Palette 与画布

- Palette 跳转表/关系：若当前为 Code 视图，先切回 Canvas 再跳转（V1）
- Palette 打开导入：可保持 Canvas 视图并开 ImportDrawer

### 7.3 测试 ID

| TC ID | 描述 |
|-------|------|
| ST-PD-01 | Ctrl+K 搜表 → Enter → 画布选中 |
