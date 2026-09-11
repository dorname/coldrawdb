# Delta: core-S04-test-cases.md

> 提案：fix-appbar-roomname-back-and-import-merge

## ADDED — ST-S04-UI-12 用例行（UI 表）

| ST-S04-UI-12 | 已进入 room-editor | 观察 AppBar 并点击「← 空间」 | `btn-back-to-rooms` 可见且位于 room-badge 左侧；点击返回 `/rooms` 房间列表页；长房间名时 room-badge ellipsis 缩略且 `title` 属性为全文 | fix-appbar-roomname-back-and-import-merge |

## ADDED — UT-S04-UI-12 — 名称缩略锚点测试

- **位置**：`frontend-rs/src/editor_panels.rs`（AppBar 组件）
- **断言**（`include_str!` 锚点口径）：
  1. 存在 `data-testid="btn-back-to-rooms"`
  2. room-badge 渲染含 `title=` 属性绑定（悬停全文）
  3. styles.css 含 room-badge 名称缩略规则（`text-overflow:ellipsis`）
