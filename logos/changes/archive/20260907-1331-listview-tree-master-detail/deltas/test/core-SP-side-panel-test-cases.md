# Delta: core-SP-side-panel-test-cases.md

## MODIFIED — ST-SP-LIST-01 — 列表视图全屏渲染（网格定位 + 层叠遮挡回归）

- 前置：画布已有 ≥2 张表（含字段与索引）
- 步骤：AppBar 视图切换 →「列表」`[data-testid="btn-view-list"]`
- 预期：
  1. `.cdb-main` 隐藏，`[data-testid="list-view-panel"]` 占据主区行（grid-row: 2），工具条与网格**完整可见**（回归：不再被网格自动放置挤到状态栏行裁掉）
  2. **绘制层叠断言**：面板中心点 `document.elementFromPoint(cx, cy)` 命中的最上层元素位于面板内（`.closest('[data-testid="list-view-panel"]')` 非空）——面板必须绘制在 `.cdb-aurora` 背景层之上
  3. **树+单表网格渲染**：左侧表树 `[data-testid="list-tree"]` 列出全部表节点（`list-tree-node-<表名>`，含「N 字段」计数），首张表默认选中高亮；右侧网格表头为 10 列（字段代码/显示名称/主键/不为空/唯一/自增/数据类型/长度/小数位/说明），只渲染当前选中表的字段行，行内嵌文本输入/复选框/类型下拉；**不再出现组头行**（`list-view-group-*` 锚点移除）
  4. 双击树节点 → 切回画布模式并选中该表
  5. 清空全部表后切列表视图 → 展示空态而非布局塌陷
- 对齐实现：`frontend-rs/src/styles.css`（`.cdb-list-view-panel`：grid-row: 2 + `position:relative; z-index:1`；树+网格两栏 grid）；`frontend-rs/src/editor_panels.rs::ListView`

## MODIFIED — ST-SP-LIST-02 — 列表视图行内编辑落账（PDManer 式字段明细网格）

- 前置：房间编辑器内已有 2 张表（table_1 含 ≥2 字段），引擎 = PostgreSQL
- 步骤：
  1. 切列表视图 → 左树单击 table_1 节点（默认选中首表时可直接操作）→ 点击首字段行（选中高亮）
  2. 行内改字段代码（名称）→ blur；改显示名称 → blur
  3. 类型下拉切 VARCHAR → 长度输入 32 → blur；勾选「不为空」
  4. 工具条「增字段」→ 新行出现在**当前树选中表**；「上移/下移」→ 字段顺序变化；「删字段」→ 行消失
- 预期：
  1. 每次编辑后 PUT 落账快照正确：`fields[i].name/comment/not_null` 更新；`type_` 为合成整串 `VARCHAR(32)`（core-01a §2.2 参数化口径）
  2. 类型下拉选项为 PG 基类型清单（含 UUID/SERIAL/NUMERIC，无 `VARCHAR(255)` 整串项）；长度列仅 VARCHAR 系可编辑，小数位列仅 NUMERIC/DECIMAL 系可编辑
  3. 增/删/移动后 PUT 快照的 fields 数组长度与顺序正确；删字段可撤销（Ctrl+Z 恢复）
  4. 编辑期间输入不丢字（受控输入 + 防抖/blur 落账）
- 对齐实现：`frontend-rs/src/editor_panels.rs::ListView`；`frontend-rs/src/editor_core.rs::parse_field_type/compose_field_type`

## ADDED — ST-SP-LIST-03 — 列表视图表树导航（搜索过滤 + 切表渲染 + 空态）

- 前置：房间编辑器内已有 ≥3 张表（table_1 / table_2 / users，字段数各不相同），引擎 = PostgreSQL
- 步骤：
  1. 切列表视图 → 左树展示 3 个节点，首张表（table_1）默认选中，右网格仅含 table_1 字段行
  2. 树搜索框输入 `user` → 树只剩 users 节点；点击该节点 → 右网格切换为 users 字段明细，节点高亮，Inspector 同步选中 users
  3. 清空搜索框 → 3 个节点恢复；点击 table_2 → 右网格切换且字段行选中态清空（`list_sel` 重置）
  4. 搜索框输入不存在的关键字（如 `zzz`）→ 树内空态提示；右网格保持当前表不变
  5. 画布侧删除当前选中表后再切列表视图 → 树选中回落首张剩余表，不白屏
- 预期：
  1. 搜索过滤为表名大小写不敏感包含匹配（复用 `filter_tables` 口径）；过滤不影响右网格当前表
  2. 切表后 PUT 快照无变化（导航不落账）；字段行选中态确实清空（删/移按钮禁用）
  3. 双击 users 节点 → 切回画布并选中 users
- 对齐实现：`frontend-rs/src/editor_panels.rs::ListView`（表树子组件 + `filter_tables` 复用）

## MODIFIED — 附录 A：用例 ID 清单（OpenLogos verify 解析用）

| ID | 标题 | 对齐实现 |
|---|---|---|
| UT-SP-02 | Tables Tab 搜索过滤 | `frontend-rs/src/editor_panels.rs` |
| UT-SP-09 | 8 Tab 图标栏切换 | `frontend-rs/src/editor_panels.rs` |
| UT-SP-10 | 全局搜索跨 Tab 过滤 | `frontend-rs/src/editor_panels.rs` |
| UT-ALIGN-A01 | Areas/Notes 与 store 同源 | `frontend-rs/src/editor_panels.rs` |
| ST-SP-01 | 端到端 5 表 0 error | `frontend-rs/tests/wasm/sp.rs` |
| ST-SP-LIST-01 | 列表视图全屏渲染（网格定位 + 层叠遮挡 + 树+单表网格渲染回归） | `frontend-rs/src/styles.css` + `frontend-rs/src/editor_panels.rs` |
| ST-SP-LIST-02 | 列表视图行内编辑落账（PDManer 式字段明细网格） | `frontend-rs/src/editor_panels.rs::ListView` |
| ST-SP-LIST-03 | 列表视图表树导航（搜索过滤 + 切表渲染 + 空态） | `frontend-rs/src/editor_panels.rs::ListView` |
