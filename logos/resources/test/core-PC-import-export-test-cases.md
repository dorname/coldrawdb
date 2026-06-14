# core-PC-import-export-test-cases.md

> 模块：core | 提案：redesign-phase-c-import-export
> 路径：`logos/resources/test/core-PC-import-export-test-cases.md`
> 最后更新：2026-06-14

## Phase C 导入/导出抽屉测试用例

| TC ID | Given | When | Then |
|-------|-------|------|------|
| UT-PC-01 | SQL 含 2 条 CREATE | `parse_sql_statements` | `Ok(vec![..])` len==2；摘要显示「2 条语句」 |
| UT-PC-02 | store 含 1 张表 | `export_diagram_sql(store, "generic")` | 输出含 `CREATE TABLE` 与表名 |
| UT-PC-03 | store 含表+关系 | `export_diagram_dbml(store)` | 输出含 `Table` 与 `ref:` |
| UT-PC-04 | Inspector 展开 | `open_import_drawer()` | `inspector_open==false` 且 `io_drawer==Import` |
| UT-PC-05 | DBML 含 2 个 Table 块 | `count_dbml_tables(text)` | 返回 2 |
| UT-PC-06 | 零表画布 | 点击 `guide-import-sql` | `import-drawer` 可见 |
| ST-PC-01 | 编辑器已加载 | AppBar 导入 → 粘贴 SQL → 提交 | 解析摘要可见；bridge 返回 diagramId |

### 回归更新

| TC ID | 变更 |
|-------|------|
| UT-AB-04 | Phase C：`btn-import` **enabled**（替换 Phase A disabled 断言） |

### 不在范围

- SQL/DBML 全屏代码视图（Phase D）
- 导入任务 logs/retry UI
- Mermaid / PNG 导出
