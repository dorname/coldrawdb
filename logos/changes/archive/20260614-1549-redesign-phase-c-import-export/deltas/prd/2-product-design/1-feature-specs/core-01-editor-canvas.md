# Delta — core-01-editor-canvas.md

## ADDED — §6 空白画布引导（EmptyGuide / Phase C）

> merge 时在 §5 渲染策略之后插入新 §6，原 §6 及以后章节序号顺延（若主文档已有 §6+，则作为 §6.1 子节合并）。

### 6.1 EmptyGuide 布局（Phase A 已实现）

居中卡片，testid：`canvas-empty-guide`。

### 6.2 引导动作（Phase C 更新）

| 按钮 | testid | Phase A | Phase C |
|------|--------|---------|---------|
| + 创建第一张表 | `guide-create-table` | 创建默认表 | 不变 |
| ↑ 导入 SQL | `guide-import-sql` | disabled | **启用** → 打开 ImportDrawer |

- Phase C 移除 `disabled` 与「即将推出」tooltip
- 与 AppBar `btn-import` 打开同一 `ImportDrawer` 实例

### 6.3 测试 ID

| TC ID | 描述 |
|-------|------|
| UT-PA-03 | EmptyGuide 零表时可见（Phase A） |
| UT-PC-06 | `guide-import-sql` 点击打开 `import-drawer` |
