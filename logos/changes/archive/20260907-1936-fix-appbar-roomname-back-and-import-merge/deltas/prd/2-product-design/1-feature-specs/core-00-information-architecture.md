# Delta: core-00-information-architecture.md

> 提案：fix-appbar-roomname-back-and-import-merge

## MODIFIED — §1.1 AppBar 信息架构（R4）

**merge 时替换 AppBar ASCII（R4）行为**：

```
| [← 空间] Logo Undo Redo [Title________] | ● 已保存 · rev:5 |     [分享][<>][⋯] |
```

**并在品牌区行后追加说明**：

> 房间上下文时品牌区最左侧出现「← 空间」返回按钮（`btn-back-to-rooms`，见 `core-S04-room-lifecycle-design.md` S04.4）；品牌区名称元素（room-badge / diagram 标题）超长一律 ellipsis 缩略 + `title` 悬停全文，不挤压状态区与操作区。
