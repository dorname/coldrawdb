# Delta — core-03-bridge-io.md

## ADDED — §8 前端 IO 抽屉对接（Phase C）

> merge 时在 §7 或文档末尾追加。

### 8.1 导入（服务端）

| 步骤 | API | 说明 |
|------|-----|------|
| 提交 | `POST /api/v1/bridge/import/local` | body: `{ format, content, engine?, title? }` |
| 成功 | 响应 `diagramId` | 前端跳转 `/editor/{id}` |
| 失败 | 400 + `message` | 抽屉内显示，不关闭抽屉 |

`ImportDrawer` 通过 `editor_data_access::DiagramClient::import_local()` 封装。

### 8.2 导出（客户端）

Phase C **不新增** bridge export 端点。导出预览由前端纯函数生成：

| format | 输入 | 输出 |
|--------|------|------|
| `sql` | `EditorStore` + `engine` | `CREATE TABLE ...` 字符串 |
| `dbml` | `EditorStore` | DBML 文本 |
| `json` | `Diagram` JSON | pretty JSON |

与 §2「7 引擎 SQL 导出」对齐的子集：V1 抽屉导出实现 generic/mysql 最小子集即可；其余引擎通过 engine 参数切换标识符占位。

### 8.3 配置读取（可选）

- `GET /api/v1/bridge/config` → 填充默认 `engine`、`maxImportSizeKb`
- V1 可 fallback：`engine=generic`，`maxImportSizeKb=5120`

## MODIFIED — §3.1 SQL 导入流程（补充前端路径）

> 在 §3.1 流程列表增加一条。

0. **（Phase C）** 用户在 ImportDrawer 粘贴 SQL → 客户端 `parse_sql_statements` 预览 → 提交 bridge import
