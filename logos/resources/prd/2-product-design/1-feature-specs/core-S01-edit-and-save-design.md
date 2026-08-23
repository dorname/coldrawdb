# S01：编辑并保存图表 — 交互设计

> 模块：core | 场景：S01 | 原型：`core-01-editor-prototype.html`
> 参考：drawdb `origin/main` → `Workspace.jsx` / `ControlPanel.jsx` / `EditorCanvas/Table.jsx`
> Phase 1 输入：`core-04-scenario-detail.md` §S01

## 0. 现行文档与原型基线

> 模块：core | 场景：S01 | 原型：`core-01-editor-prototype.html`（唯一现行主原型）
> Phase 1 输入：`core-04-scenario-detail.md` §S01
> 页面上下文：默认在 `room-editor` 内编辑；自动保存与 revision 反馈对齐 AppBar

| 项 | 结论 |
|---|---|
| 产品类型 | Web 应用（浏览器端 ER 编辑器） |
| 原型形式 | 单文件可交互 HTML（内联 CSS/JS，不断网） |
| 对齐锚点 | 主原型 AppBar / ToolRail / Canvas / Inspector / StatusBar |
| 共享样式 | **不引用** `core-00-prototype-shared.css`；样式以内联主原型为准 |
| 生产边界 | 保存语义以 REST PUT + revision 为准；协作房间内并发合并见 S05，不得误用 409 模态 |

## 1. 产品类型与原型策略

| 项 | 结论 |
|---|---|
| 产品类型 | Web 应用（浏览器端 ER 编辑器） |
| 原型形式 | 可交互 HTML（`core-01-editor-prototype.html`） |
| 对齐锚点 | drawdb main 顶层布局 + coldrawdb V2 AppBar/ToolRail/Inspector 栅格 |
| 共享样式 | `core-00-prototype-shared.css`（token / 图标 / focus / reduced-motion） |

## 2. 涉及页面与区域

| 区域 | coldrawdb 原型锚点 |
|---|---|
| 顶栏 | `[data-testid="app-bar"]` |
| 左侧工具 | `[data-testid="tool-rail"]` |
| 画布 | `[data-testid="editor-canvas"]` |
| 右侧 Inspector | `[data-testid="inspector"]` |
| 保存状态 | `[data-testid="save-state"]` / `[data-testid="revision-display"]` |
| 409 冲突（非 OT 路径） | `[data-testid="modal-conflict"]`（若生产仍保留） |
| Command Palette | `[data-testid="command-palette"]` · ⌘K / Ctrl+K |
| Code View | `[data-testid="code-view-modal"]` · `[data-testid="btn-code-view"]` |
| 房间上下文 | `[data-testid="room-badge"]`（可返回 rooms） |

## 3. 交互流程

### 3.1 创建表并自动保存（主路径）

1. 用户在 Tool Rail 点击「新建表」图标或在 Inspector「表」Tab 底部「+」
2. 画布 `(100, 100)` 出现新表卡片（对齐 drawdb `Table.jsx` 默认坐标）
3. 用户双击表名进入编辑；Inspector 同步显示字段列表
4. 任意 diagram 变更后 **1s debounce** → AppBar 保存状态：`已保存` → `保存中…` → `已保存`
5. `revision-display` 递增（如 `rev: 5` → `rev: 6`）

### 3.2 Inspector 字段编辑

1. 用户单击画布表 `users` → 表高亮（`.cdb-is-selected`）+ Inspector 列表项激活
2. Inspector 显示字段编辑器 `[data-testid="field-editor"]`
3. 用户修改字段类型 / NOT NULL / UNIQUE 勾选
4. 触发 debounce 保存（同 3.1 步骤 4–5）

### 3.3 409 revision 冲突

仅适用于**非 OT 协作合并**的快照冲突路径。房间协作模式（S05）下服务器已合并的并发操作须 Toast/Activity 反馈，**禁止**弹出 S01 409 冲突模态。

### 3.4 网络失败重试

1. PUT 失败 → `[data-testid="save-state"]` 变红，文案「保存失败（离线）」
2. 指数退避 3s / 6s / 12s（封顶 30s）自动重试
3. 恢复后 revision 推进，状态回「已保存」

### 3.5 Command Palette（E4）

1. 用户按 `Ctrl+K`（macOS `Cmd+K`）或点击 StatusBar 提示
2. 浮层 `[data-testid="command-palette"]` 打开，焦点在搜索框
3. 输入过滤表/关系；↑/↓ 导航，Enter 选中 → 画布聚焦对象并关闭
4. Esc 关闭浮层

### 3.6 Code View（E4）

1. 用户点击 AppBar `[data-testid="btn-code-view"]`
2. 主区域切换为 Code View（ToolRail / Inspector 隐藏），Tab 切换 SQL / DBML / JSON
3. 右下角「复制」写入剪贴板并 toast
4. 再次点击「返回」或 Esc 回到 Canvas 模式

### 3.0 进入路径

1. 用户经 `auth → rooms` 进入 `room-editor`（或兼容的已打开 diagram）
2. Viewer 角色下写工具禁用，不触发 PUT
3. Owner/Editor 的画布变更进入 debounce 保存；协作模式下另见 S05 OT/ack

## 4. 验收条件（交互级）

##### 正常：创建表后自动保存

- **GIVEN** 用户在编辑器，`revision-display` 显示 `rev: 5`，保存状态为「已保存」
- **WHEN** 用户点击画布表 `users`（触发编辑态）并等待 1s
- **THEN**
  - `[data-testid="save-state"]` 先显示「保存中…」（`.is-saving`）
  - 1s 后变为「已保存」，`revision-display` 变为 `rev: 6`

##### 正常：Inspector 与画布选中同步

- **GIVEN** Inspector「表」Tab 激活，列表含 `users` / `posts`
- **WHEN** 用户点击 `[data-testid="table-posts"]`
- **THEN**
  - 画布 `posts` 表获得 `.cdb-is-selected`
  - Inspector 列表 `posts` 项获得 `.cdb-is-active`
  - `[data-testid="field-editor"]` 更新为 posts 上下文（实装时）

##### 正常：Command Palette 跳转

- **GIVEN** 用户在 Canvas 模式
- **WHEN** 用户按 `Ctrl+K`，在搜索框输入 `posts` 并 Enter
- **THEN**
  - `[data-testid="command-palette"]` 关闭
  - 画布 `posts` 表获得 `.cdb-is-selected`

##### 正常：Code View 复制

- **GIVEN** Code View 已打开，SQL Tab 激活
- **WHEN** 用户点击 `[data-testid="btn-copy-code"]`
- **THEN** 剪贴板含 SQL 文本，toast 显示「已复制到剪贴板」

##### 异常：409 冲突对话框

- **GIVEN** 本地 revision 与服务端不一致
- **WHEN** 保存触发 409（原型：双击 diagram 标题）
- **THEN**
  - `[data-testid="modal-conflict"]` 打开
  - 可见 `[data-testid="conflict-reload"]` / `[data-testid="conflict-force"]` / Cancel
  - ESC 或 Cancel 关闭模态，遮罩从 DOM 移除

##### 异常：离线保存失败

- **GIVEN** 网络不可用
- **WHEN** debounce 触发 PUT
- **THEN**
  - 保存状态变红 + 「保存失败（离线）」
  - 网络恢复后自动重试并成功，revision 递增

## 5. 原型操作指南

在浏览器打开 `logos/resources/prd/2-product-design/2-page-design/core-01-editor-prototype.html`（需同目录 `core-00-prototype-shared.css`）：

| 操作 | 预期 |
|---|---|
| 点击画布/列表中的表 | 选中同步 + 模拟 1s 自动保存 |
| 双击 diagram 标题 | 打开 409 冲突模态 |
| 点击「导入」 | IO 抽屉（导入模式）从右侧 400px 滑出 |
| 点击「导出」 | IO 抽屉（导出模式）显示 SQL/DBML/JSON 预览 + 复制 |
| `Ctrl+K` | 打开 Command Palette，Enter 聚焦表 |
| 点击「代码」 | 进入 Code View；Esc 返回 Canvas |

## 命令与代码视图（与主原型一致）

- Command Palette：搜索表/命令；Esc 关闭；关闭后焦点回画布
- Code View：SQL / DBML / JSON 分段切换；只读生成；复制；返回画布
- 二者不得破坏自动保存状态机；只读代码视图不产生 diagram mutation
