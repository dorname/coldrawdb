# Delta: core-05-top-menu-modals.md

> 提案：fix-overflow-menu-room-delete-listview-io

## ADDED — §1.1 溢出菜单视口边界保护

> 锚点：追加于 §1.1「溢出菜单 `⋯`（R4）」表格之后。

**视口边界保护（fix-overflow-menu-room-delete-listview-io）**：

- 溢出菜单须约束在视口内完整可见：
  - `max-height: calc(100vh - 80px)`（AppBar 64px + 间距余量），超出时 `overflow-y: auto` 菜单体内滚动；
  - 右缘夹取：菜单右缘不得超出视口右缘（`right: 0` 锚定之外，加 `max-width: calc(100vw - 16px)` 兜底）；
  - 小视口（如高度 < 480px）下所有菜单项仍可经滚动全部到达，不得裁切。
- 回归锚点：`app-bar-overflow-menu` 打开后其 bounding box 必须完整落在视口内（right ≤ viewport width，bottom ≤ viewport height）。
