## MODIFIED — ST-SP-LIST-01 列表视图全屏渲染（网格定位 + 层叠遮挡回归）

### ST-SP-LIST-01 — 列表视图全屏渲染（网格定位 + 层叠遮挡回归）

- 前置：画布已有 ≥2 张表（含字段与索引）
- 步骤：AppBar 视图切换 →「列表」`[data-testid="btn-view-list"]`
- 预期：
  1. `.cdb-main` 隐藏，`[data-testid="list-view-panel"]` 占据主区行（grid-row: 2），工具条与网格**完整可见**（回归：不再被网格自动放置挤到状态栏行裁掉）
  2. **绘制层叠断言**：面板中心点 `document.elementFromPoint(cx, cy)` 命中的最上层元素位于面板内（`.closest('[data-testid="list-view-panel"]')` 非空）——面板必须绘制在 `.cdb-aurora` 背景层之上
  3. **编辑网格渲染**：表头为 10 列（字段代码/显示名称/主键/不为空/唯一/自增/数据类型/长度/小数位/说明）；按表分组出组头行「表名（N 字段）」；字段行内嵌文本输入/复选框/类型下拉——列表视图即字段明细编辑网格（redesign-listview-type-length-canvas-fix，不再是只读清单）
  4. 双击表行 → 切回画布模式并选中该表
  5. 清空全部表后切列表视图 → 展示空态而非布局塌陷
- 对齐实现：`frontend-rs/src/styles.css`（`.cdb-list-view-panel`：grid-row: 2 + `position:relative; z-index:1`）；`frontend-rs/src/editor_panels.rs::ListView`

## ADDED — ST-SP-LIST-02 列表视图行内编辑落账（PDManer 式字段明细网格）

### ST-SP-LIST-02 — 列表视图行内编辑落账（PDManer 式字段明细网格）

- 前置：房间编辑器内已有 2 张表（table_1 含 ≥2 字段），引擎 = PostgreSQL
- 步骤：
  1. 切列表视图 → 点击 table_1 首字段行（选中高亮）
  2. 行内改字段代码（名称）→ blur；改显示名称 → blur
  3. 类型下拉切 VARCHAR → 长度输入 32 → blur；勾选「不为空」
  4. 工具条「增字段」→ 新行出现；「上移/下移」→ 字段顺序变化；「删字段」→ 行消失
- 预期：
  1. 每次编辑后 PUT 落账快照正确：`fields[i].name/comment/not_null` 更新；`type_` 为合成整串 `VARCHAR(32)`（core-01a §2.2 参数化口径）
  2. 类型下拉选项为 PG 基类型清单（含 UUID/SERIAL/NUMERIC，无 `VARCHAR(255)` 整串项）；长度列仅 VARCHAR 系可编辑，小数位列仅 NUMERIC/DECIMAL 系可编辑
  3. 增/删/移动后 PUT 快照的 fields 数组长度与顺序正确；删字段可撤销（Ctrl+Z 恢复）
  4. 编辑期间输入不丢字（受控输入 + 防抖/blur 落账）
- 对齐实现：`frontend-rs/src/editor_panels.rs::ListView`；`frontend-rs/src/editor_core.rs::parse_field_type/compose_field_type`

## MODIFIED — 附录 A：用例 ID 清单（OpenLogos verify 解析用）

附录 A 中 ST-SP-LIST-01 行更新为，并新增 ST-SP-LIST-02 行：

| ST-SP-LIST-01 | 列表视图全屏渲染（网格定位 + 层叠遮挡 + 编辑网格渲染回归） | `frontend-rs/src/styles.css` + `frontend-rs/src/editor_panels.rs` |
| ST-SP-LIST-02 | 列表视图行内编辑落账（PDManer 式字段明细网格） | `frontend-rs/src/editor_panels.rs::ListView` |
