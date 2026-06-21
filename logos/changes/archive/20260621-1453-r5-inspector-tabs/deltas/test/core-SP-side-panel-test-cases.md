## MODIFIED — UT-SP-09 6 业务 Tab 切换

**merge 时替换** UT-SP-09 标题与断言为：

### UT-SP-09 8 Tab 图标栏切换（R5）

- **Given** Inspector 已展开
- **When** 检查 Tab 栏
- **Then**
  - 存在 `.cdb-tabs--icon-grid`
  - `data-testid="tab-tables"`、`tab-areas`、`tab-enums`、`tab-notes`、`tab-relationships`、`tab-types`、`tab-issues`、**`tab-fields`** 全部存在
  - 每个 Tab 含 `title` 属性（Tooltip 文案）
  - 点击 Tab A→B→C 验证 `.cdb-is-active` 与内容区切换
  - **不存在** `.cdb-side-panel--right` 45% 分割容器
