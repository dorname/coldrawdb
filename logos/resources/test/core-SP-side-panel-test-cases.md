# 侧栏 Tab 测试用例规格

> 模块：core | 提案：add-frontend-completeness
> 路径：`logos/resources/test/core-SP-side-panel-test-cases.md`
> 对齐参考源：`core-04-side-panel-tabs.md` §2~§10 + §11 测试 ID 索引

## 1. 范围

侧栏 6 业务 Tab（Tables / Areas / Enums / Notes / Relationships / Types）+ Issues Tab = 7 Tab；
顶部全局搜索框 + 类型筛选下拉（spec §10）。

**对应实现**：`frontend-rs/src/editor_panels.rs::LeftPanel`（7 Tab 切换 + 7 子组件 + 搜索/筛选）

**对账**：
- UT-SP-02（来自 `core-04-side-panel-tabs.md` §11）
- UT-SP-09 / UT-SP-10（本文件新增，B2 范围）
- ST-SP-01（来自 §11，B2 间接覆盖）

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

## 4. V1 边界

- ❌ 全局搜索的 Tab 间跳转头（点击搜索结果跳到对应 Tab）— B3 接入
- ❌ 类型筛选下拉的 field type 完整集合 — B2 暂用 `INT / VARCHAR(255) / TEXT / BOOLEAN` 4 类硬编码
- ❌ Areas/Enums/Notes/Types Tab 内的双击重命名 / 右键菜单 — B3 接入
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
| ST-SP-01 | 端到端 5 表 0 error | `frontend-rs/tests/wasm/sp.rs` |
