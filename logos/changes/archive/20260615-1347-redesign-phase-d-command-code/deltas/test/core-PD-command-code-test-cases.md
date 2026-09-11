# core-PD-command-code-test-cases.md

> 模块：core | 提案：redesign-phase-d-command-code
> 路径：`logos/resources/test/core-PD-command-code-test-cases.md`
> 最后更新：2026-06-14

## Phase D Command Palette + Code View 测试用例

| TC ID | Given | When | Then |
|-------|-------|------|------|
| UT-PD-01 | store 含 2 表 + Action | `build_palette_items(store)` | len >= 4；含 Table 与 `action-create-table` |
| UT-PD-02 | items 含 users/orders | `filter_palette_items(items, "user")` | 仅 users 匹配 |
| UT-PD-03 | keydown Ctrl+K | `is_palette_shortcut(ev)` | true；Cmd+K on mac |
| UT-PD-04 | store 含 1 表 | Code 视图 SQL 预览 | 含 `CREATE TABLE` |
| UT-PD-05 | view_mode=Code | 渲染 AppRoot class | `.cdb-main` 含 `cdb-is-code-view` |
| UT-PD-06 | 源码检查 | `btn-code-view` | 非 disabled；含 testid |
| UT-PD-07 | store 六类对象各 1 | `build_palette_items` | 6 kind 均出现（替代 UT-SP-09） |
| UT-PD-08 | tables+areas 含 "user" | filter query "user" | 跨类均过滤（替代 UT-SP-10） |
| ST-PD-01 | 编辑器已加载 1 表 | Ctrl+K → 输入表名 → Enter | Inspector h3 含表名 |

### 回归更新

| TC ID | 变更 |
|-------|------|
| UT-SP-09 | 搁置 → UT-PD-07 覆盖 |
| UT-SP-10 | 搁置 → UT-PD-08 覆盖 |

### 不在范围

- 代码视图双向编辑 Apply
- Palette 主题/批量命令
- Monaco 语法高亮
