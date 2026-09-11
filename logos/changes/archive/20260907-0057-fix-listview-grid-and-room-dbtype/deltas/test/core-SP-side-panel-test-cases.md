## ADDED — 3. ST 用例（追加）

### ST-SP-LIST-01 — 列表视图全屏渲染（网格定位回归）

- 前置：画布已有 ≥2 张表（含字段与索引）
- 步骤：AppBar 视图切换 →「列表」`[data-testid="btn-view-list"]`
- 预期：
  1. `.cdb-main` 隐藏，`[data-testid="list-view-panel"]` 占据主区行（grid-row: 2），过滤工具条与表格**完整可见**（回归：不再被网格自动放置挤到状态栏行裁掉）
  2. 表结构清单按表分组展示（表名/字段数/类型/索引），排序/过滤可用
  3. 双击表行 → 切回画布模式并选中该表
  4. 清空全部表后切列表视图 → 展示空态而非布局塌陷
- 对齐实现：`frontend-rs/src/styles.css`（`.cdb-list-view-panel`）；`frontend-rs/src/editor_panels.rs::ListView`

## ADDED — 附录 A：用例 ID 清单（追加行）

| ID | 标题 | 对齐实现 |
|---|---|---|
| ST-SP-LIST-01 | 列表视图全屏渲染（网格定位回归） | `frontend-rs/src/styles.css` + `frontend-rs/src/editor_panels.rs` |
