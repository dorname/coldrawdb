# Delta: core-03-bridge-io.md

> 提案：feat-db-connect-import-and-ddl-realdb-verify

## MODIFIED — 1. Bridge API 端点

替换为（端点 5 → 6 个，新增连接数据库导入）：

```markdown
## 1. Bridge API 端点（6 个）

| 端点 | 方法 | 用途 | 状态码 |
|---|---|---|---|
| `/api/v1/bridge/import/local` | POST | 从 SQL/DBML/JSON 内容导入 diagram | 201 / 400 |
| `/api/v1/bridge/import/local/logs` | GET | 列出所有导入日志 | 200 |
| `/api/v1/bridge/import/local/retry/{id}` | POST | 重试失败的导入任务 | 200 / 404 / 409 |
| `/api/v1/bridge/config` | GET | 读取桥接配置（默认引擎 / 缩进 / 命名风格） | 200 |
| `/api/v1/bridge/config` | PUT | 更新桥接配置 | 200 / 400 |
| `/api/v1/bridge/import/connect` | POST | 连接真实数据库 introspect schema 并返回方言 DDL（feat-db-connect-import-and-ddl-realdb-verify） | 200 / 400 / 502 |

完整 OpenAPI 规格见 `deltas/api/bridge.yaml`。
```

## ADDED — 11. 连接数据库导入（V2 增量）

> 提案：feat-db-connect-import-and-ddl-realdb-verify

### 11.1 端点契约

`POST /api/v1/bridge/import/connect`，挂既有 auth 中间件（Bearer）。

请求：

```json
{ "engine": "sqlite | postgres", "source": "<SQLite 文件绝对路径 | PG 连接串>" }
```

响应 200（`ApiResp` 包裹）：

```json
{ "code": 0, "data": { "engine": "sqlite", "tables": 3, "ddl": "CREATE TABLE ..." } }
```

- `ddl`：按目标引擎方言生成的 `CREATE TABLE` 文本（含列级约束与 FOREIGN KEY 子句），供前端**复用本地合并管道**（`parse_sql_import_tables` → 摘要 → `merge_import_into_store`）直接消费。
- `tables`：introspect 到的业务表数量；为 0 时 `ddl` 为空串，前端按「未检测到数据表」提示（走既有解析错误路径，不新增错误码）。

错误语义：

| 状态码 | 场景 | 前端文案 |
|---|---|---|
| 400 | `engine` 非 `sqlite`/`postgres`，或 `source` 为空 | 「不支持的数据库引擎」/「请填写连接信息」 |
| 502 | 连接失败（文件不存在 / 无权限 / PG 不可达 / 认证失败） | 「连接数据库失败，请检查连接信息」 |

### 11.2 Introspection 口径

- **SQLite**（sqlx-sqlite）：`SELECT name FROM sqlite_master WHERE type='table' AND name NOT LIKE 'sqlite_%'`；逐表 `PRAGMA table_info(t)`（列名/类型/notnull/dflt_value/pk）+ `PRAGMA foreign_key_list(t)`（FK）。
- **PostgreSQL**（sqlx-postgres）：`information_schema.tables`（`table_type='BASE TABLE'`，排除 `pg_catalog`/`information_schema`）；`information_schema.columns` + `information_schema.table_constraints`/`key_column_usage`/`constraint_column_usage` 解析 PK/FK。
- 视图/索引/触发器/序列不 introspect（与 §3.2 V1 解析器边界一致）。
- 生成 DDL 前先按 FK 依赖拓扑排序失败时按字典序兜底（不保证引用顺序，导入解析器对前向引用 tolerant）。

### 11.3 DDL 生成规则

- 内部 IR：`IntrospectedTable { name, columns: [{ name, type, not_null, default, pk }], fks: [{ column, ref_table, ref_column }] }`。
- 渲染为对应引擎方言：SQLite 原样类型 + `PRIMARY KEY`/`NOT NULL`/`DEFAULT`；PG 保留原始类型名（含 `character varying(n)` → `VARCHAR(n)` 规范化）、联合主键渲染表级 `PRIMARY KEY(a, b)`、FK 渲染表级 `FOREIGN KEY (c) REFERENCES t(c2)`。
- 标识符按需加双引号（含保留字/大小写混合）。

### 11.4 安全边界

- `source` 仅由后端进程本机解析（SQLite 路径为服务器本地文件，PG 连接串直连），不在前端持久化、不写导入日志表（连接串可能含口令）。
- 端点需登录态；不校验路径白名单（桌面/自托管场景，服务端即用户本机）。

## MODIFIED — 10. 测试用例 ID 索引

表格追加行（详细步骤见 `core-PC-import-export-test-cases.md`）：

```markdown
| UT-PC-11 | SQLite introspection：临时真实 SQLite 库建表 → 端点生成 DDL 含表/列/PK/FK |
| UT-PC-12 | introspection DDL 渲染纯函数 + 错误映射（400 不支持引擎 / 502 连接失败 / 无表 tables=0） |
| UT-PC-13 | PG introspection：嵌入式 PG 起库建表 → 端点生成 DDL 含表/列/联合主键/FK |
| UT-PC-14 | 导出 DDL 真实库执行验证：`export_diagram_sql` 输出在嵌入式 PG 与 SQLite 上执行成功 |
| UT-PC-15 | 前端锚点：ImportDrawer「数据库」来源 testid 与接线 |
| ST-PC-04 | e2e：ImportDrawer 数据库来源 → mock 端点返回 DDL → 本地合并落账 |
```
