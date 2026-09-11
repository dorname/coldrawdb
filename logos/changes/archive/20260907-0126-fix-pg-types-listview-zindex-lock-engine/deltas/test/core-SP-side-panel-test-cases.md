## MODIFIED — ST-SP-LIST-01 列表视图全屏渲染（网格定位回归）

### ST-SP-LIST-01 — 列表视图全屏渲染（网格定位 + 层叠遮挡回归）

- 前置：画布已有 ≥2 张表（含字段与索引）
- 步骤：AppBar 视图切换 →「列表」`[data-testid="btn-view-list"]`
- 预期：
  1. `.cdb-main` 隐藏，`[data-testid="list-view-panel"]` 占据主区行（grid-row: 2），过滤工具条与表格**完整可见**（回归：不再被网格自动放置挤到状态栏行裁掉）
  2. **绘制层叠断言**：面板中心点 `document.elementFromPoint(cx, cy)` 命中的最上层元素位于面板内（`.closest('[data-testid="list-view-panel"]')` 非空）——面板必须绘制在 `.cdb-aurora` 背景层之上（修复前面板为非定位元素，被 `position:fixed` 不透明 aurora 盖死；几何尺寸断言无法发现此类遮挡）
  3. 表结构清单按表分组展示（表名/字段数/类型/索引），排序/过滤可用
  4. 双击表行 → 切回画布模式并选中该表
  5. 清空全部表后切列表视图 → 展示空态而非布局塌陷
- 对齐实现：`frontend-rs/src/styles.css`（`.cdb-list-view-panel`：grid-row: 2 + `position:relative; z-index:1`）；`frontend-rs/src/editor_panels.rs::ListView`

## MODIFIED — 附录 A：用例 ID 清单（OpenLogos verify 解析用）

附录 A 中 ST-SP-LIST-01 行更新为：

| ST-SP-LIST-01 | 列表视图全屏渲染（网格定位 + 层叠遮挡回归） | `frontend-rs/src/styles.css` + `frontend-rs/src/editor_panels.rs` |
