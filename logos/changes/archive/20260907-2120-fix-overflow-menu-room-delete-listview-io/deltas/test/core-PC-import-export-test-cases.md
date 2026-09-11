# Delta: core-PC-import-export-test-cases.md

> 提案：fix-overflow-menu-room-delete-listview-io

## ADDED — 用例行

| TC ID | Given | When | Then |
|-------|-------|------|------|
| ST-PC-02 | 已进入 ListView（有表） | 工具条点「导入」→ 粘贴含 1 张表的 DDL → 提交；再点「导出」 | `list-btn-import` 打开 ImportDrawer；提交走本地合并（PUT 表数 +1、Toast「已导入」、不跳转、不调 bridge）；关闭抽屉回到 ListView 且表树含新表；`list-btn-export` 打开 ExportDrawer 预览含当前模型 |
| ST-PC-03 | 编辑器已加载，视口 720×480 | 打开更多菜单 | `app-bar-overflow-menu` bounding box 完整落在视口内（right ≤ 720、bottom ≤ 480）；菜单可滚动到达末项「命令面板」 |
| UT-PC-10 | — | `include_str!` 锚点 | `editor_panels.rs` 含 `list-btn-import` / `list-btn-export` 且 IoDrawer 在 ListView 态可渲染；`styles.css` 溢出菜单含 `max-height` + `overflow-y` |

### UT-PC-10 — ListView IO 入口与菜单边界锚点测试

- **位置**：`frontend-rs/src/editor_panels.rs`
- **断言**（`include_str!` 锚点口径）：
  1. 存在 `data-testid="list-btn-import"` 与 `data-testid="list-btn-export"`
  2. ListView 工具条导入按钮接线到 IoDrawer 打开（on_open_import / on_open_export 或等价路径）
  3. `styles.css` 的 `.cdb-app-bar__overflow-menu` 规则含 `max-height` 与 `overflow-y`

### 变更记录

| TC ID | 变更 | 说明 |
|-------|------|------|
| ST-PC-02 | ADDED（fix-overflow-menu-room-delete-listview-io） | ListView 导入/导出入口端到端 |
| ST-PC-03 | ADDED（fix-overflow-menu-room-delete-listview-io） | 更多菜单小视口边界保护 |
| UT-PC-10 | ADDED（fix-overflow-menu-room-delete-listview-io） | ListView IO 按钮 + 菜单边界 CSS 锚点 |
