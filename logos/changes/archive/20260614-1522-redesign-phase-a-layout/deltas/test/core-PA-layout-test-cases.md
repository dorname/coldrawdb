# Delta — core-PA-layout-test-cases.md（新文件）
# 模块：core | 提案：redesign-phase-a-layout
# merge 目标：`logos/resources/test/core-PA-layout-test-cases.md`

## ADDED — Phase A 布局重构测试用例

> 模块：core | 提案：redesign-phase-a-layout
> 路径：`logos/resources/test/core-PA-layout-test-cases.md`
> 对齐规格：`core-00` §8、`core-01` §11、`core-04` §11、`core-05` §10、`core-06` §9

# Phase A 布局重构 — 测试用例

## 1. 范围

覆盖 V2 布局重构的单元测试（UT）与场景测试（ST）：

- 单行 AppBar（合并双顶栏）
- Tool Rail（替代左栏 7 Tab）
- 可折叠 Inspector
- 选中态统一
- 空白画布引导卡片
- StatusBar revision 迁移

**不在范围**：关系工具（Phase B）、导入抽屉（Phase C）、Command Palette（Phase D）。

## 2. AppBar 组

### UT-AB-01 AppBar 单行无独立 Toolbar

- **Given** 编辑器已挂载（`editor-ready` 可见）
- **When** 查询 DOM
- **Then**
  - 存在 `[data-testid="app-bar"]`
  - **不存在** `[data-testid="toolbar"]`
  - `.cdb-app` 的 `grid-template-rows` 计算值为 `48px 1fr 28px`（styles 断言或 computed style）

### UT-AB-02 项目名单击编辑

- **Given** 当前标题为 `Untitled Diagram`
- **When** 单击 `[data-testid="diagram-title"]`
- **Then** 出现 input；输入 `电商 ER` 并 blur
- **And** 标题显示 `电商 ER`；`on_title_blur` 被调用

### UT-AB-03 dirty 星号

- **Given** `store.dirty = false`
- **When** 修改任意字段触发 dirty
- **Then** 项目名旁显示 `*` 或 SaveState 显示未保存

### UT-AB-04 导入按钮 Phase A 占位

- **Given** AppBar 已渲染
- **When** 检查 `[data-testid="btn-import"]`
- **Then** `disabled=true`；title 含「即将推出」

### UT-AB-05 revision 在 StatusBar

- **Given** `store.revision = 5`
- **When** 查询 `[data-testid="revision-display"]`
- **Then** 文本含 `rev: 5`；祖先元素 class 含 `cdb-footer` 或 `cdb-status-bar`

## 3. Tool Rail 组

### UT-TR-01 Tool Rail 渲染

- **Given** 编辑器已挂载
- **When** 查询 `[data-testid="tool-rail"]`
- **Then** 元素存在；宽度 ≤ 48px

### UT-TR-02 新建表

- **Given** 画布空白（0 表）
- **When** 点击 `[data-testid="tool-new-table"]`
- **Then** `store.tables.len() == 1`；`SelectionState.kind == table`

### UT-TR-03 Issues 徽章计数

- **Given** 校验器返回 2 error + 1 warning
- **When** 渲染 Tool Rail
- **Then** 徽章显示 `3`（或 configurable：仅 error 显示 `2`，实现时按 spec 定）

### UT-TR-04 Issues 打开 Inspector

- **Given** Issues 列表非空
- **When** 点击 Issues 徽章
- **Then** Inspector 展开；内容区含错误项列表

## 4. Inspector 组

### UT-IN-01 选表自动展开

- **Given** Inspector 折叠
- **When** 画布单击某表
- **Then** Inspector 展开；标题含表名

### UT-IN-02 选字段表单

- **Given** 表 `users` 含字段 `email`
- **When** 单击字段行
- **Then** `[data-testid="inspector-field-name"]` value 为 `email`

### UT-IN-03 折叠按钮

- **Given** Inspector 展开
- **When** 点击 `[data-testid="btn-inspector-toggle"]`
- **Then** `.cdb-main` 含 class `cdb-is-inspector-collapsed`

### UT-IN-04 Issues 子视图

- **Given** 点击 Issues 徽章
- **When** Inspector 渲染
- **Then** 列表项数 = 校验错误数；每项有「定位」按钮

### UT-IN-05 字段列表行选中

- **Given** `InspectorTable` 显示字段列表
- **When** 单击字段行 `id`
- **Then** `SelectionState.kind == field` 且 `fieldId` 匹配

## 5. 空白引导组

### UT-PA-01 引导卡片显示

- **Given** `tables.is_empty()`
- **When** 渲染画布
- **Then** `[data-testid="canvas-empty-guide"]` 可见

### UT-PA-02 引导建表

- **Given** 引导卡片可见
- **When** 点击「创建第一张表」
- **Then** `tables.len() == 1`；引导卡片不可见；Inspector 展开

### UT-PA-03 选中态同步

- **Given** 画布有表 `users`
- **When** 单击表头
- **Then** `selected_table_id == users.id`；Inspector 标题含 `users`

### UT-PA-04 选字段 Inspector

- **Given** 表已选
- **When** 单击字段 `email`
- **Then** Inspector 显示 `InspectorField`

### UT-PA-05 双击空白折叠

- **Given** Inspector 展开
- **When** 双击画布空白
- **Then** Inspector 折叠；`SelectionState.kind == none`

### UT-PA-06 栅格 CSS

- **Given** `styles.css` 已加载
- **When** grep `.cdb-main`
- **Then** 含 `grid-template-columns: 48px 1fr auto`

## 6. 场景测试（ST）

### ST-PA-01 空白引导全流程

1. 打开 `/editor/new-id`（0 表）
2. 断言 `canvas-empty-guide` 可见
3. 点击「创建第一张表」
4. 断言引导消失；表 `Table_1` 可见；Inspector 展开
5. 重命名表为 `users`；添加字段 `email`
6. 等待自动保存；断言 `save-state` 含「已保存」

### ST-PA-02 布局回归（smoke 兼容）

1. 打开编辑器（HP-01 路径）
2. 断言无 `cdb-modal-overlay`（继承 UT-FIX-01）
3. 断言 `app-bar` + `tool-rail` + `editor-canvas` 均可见
4. 断言**不**存在 `tab-tables`（左栏 Tab 已移除）
5. Tool Rail 建表 → 保存（HP-02 路径适配）

### ST-IN-01 Inspector 编辑保存

1. 建表 `orders`
2. Inspector 添加字段 `total` / `DECIMAL`
3. 勾选非空
4. 触发保存；断言 revision ≥ 1

## 7. OpenLogos Reporter 映射

| TC ID | reporter case_id | 批次 |
|-------|------------------|------|
| UT-AB-01..05 | `UT-AB-*` | Phase A code |
| UT-TR-01..04 | `UT-TR-*` | Phase A code |
| UT-IN-01..05 | `UT-IN-*` | Phase A code |
| UT-PA-01..06 | `UT-PA-*` | Phase A code |
| ST-PA-01..02 | `ST-PA-*` | Phase A e2e |

## 8. 搁置用例（Phase D 恢复）

| 原 TC ID | 原因 |
|----------|------|
| UT-SP-09 | 左栏 Tab 移除 |
| UT-SP-10 | 全局搜索移至 Command Palette |
