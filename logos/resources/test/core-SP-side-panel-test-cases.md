# 侧栏 Tab 测试用例规格

> 模块：core | 提案：add-frontend-completeness
> 路径：`logos/resources/test/core-SP-side-panel-test-cases.md`
> 对齐参考源：`core-04-side-panel-tabs.md` §2~§10 + §11 测试 ID 索引

## 1. 范围

Inspector 锚点与响应式抽屉。验收锚点：**`data-testid="inspector"`**（禁止仅用 `inspector-panel`）。

状态：后端已实现；生产前端部分接入；逐项对齐待第二阶段。实现阶段须将用例结果写入 `logos/resources/verify/test-results.jsonl`（OpenLogos reporter）；本提案仅规格收口，不执行自动化。

## ADDED / MODIFIED

| ID | 变更 | 合同 |
|---|---|---|
| UT-SP-ANCHOR-01（ADDED） | ADDED | 生产 DOM 存在 `inspector`；选中表后面板字段可编辑（可写角色） |
| ST-SP-RESP-01（ADDED） | ADDED | ≤720px：Inspector 以抽屉/叠层呈现；`btn-inspector-toggle` 可开关；关闭后不挡画布 |
| ST-SP-RESP-02（ADDED） | ADDED | 桌面三列与窄屏单列切换后，不残留错误 layout class |
| UT-SP-02/09/10 | 保留 | Tab 搜索/切换；与统一壳层共存 |
| ST-SP-VIEWER（ADDED） | ADDED | Viewer：Inspector 只读 |

## 2. UT 用例

### UT-SP-02 — Tables Tab 搜索过滤

- **位置**：`frontend-rs/src/editor_panels.rs::LeftPanel`（搜索框 + 列表）
- **前置**：store 中存在 tables: `[users, orders, products]`
- **步骤**：
  1. 渲染 LeftPanel
  2. 在搜索框输入 "user"
  3. 验证列表项渲染
- **断言**：
  - 列表项数 == 1
  - 列表项文本 == "users"
  - 内部 store 状态未变更（仅 UI 过滤）

### UT-SP-09 — 8 Tab 图标栏切换（R5）

- **位置**：`frontend-rs/src/editor_panels.rs::LeftPanel`（图标 Tab 栏 + 8 个 Tab 子组件）
- **前置**：AppRoot mount，store 包含 tables / areas / enums / notes / references / types（每种至少 1 项）
- **步骤**：
  1. 默认渲染 → 验证 Tables Tab 处于激活态
  2. 验证 Tab 栏为 `.cdb-tabs--icon-grid`（4 列 × 2 行）
  3. 依次点击 Areas / Enums / Notes / Relationships / Types / Issues / **Fields**
  4. 验证每个 Tab 内容区正确切换
- **断言**：
  - `data-testid="tab-tables"`、`tab-areas`、`tab-enums`、`tab-notes`、`tab-relationships`、`tab-types`、`tab-issues`、**`tab-fields`** 全部存在
  - 每个 Tab 含 `title` 属性（Tooltip 文案）
  - 每次点击后 `cdb-tab.cdb-is-active` 类正确指向当前 Tab
  - **不存在** `.cdb-side-panel--right` 45% 分割容器
  - `field-editor` 仅在 Fields Tab 内容区渲染
  - 切换不丢失 store 数据（只换渲染）

### UT-SP-10 — 全局搜索跨 Tab 过滤

- **位置**：`frontend-rs/src/editor_panels.rs::LeftPanel`（顶部搜索框 + 7 Tab 联动）
- **前置**：store 包含 tables: `[users]`、areas: `[user_area]`、enums: `[user_role]`
- **步骤**：
  1. 渲染 LeftPanel
  2. 在搜索框输入 "user"
  3. 验证 Tables / Areas / Enums Tab 的列表项都被过滤到含 "user" 的项
- **断言**：
  - Tables Tab 列表项 == 1（"users"）
  - Areas Tab 列表项 == 1（"user_area"）
  - Enums Tab 列表项 == 1（"user_role"）
  - Notes / Relationships / Types / Issues Tab 不受影响（若无匹配项则显示空态）

### UT-ALIGN-A01 — Areas/Notes Tab 与 store 同源

- **位置**：`frontend-rs/src/editor_panels.rs`（`AreasTab` / `NotesTab` + `new_default_area` / `new_default_note`）
- **步骤**：
  1. `EditorStore` 初始 `areas`/`notes` 为空
  2. 向 `store.areas` push 默认 `Area`
  3. `snapshot()` 断言 `areas.len() == 1` 且 `name` 一致
  4. 向 `store.notes` push 默认 `Note`
  5. `snapshot()` 断言 `notes.len() == 1`
- **预期**：侧栏与保存 payload 使用同一 store 信号

## 3. ST 用例

### ST-SP-01 — 端到端：编辑 5 表 → Issues Tab 显示 0 error（B2 间接覆盖）

- **位置**：`frontend-rs/tests/wasm/sp.rs`
- **类型**：wasm-pack test --headless
- **步骤**：
  1. 启动前端 + 后端（headless）
  2. 通过 UI 创建 5 张表，每张 4 字段，每张含主键
  3. 切换到 Issues Tab
  4. 验证错误列表为空
- **断言**：
  - Issues Tab 列表项数 == 0
  - 顶部 "Issues (0)" badge 正确
- **注**：B2 范围仅基础校验（表名重复、主键缺失、字段类型不兼容），B3 补全（端点不存在、自增非整数等）

### ST-SP-LIST-01 — 列表视图全屏渲染（网格定位 + 层叠遮挡回归）

- 前置：画布已有 ≥2 张表（含字段与索引）
- 步骤：AppBar 视图切换 →「列表」`[data-testid="btn-view-list"]`
- 预期：
  1. `.cdb-main` 隐藏，`[data-testid="list-view-panel"]` 占据主区行（grid-row: 2），工具条与网格**完整可见**（回归：不再被网格自动放置挤到状态栏行裁掉）
  2. **绘制层叠断言**：面板中心点 `document.elementFromPoint(cx, cy)` 命中的最上层元素位于面板内（`.closest('[data-testid="list-view-panel"]')` 非空）——面板必须绘制在 `.cdb-aurora` 背景层之上
  3. **树+单表网格渲染**：左侧表树 `[data-testid="list-tree"]` 列出全部表节点（`list-tree-node-<表名>`，含「N 字段」计数），首张表默认选中高亮；右侧网格表头为 10 列（字段代码/显示名称/主键/不为空/唯一/自增/数据类型/长度/小数位/说明），只渲染当前选中表的字段行，行内嵌文本输入/复选框/类型下拉；**不再出现组头行**（`list-view-group-*` 锚点移除）
  4. 双击树节点 → 切回画布模式并选中该表
  5. 清空全部表后切列表视图 → 展示空态而非布局塌陷
- 对齐实现：`frontend-rs/src/styles.css`（`.cdb-list-view-panel`：grid-row: 2 + `position:relative; z-index:1`；树+网格两栏 grid）；`frontend-rs/src/editor_panels.rs::ListView`

### ST-SP-LIST-02 — 列表视图行内编辑落账（PDManer 式字段明细网格）

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

### ST-SP-LIST-03 — 列表视图表树导航（搜索过滤 + 切表渲染 + 空态）

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

## 与 IO / 成员抽屉互斥

同时打开多个侧层时，必须有可关闭路径；不得出现不可恢复遮挡（对齐 ST-PU-17）。

## 4. V1 边界

- ❌ 全局搜索的 Tab 间跳转头（点击搜索结果跳到对应 Tab）— B3 接入
- ❌ 类型筛选下拉的 field type 完整集合 — B2 暂用 `INT / VARCHAR(255) / TEXT / BOOLEAN` 4 类硬编码
- ❌ Enums/Types Tab 内的双击重命名 / 右键菜单 — B3 接入
- ❌ Areas/Notes Tab 双击重命名 / 右键菜单 — 后续批次
- ❌ DBML Editor（spec §9）— B5 接入

## 5. 对齐参考源

- `core-04-side-panel-tabs.md` §2~§10（功能规格）+ §11（测试 ID 索引）
- `frontend-rs/src/editor_panels.rs::LeftPanel`
- `logos/resources/verify/test-results.jsonl`（reporter）

## 附录 A：用例 ID 清单（OpenLogos verify 解析用）

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
