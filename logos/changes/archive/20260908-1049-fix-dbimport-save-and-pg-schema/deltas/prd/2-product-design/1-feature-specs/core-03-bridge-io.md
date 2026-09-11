# Delta: core-03-bridge-io.md

> 提案：fix-dbimport-save-and-pg-schema

## MODIFIED — §13.1 端点契约（请求体）

请求示例改为：

```json
{ "engine": "sqlite | postgres", "source": "<SQLite 文件绝对路径 | PG 连接串>", "schema": "public" }
```

- `schema`：**可选，仅 `engine=postgres` 时生效**，缺省按 `public` 处理；`engine=sqlite` 时忽略。用于把 introspection 限定在单个 schema 内。

## MODIFIED — §13.2 Introspection 口径（PostgreSQL 段）

PostgreSQL 口径改为：

- **PostgreSQL**（sqlx-postgres）：表/列/PK/FK 四个查询全部按**请求指定的单个 schema** 精确过滤——`information_schema.tables` / `information_schema.columns` 使用 `table_schema = $2`，`pg_catalog` 系 PK/FK 查询使用 `ns.nspname = $2`（`$2` 为请求 `schema`，缺省 `public`）。不再使用「排除系统 schema」的宽口径，不同 schema 的同名表不会互相串列、串 PK/FK。
