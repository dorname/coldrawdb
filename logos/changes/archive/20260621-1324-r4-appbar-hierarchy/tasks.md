# 实现任务

## [delta] 规格变更
- [x] 产出 delta → `deltas/prd/2-product-design/1-feature-specs/core-05-top-menu-modals.md` — R4 AppBar 三区 + 溢出菜单
- [x] 产出 delta → `deltas/prd/2-product-design/1-feature-specs/core-00-information-architecture.md` — AppBar 信息架构
- [x] 产出 delta → `deltas/prd/2-product-design/1-feature-specs/core-07-design-tokens.md` — AppBar 分区间距 Token
- [x] 产出 delta → `deltas/prd/2-product-design/2-page-design/core-01-editor-prototype.html` — 原型 AppBar 结构

## [code] 代码实现
- [x] `editor_panels.rs` — AppBar 三区布局、`SaveStatusChip`、`AppBarOverflowMenu`
- [x] `styles.css` — `.cdb-app-bar__brand/status/actions`、`.cdb-status-chip`、溢出菜单样式
- [x] `code_view.rs` — `ViewModeToggle` 改为图标按钮（减宽）
- [x] 更新 UT-AB-05 断言：`revision-display` 位于状态 Chip（移除 StatusBar 重复）
