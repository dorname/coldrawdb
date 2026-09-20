# Delta — core-01d-import-export.md（修改）

> 模块：core | 提案：fix-remote-github-issues-7-18
> 关联 issue：#7（导入拖放区域无效）、#8（导入不支持 .ddl）

## MODIFIED — 4.3 文件拖放

### 4.3 文件拖放

- **接受扩展名**：`.sql` / `.ddl` / `.dbml` / `.json`。`.ddl` 视为 SQL DDL 文本，与 `.sql` 同路径解析（format=`sql`，引擎选择规则不变）。
- **交互合同（生产必须实现；issue #7 根因：dropzone 曾仅为展示 `<div>`，无任何事件绑定）**：

| 事件 | 行为 |
|---|---|
| `dragenter` / `dragover` | `preventDefault()` 允许放置；dropzone 附加 `is-dragover` 高亮态 |
| `dragleave` | 移除 `is-dragover` |
| `drop` | `preventDefault()`；取 `DataTransfer.files[0]`；扩展名不在白名单 → inline 提示「仅支持 .sql / .ddl / .dbml / .json」，不写内容；否则 `File.text()` 读取文本写入导入内容 signal，触发解析摘要更新 |
| 点击 dropzone | 触发隐藏 `<input type="file" accept=".sql,.ddl,.dbml,.json" data-testid="io-file-input">` 作为兜底；选中后走与 drop 相同的读取路径 |

- **扩展名 → format Tab 映射**：`.sql` / `.ddl` → `sql`；`.dbml` → `dbml`；`.json` → `json`；拖入后自动切换到对应 Tab。
- 大小上限：读取 `bridge/config` 的 `maxImportSizeKb`（V1 可硬编码 5120 KB，与 bridge 默认一致）；超限 inline 报错且不写入内容。
- 拖放仅读取本地文件文本，不上传；解析仍走 §4.4 本地口径。
- Viewer / 只读：ImportDrawer 整体不可达（入口禁用），dropzone 不单独授权。
- testid 增量：`io-dropzone`（既有，补上事件后语义不变）、`io-file-input`（新增隐藏文件选择器）。

## MODIFIED — 8. 测试 ID 索引

| TC ID | 描述 |
|-------|------|
| UT-PC-01 | `parse_sql_statements` 驱动导入摘要 |
| UT-PC-02 | `export_diagram_sql` 非空 diagram 输出含 `CREATE TABLE` |
| UT-PC-03 | `export_diagram_dbml` 含 `Table` 块 |
| UT-PC-04 | `IoDrawerKind` 互斥：开 Import 折叠 Inspector |
| UT-PC-05 | `count_dbml_tables` 纯函数 |
| ST-PC-01 | e2e：btn-import → 粘贴 SQL → 解析摘要可见 |
| UT-PC-11 | 后端 SQLite introspection → 方言 DDL（真实临时库） |
| UT-PC-12 | 后端 introspection DDL 渲染纯函数 + 错误映射（400/502/tables=0） |
| UT-PC-13 | 后端 PG introspection（嵌入式 PG） |
| UT-PC-14 | 导出 DDL 真实库执行验证（嵌入式 PG + SQLite） |
| UT-PC-15 | 前端锚点：ImportDrawer 数据库来源 testid 与接线 |
| ST-PC-04 | e2e：数据库来源连接导入 → 本地合并落账 |
| UT-PC-16 | 后端导出执行：SQLite 真实库执行 DDL（建表数/语句数） |
| UT-PC-17 | 后端导出执行：嵌入式 PG + 错误映射（400/502） |
| UT-PC-18 | 前端锚点：ExportDrawer 导出到数据库 testid 与接线 |
| UT-PC-19 | 样式锚点：`.cdb-form-select` / `.cdb-select` 视觉条款落地 |
| UT-PC-31 | `drop` 事件读取文件文本写入导入内容并触发解析摘要 |
| UT-PC-32 | 扩展名白名单与 Tab 映射：`.ddl` 归 SQL；非白名单拒绝 |
| UT-PC-33 | 点击 dropzone 触发隐藏 `io-file-input`（accept 含 `.ddl`） |
| ST-PC-09 | e2e：拖入 `.ddl` 文件 → textarea 填充 → 摘要非 0 → 导入到画布成功 |

详细步骤见 `core-PC-import-export-test-cases.md`。
