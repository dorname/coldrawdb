# Delta: core-S04-room-lifecycle-design.md

> 提案：fix-appbar-roomname-back-and-import-merge

## MODIFIED — S04.4 房间内编辑 · AppBar 增量

**merge 时替换「AppBar 增量（room 上下文）」ASCII 行为**：

```
[← 空间] [C] [room-badge: 评审周会…] [diagram 标题…] [成员头像×3 +2] [邀请] ... [user-menu]
```

**并追加以下条款**：

- **显性返回入口（fix-appbar-roomname-back-and-import-merge）**：room-badge 左侧新增「← 空间」按钮（`data-testid="btn-back-to-rooms"`，ghost 样式，图标 + 文字「空间」），点击返回房间列表页（`on_open_rooms`，与 room-badge 原有点击行为一致）；room-badge 点击行为保留不变
- **名称缩略**：room-badge 名称与 diagram 标题超长时 `max-width` + `text-overflow: ellipsis` + `white-space: nowrap` 缩略，并带 `title` 属性悬停显示全文（对齐主原型 `.room-badge` 已有口径：badge max-width 190px、strong ellipsis）
