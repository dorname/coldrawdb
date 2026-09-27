# Delta — core-03-bridge-io.md（fix-open-issues-26-33 / #29）

> 模块：core | 提案：fix-open-issues-26-33
> 目标：import/connect 响应输出 references（fks 展开），消除「数据库导入有注释无关系线」；SQL 导入注释解析加固 schema 限定名，消除「SQL 导入有关系无注释」。

## MODIFIED — 3.2 解析器能力（V1）

| 能力 | 状态 | 说明 |
|---|---|---|
| `CREATE TABLE` | ✅ | 主流程 |
| `PRIMARY KEY` / `NOT NULL` / `UNIQUE` | ✅ | 列级约束 |
| `AUTO_INCREMENT` / `SERIAL` / `IDENTITY` | ✅ | 自增识别 |
| `FOREIGN KEY ... REFERENCES` | ✅ | 含 ON UPDATE/DELETE；支持 schema 限定目标（`REFERENCES public.t(c)` / `REFERENCES "app"."t"(c)`），按最后一段标识符匹配已解析表 |
| `CHECK` | ⚠️ 部分 | 仅简单表达式 |
| `DEFAULT` | ✅ | 字面量；NOW()/CURRENT_TIMESTAMP 字符串保留 |
| `COMMENT` | ✅ MySQL / PG | MySQL 行内 `COMMENT '...'`；PG `COMMENT ON TABLE` / `COMMENT ON COLUMN`（fix-canvas-zoom-invite-comment-resize 起）。**本变更加固**：支持 schema 限定与引号混写目标（`COMMENT ON TABLE public.users IS ...` / `COMMENT ON COLUMN "app"."orders"."id" IS ...`），目标匹配取最后一段标识符（去引号）；目标表/列不存在时跳过该条注释（不报错） |
| `CREATE TYPE ... AS ENUM` (PostgreSQL) | ✅ | 转为内部 Enum |
| `CREATE TYPE ... AS OBJECT` (OracleSQL) | ✅ | 转为内部 CustomType |
| `CREATE INDEX` | ❌ | V1 导入时丢弃 |
| `TRIGGER` / `SEQUENCE` | ❌ | V1 导入时丢弃 |
| `VIEW` / `PROCEDURE` / `FUNCTION` | ❌ | V1 不支持 |

## MODIFIED — 13.1 端点契约

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
    ],
    "references": [
      { "table": "orders", "column": "user_id", "ref_table": "users", "ref_column": "id", "on_delete": "CASCADE", "on_update": "" }
    ]
  },
  "request_id": "..."
}
```

- `tables`：**结构化表数组**，元素与 persistence / JSON 导入格式同构（`name` / `comment` / `fields[{ name, type, primary, unique, not_null, increment, default, comment }]`），供前端**复用 JSON 导入解析路径**（`parse_json_import_tables` 或轻适配）直接消费，**不再经 DDL 文本解析**。
- `references`（**本变更新增**）：**名址（name-based）关系数组**，由 introspect IR `fks` 展开（规则见 §13.3），元素为 `{ table, column, ref_table, ref_column, on_delete, on_update }`：
  - 复合 FK 按列对展开为多条（`columns[i]` ↔ `ref_columns[i]`），与 SQL 导入「每列对一条关系线」口径一致；
  - `on_delete` / `on_update` 无规则时为空串；
  - 指向未 introspect 到的表/列的 FK 不输出（不产悬空引用）；
  - 无 FK 时输出空数组（键始终存在，前端无需判缺省）。
- `table_count`：introspect 到的业务表数量（= `tables` 长度）；为 0 时 `tables` 为空数组，前端按「未检测到数据表」提示（走既有解析错误路径，不新增错误码）。
- 字段口径：`type` 为方言类型整串（PG 侧做 `character varying(n)` → `VARCHAR(n)` 等规范化，同原 §13.3）；`primary` 含联合主键各列；`default` 无默认值时为空串；`comment` 为表/列注释（见 §13.2，SQLite 恒为空串——引擎限制）。
- 原 `ddl` 字段与整数 `tables` 计数**移除**（被 `tables` 数组 / `table_count` 取代）；`render_introspect_ddl` 仅保留为内部/测试辅助或一并移除，不再出现在响应路径。

错误语义：

| 状态码 | 场景 | 前端文案 |
|---|---|---|
| 400 | `engine` 非 `sqlite`/`postgres`，或 `source` 为空 | 「不支持的数据库引擎」/「请填写连接信息」 |
| 502 | 连接失败（文件不存在 / 无权限 / PG 不可达 / 认证失败） | 「连接数据库失败，请检查连接信息」 |

## MODIFIED — 13.3 内部 IR 与响应生成规则

- 内部 IR（新增 `comment` 字段）：

```rust
IntrospectedTable { name, comment, columns: [...], fks: [...] }
IntrospectedColumn { name, col_type, not_null, default, pk, comment }
IntrospectedFk { columns, ref_table, ref_columns, on_delete, on_update }
```

- 响应序列化：IR → `tables` JSON。字段映射：`col_type` → `type`；`pk` → `primary`（联合主键各列均 `true`）；`default=None` → 空串；`comment` 原样透传。
- **references 序列化（本变更新增）**：IR `fks` → `references` JSON 数组，规则：
  1. 每条 `IntrospectedFk` 按 `columns.len()` 展开为 `{ table: 当前表名, column: columns[i], ref_table, ref_column: ref_columns[i], on_delete, on_update }`；
  2. `columns` 与 `ref_columns` 长度不一致时按较短者截断并跳过残缺列对；
  3. `ref_table` 不在本次 introspect 结果内、或 `column` / `ref_column` 不在对应表列集合内 → 跳过该条（不产悬空引用）；
  4. 输出顺序：按 `tables` 排序后的表序 + FK 收集顺序，保证确定性（快照断言可比对）。
- PG 类型规范化保留：原始类型名（含 `character varying(n)` → `VARCHAR(n)`）。
- 标识符**不**在响应中加引号（结构化 JSON 无 SQL 注入面；渲染层才需引号处理，原 DDL 渲染路径移除后该顾虑消失）。
