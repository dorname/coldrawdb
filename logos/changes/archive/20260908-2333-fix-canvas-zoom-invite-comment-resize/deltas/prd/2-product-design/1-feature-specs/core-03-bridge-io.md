# Delta: core-03-bridge-io.md

> 模块：core | 提案：fix-canvas-zoom-invite-comment-resize（问题3：数据库导入注释丢失 + 注释展示）

背景：`/bridge/import/connect` 原响应为 `{ engine, tables, ddl }`——后端把 introspection IR 渲染成方言 DDL 文本，前端再经 `parse_sql_import_tables` 文本解析，表/列注释在「查询 → IR → 渲染 → 解析」四个环节全部丢失。修复：响应改为**结构化 tables JSON**（与 persistence JSON 同构，含 comment），前端复用 JSON 导入路径，不再经 DDL 文本中转。

## MODIFIED — §1 Bridge API 端点（import/connect 行）

| 端点 | 方法 | 用途 | 状态码 |
|---|---|---|---|
| `/api/v1/bridge/import/connect` | POST | 连接真实数据库 introspect schema 并返回**结构化 tables JSON**（含表/列 comment；fix-canvas-zoom-invite-comment-resize 由 DDL 文本改为结构化直传） | 200 / 400 / 502 |

完整 OpenAPI 规格见 `deltas/api/bridge.yaml`。

## MODIFIED — §13 连接数据库导入（V2 增量）

> 提案：feat-db-connect-import-and-ddl-realdb-verify（fix-canvas-zoom-invite-comment-resize 改结构化 JSON 响应）

### 13.1 端点契约

`POST /api/v1/bridge/import/connect`，挂既有 auth 中间件（Bearer）。

请求：

```json
{ "engine": "sqlite | postgres", "source": "<SQLite 文件绝对路径 | PG 连接串>", "schema": "public" }
```

- `schema`（fix-dbimport-save-and-pg-schema）：**可选，仅 `engine=postgres` 时生效**，缺省按 `public` 处理；`engine=sqlite` 时忽略。用于把 introspection 限定在单个 schema 内。

响应 200（`ApiResp` 包裹）：

```json
{
  "code": 0,
  "data": {
    "engine": "postgres",
    "table_count": 2,
    "tables": [
      {
        "name": "users",
        "comment": "用户表",
        "fields": [
          { "name": "id", "type": "INTEGER", "primary": true, "not_null": true, "default": "", "comment": "" },
          { "name": "status", "type": "TEXT", "primary": false, "not_null": false, "default": "", "comment": "状态" }
        ]
      }
    ]
  },
  "request_id": "..."
}
```

- `tables`：**结构化表数组**，元素与 persistence / JSON 导入格式同构（`name` / `comment` / `fields[{ name, type, primary, unique, not_null, increment, default, comment }]`），供前端**复用 JSON 导入解析路径**（`parse_json_import_tables` 或轻适配）直接消费，**不再经 DDL 文本解析**。
- `table_count`：introspect 到的业务表数量（= `tables` 长度）；为 0 时 `tables` 为空数组，前端按「未检测到数据表」提示（走既有解析错误路径，不新增错误码）。
- 字段口径：`type` 为方言类型整串（PG 侧做 `character varying(n)` → `VARCHAR(n)` 等规范化，同原 §13.3）；`primary` 含联合主键各列；`default` 无默认值时为空串；`comment` 为表/列注释（见 §13.2，SQLite 恒为空串——引擎限制）。
- 原 `ddl` 字段与整数 `tables` 计数**移除**（被 `tables` 数组 / `table_count` 取代）；`render_introspect_ddl` 仅保留为内部/测试辅助或一并移除，不再出现在响应路径。

错误语义：

| 状态码 | 场景 | 前端文案 |
|---|---|---|
| 400 | `engine` 非 `sqlite`/`postgres`，或 `source` 为空 | 「不支持的数据库引擎」/「请填写连接信息」 |
| 502 | 连接失败（文件不存在 / 无权限 / PG 不可达 / 认证失败） | 「连接数据库失败，请检查连接信息」 |

### 13.2 Introspection 口径

- **SQLite**（sqlx-sqlite）：`SELECT name FROM sqlite_master WHERE type='table' AND name NOT LIKE 'sqlite_%'`；逐表 `PRAGMA table_info(t)`（列名/类型/notnull/dflt_value/pk）+ `PRAGMA foreign_key_list(t)`（FK）。**SQLite 无 COMMENT 语法，表/列 `comment` 恒为空串（引擎限制，非缺陷）**。
- **PostgreSQL**（sqlx-postgres）：表/列/PK/FK 四个查询全部按**请求指定的单个 schema** 精确过滤（fix-dbimport-save-and-pg-schema）——`information_schema.tables` / `information_schema.columns` 使用 `table_schema = $2`，`pg_catalog` 系 PK/FK 查询使用 `ns.nspname = $2`（`$2` 为请求 `schema`，缺省 `public`）。不再使用「排除系统 schema」的宽口径，不同 schema 的同名表不会互相串列、串 PK/FK。
  - **注释（fix-canvas-zoom-invite-comment-resize 新增）**：表注释经 `obj_description(c.oid, 'pg_class')` 取得（表查询 JOIN `pg_class` / `pg_namespace`）；列注释经 `col_description(c.oid, a.attnum)` 取得（列查询 JOIN `pg_attribute`）。无注释返回 NULL → 落空串。
- 视图/索引/触发器/序列不 introspect（与 §3.2 V1 解析器边界一致）。
- ~~生成 DDL 前先按 FK 依赖拓扑排序~~（结构化响应后不再需要渲染顺序保证；`tables` 按表名排序返回，前端合并管道对引用顺序 tolerant）。

### 13.3 内部 IR 与响应生成规则

- 内部 IR（新增 `comment` 字段）：

```rust
IntrospectedTable { name, comment, columns: [...], fks: [...] }
IntrospectedColumn { name, col_type, not_null, default, pk, comment }
IntrospectedFk { columns, ref_table, ref_columns }
```

- 响应序列化：IR → `tables` JSON。字段映射：`col_type` → `type`；`pk` → `primary`（联合主键各列均 `true`）；`default=None` → 空串；`comment` 原样透传。
- PG 类型规范化保留：原始类型名（含 `character varying(n)` → `VARCHAR(n)`）。
- 标识符**不**在响应中加引号（结构化 JSON 无 SQL 注入面；渲染层才需引号处理，原 DDL 渲染路径移除后该顾虑消失）。

### 13.4 安全边界

- `source` 仅由后端进程本机解析（SQLite 路径为服务器本地文件，PG 连接串直连），不在前端持久化、不写导入日志表（连接串可能含口令）。
- 端点需登录态；不校验路径白名单（桌面/自托管场景，服务端即用户本机）。
