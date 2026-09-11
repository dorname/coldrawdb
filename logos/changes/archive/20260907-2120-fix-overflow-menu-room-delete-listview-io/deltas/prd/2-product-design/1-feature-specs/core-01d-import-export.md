# Delta: core-01d-import-export.md

> 提案：fix-overflow-menu-room-delete-listview-io

## MODIFIED — §4.5 入口汇总

| 入口 | testid | 行为 |
|------|--------|------|
| AppBar | `btn-import` | 打开 ImportDrawer |
| File 菜单 | `cdb-menu-import` | 打开 ImportDrawer（不再开 Import 模态） |
| EmptyGuide | `guide-import-sql` | 打开 ImportDrawer |
| ListView 工具条 | `list-btn-import` | 打开 ImportDrawer（本地合并管道，见 §4.4；只读禁用） |
| ListView 工具条 | `list-btn-export` | 打开 ExportDrawer（只读可用） |

> ListView 入口（fix-overflow-menu-room-delete-listview-io）：列表视图工具条右侧「导入」「导出」按钮打开与画布态相同的 IO 抽屉；导入提交走 §4.4 本地合并（不跳转、不调 bridge），合并后 ListView 表树刷新；交互细节见 `core-04` §10.5。
