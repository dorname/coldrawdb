# 桥接导入 / 导出规格（V1）

## 1. Bridge API 端点（6 个）

| 端点 | 方法 | 用途 | 状态码 |
|---|---|---|---|
| `/api/v1/bridge/import/local` | POST | 从 SQL/DBML/JSON 内容导入 diagram | 201 / 400 |
| `/api/v1/bridge/import/local/logs` | GET | 列出所有导入日志 | 200 |
| `/api/v1/bridge/import/local/retry/{id}` | POST | 重试失败的导入任务 | 200 / 404 / 409 |
| `/api/v1/bridge/config` | GET | 读取桥接配置（默认引擎 / 缩进 / 命名风格） | 200 |
| `/api/v1/bridge/config` | PUT | 更新桥接配置 | 200 / 400 |
| `/api/v1/bridge/import/connect` | POST | 连接真实数据库 introspect schema 并返回**结构化 tables JSON**（含表/列 comment；fix-canvas-zoom-invite-comment-resize 由 DDL 文本改为结构化直传） | 200 / 400 / 502 |

完整 OpenAPI 规格见 `deltas/api/bridge.yaml`。

## 2. 7 引擎 SQL 导出

### 2.1 引擎清单

| 引擎 | 标识符 | 特色能力 | 字段类型子集 |
|---|---|---|---|
| MySQL | `mysql` | unsigned types / ENUM inline | INT/BIGINT/VARCHAR/TEXT/DATE/DATETIME/TIMESTAMP/DECIMAL/FLOAT/DOUBLE/BLOB/JSON/BOOLEAN/ENUM |
| PostgreSQL | `postgresql` | JSONB/UUID/SERIAL/ENUM/ARRAY | + JSONB/UUID/SERIAL/ENUM/ARRAY |
| SQLite | `sqlite` | 弱类型 | INT/INTEGER/TEXT/REAL/BLOB/NUMERIC |
| MariaDB | `mariadb` | 同 MySQL + BOOLEAN | 同 MySQL |
| MSSQL | `mssql` | NVARCHAR/DATETIME2/BIT | INT/BIGINT/VARCHAR/NVARCHAR/TEXT/NTEXT/DATETIME/DATETIME2/BIT/DECIMAL/FLOAT/REAL |
| OracleSQL | `oraclesql` | VARCHAR2/NUMBER/CLOB/OBJECT | + VARCHAR2/NUMBER/CLOB/BLOB/DATE/TIMESTAMP |
| Generic | `generic` | 通用基线 | INT/VARCHAR/TEXT/DATE/BOOLEAN |

### 2.2 导出 SQL 结构

```sql
-- 表 A
CREATE TABLE A (
  id INT PRIMARY KEY,
  name VARCHAR(255) NOT NULL
);

-- 表 B（含外键）
CREATE TABLE B (
  id INT PRIMARY KEY,
  a_id INT NOT NULL,
  FOREIGN KEY (a_id) REFERENCES A(id) ON DELETE CASCADE
);
```

### 2.3 引擎特殊处理

| 引擎 | 特殊导出规则 |
|---|---|
| MySQL / MariaDB | `AUTO_INCREMENT` 关键字；ENUM 内联；ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 |
| PostgreSQL | `SERIAL` 替代 AUTO_INCREMENT；CREATE TYPE ... AS ENUM 独立 |
| SQLite | 弱类型 — `AUTOINCREMENT` 仅 INTEGER PRIMARY KEY；不支持 CHECK 完整子集 |
| MSSQL | `IDENTITY(1,1)` 替代 AUTO_INCREMENT；DATETIME2 替代 DATETIME |
| OracleSQL | `CREATE SEQUENCE` + `TRIGGER` 实现自增；CREATE TYPE ... AS OBJECT 实现自定义类型 |
| Generic | 不带任何方言；最简化基线 |

## 3. SQL 导入

### 3.1 流程

1. 客户端上传 `.sql` 文件 + 选择目标引擎
2. 服务端解析 → 内部 IR（与 Diagram 等价）→ 写入数据库
3. 返回 `task_id`（异步时）或同步结果

### 3.2 解析器能力（V1）

| 能力 | 状态 | 说明 |
|---|---|---|
| `CREATE TABLE` | ✅ | 主流程 |
| `PRIMARY KEY` / `NOT NULL` / `UNIQUE` | ✅ | 列级约束 |
| `AUTO_INCREMENT` / `SERIAL` / `IDENTITY` | ✅ | 自增识别 |
| `FOREIGN KEY ... REFERENCES` | ✅ | 含 ON UPDATE/DELETE |
| `CHECK` | ⚠️ 部分 | 仅简单表达式 |
| `DEFAULT` | ✅ | 字面量；NOW()/CURRENT_TIMESTAMP 字符串保留 |
| `COMMENT` | ⚠️ MySQL only | 引擎受限 |
| `CREATE TYPE ... AS ENUM` (PostgreSQL) | ✅ | 转为内部 Enum |
| `CREATE TYPE ... AS OBJECT` (OracleSQL) | ✅ | 转为内部 CustomType |
| `CREATE INDEX` | ❌ | V1 导入时丢弃 |
| `TRIGGER` / `SEQUENCE` | ❌ | V1 导入时丢弃 |
| `VIEW` / `PROCEDURE` / `FUNCTION` | ❌ | V1 不支持 |

## 4. DBML 导出 / 导入

### 4.1 导出

DBML 是一种 schema 描述语言（dbdiagram.io）。导出格式：

```dbml
Table A {
  id INT [pk]
  name VARCHAR(255) [not null]
}

Table B {
  id INT [pk]
  a_id INT [not null, ref: > A.id]
}
```

### 4.2 导入

DBML 解析器（同 SQL）→ 内部 IR → Diagram。

## 5. JSON 导入 / 导出

### 5.1 格式

drawdb 主分支导出的 JSON 格式（含 tables / references / areas / notes / enums / types）。coldrawdb V1 与之**部分兼容**（详见 `core-02-diagram-persistence.md` §5.2）。

### 5.2 导入路径

- 用户拖拽 `.json` 到编辑器 → 触发 `POST /api/v1/diagrams/import`
- 或通过 Bridge API 导入（`POST /api/v1/bridge/import/local`，body 含 content + format=json）

## 6. Bridge 配置

| 字段 | 类型 | 默认值 | 说明 |
|---|---|---|---|
| `default_engine` | enum | `mysql` | 7 选 1 |
| `sql_indent` | enum | `  `（两空格） | 缩进风格 |
| `sql_naming` | enum | `snake_case` | 标识符命名风格 |
| `sql_include_drop_table` | boolean | `false` | 是否在导出前 DROP TABLE |
| `sql_include_comments` | boolean | `true` | 是否包含 COMMENT |
| `max_import_size_kb` | number | `5120`（5 MB） | 导入文件大小限制 |

## 7. 导入日志

每条导入任务写入 `task` 表（11 张表之一）：

| 字段 | 说明 |
|---|---|
| `id` | 任务 id |
| `type` | `import_sql` / `import_dbml` / `import_json` |
| `status` | `pending` / `success` / `failed` |
| `message` | 错误信息（失败时） |
| `created_at` | 创建时间 |
| `completed_at` | 完成时间 |

通过 `GET /api/v1/bridge/import/local/logs` 列出（默认最近 50 条，可分页）。

## 8. 本地重试

`POST /api/v1/bridge/import/local/retry/{id}`：
- 仅 `status = failed` 的任务可重试
- 重试时重新读取原始上传内容
- 重置 `status = pending`，进入新一轮处理

## 9. 缺失能力（coldrawdb V1 未实现）

drawdb 主分支提供以下能力，**coldrawdb V1 不实现**（明确标注，避免误导）：

| 能力 | 状态 | 备注 |
|---|---|---|
| Mermaid 导出 | ❌ | 需新增 ER 渲染模块 |
| PNG 导出 | ❌ | 需 html2canvas 或服务端 puppeteer |
| PDF 导出 | ❌ | 依赖 PNG |
| ZIP 打包导出 | ❌ | 需多文件聚合 |
| SQL 模板变量替换 | ❌ | drawdb 有 `${schema}` 占位符 |
| 跨 diagram 引用导入 | ❌ | drawdb 支持 `diagram_link` 共享 |
| 实时导入进度推送 | ❌ | V1 仅同步或轮询 task 表 |
| 增量 SQL 导出（diff） | ❌ | V1 仅全量 |
| DBML ↔ SQL 双向同步 | ❌ | V1 仅单向（DBML → Diagram） |

## 10. 测试用例 ID 索引

| TC ID | 描述 |
|---|---|
| UT-B-01 | 5 表 20 字段 → 导出 MySQL SQL → 验证 AUTO_INCREMENT / ENGINE=InnoDB |
| UT-B-02 | 导出 PostgreSQL → 验证 SERIAL / CREATE TYPE ... AS ENUM |
| UT-B-03 | 导出 OracleSQL → 验证 CREATE SEQUENCE + CREATE TRIGGER |
| UT-B-04 | 导出 SQLite → 验证弱类型 |
| UT-B-05 | 导入 MySQL SQL → 反向生成 Diagram 字段 |
| UT-B-06 | 导入 drawdb JSON → 字段映射正确 |
| UT-B-07 | 导入失败 → task.status = failed，message 含错误 |
| UT-B-08 | 重试失败任务 → 重新进入 pending |
| UT-B-09 | 上传 > 5MB → 400 拒绝 |
| ST-B-01 | 端到端：编辑 diagram → 导出 MySQL → 在 MySQL 实例执行 → schema 一致 |
| UT-PC-11 | SQLite introspection：临时真实 SQLite 库建表 → 端点返回结构化 tables 含表/列/PK/FK（comment 恒空串，引擎限制） |
| UT-PC-12 | introspection IR → 响应序列化纯函数 + 错误映射（400 不支持引擎 / 502 连接失败 / 无表 table_count=0） |
| UT-PC-13 | PG introspection：嵌入式 PG 起库建表 → 端点返回结构化 tables 含表/列/联合主键/FK |
| UT-PC-14 | 导出 DDL 真实库执行验证：`export_diagram_sql` 输出在嵌入式 PG 与 SQLite 上执行成功 |
| UT-PC-15 | 前端锚点：ImportDrawer「数据库」来源 testid 与接线 |
| ST-PC-04 | e2e：ImportDrawer 数据库来源 → mock 端点返回 DDL → 本地合并落账 |

## 11. V1 边界

- ❌ 上表 9 项未实现能力
- ❌ 跨引擎 schema diff（V1 仅全量导出）
- ❌ 大文件流式导入（V1 整文件读入）
- ❌ 导入时保留 drawdb 颜色 / 锁定状态 / 索引 DDL

## 12. 对齐参考源

- drawdb §2.6 Bridge I/O
- drawdb `src/utils/exportSQL/`（7 引擎 SQL 导出器）
- drawdb `src/utils/importSQL/`（SQL 解析器）
- drawdb `src/utils/exportImport/dbml.js`（DBML 转换）
- `backend/src/phase3_bridge.rs`（5 端点 Rust 路由）
- `backend/src/areas/`, `backend/src/diagrams/`, `backend/src/fields/`, `backend/src/notes/`, `backend/src/references/`, `backend/src/tables/`, `backend/src/indices/`（领域子模块）
- `backend/src/todos/`（task 实体）
- `docs/drawdb-capability-checklist.md` §1.6 / §1.7 / §1.8

## ADDED — §8 前端 IO 抽屉对接（Phase C）

> 模块：core | 提案：redesign-phase-c-import-export

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

0. **（Phase C）** 用户在 ImportDrawer 粘贴 SQL → 客户端 `parse_sql_statements` 预览 → 提交 bridge import

## 13. 连接数据库导入（V2 增量）

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
- `tables` 按表名排序返回；前端合并管道对引用顺序 tolerant。

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


## 14. 连接数据库导出执行（V2 增量）

> 提案：feat-room-recycle-bin-dropdown-and-db-export

### 14.1 端点契约

`POST /api/v1/bridge/export/execute`，挂既有 auth 中间件（Bearer）。

请求：

```json
{ "engine": "sqlite | postgres", "source": "<SQLite 文件绝对路径 | PG 连接串>", "ddl": "CREATE TABLE ..." }
```

响应 200（`ApiResp` 包裹）：

```json
{ "code": 0, "data": { "engine": "sqlite", "statements": 5, "tables": 3 } }
```

- `ddl`：通常为前端导出预览（`export_diagram_sql`）生成的 SQL 文本；后端不做方言改写，原样执行。
- `statements`：成功执行的语句条数（按分号拆分逐条执行）。
- `tables`：所执行 DDL 中 `CREATE TABLE` 语句计数（大小写不敏感，含 `IF NOT EXISTS` 变体）。
- 任一条语句执行失败即中止，不继续执行后续语句（不重试、不跳过）。

错误语义：

| 状态码 | 场景 | 前端文案 |
|---|---|---|
| 400 | `engine` 非 `sqlite`/`postgres`，或 `source`/`ddl` 为空 | 「不支持的数据库引擎」/「请填写连接信息」 |
| 502 | 连接失败（文件不可写 / PG 不可达 / 认证失败）或 DDL 执行失败（语法错误 / 表已存在 / 约束冲突） | 「导出执行失败：<引擎错误消息>」 |

### 14.2 执行口径

- **SQLite**（sqlx-sqlite）：以写模式打开（不存在则创建文件）；逐条 `execute` 拆分后的语句。
- **PostgreSQL**（sqlx-postgres）：单连接逐条 `execute`；不开显式事务（DDL 在 PG 可事务化，但为与 SQLite 口径一致、且便于定位失败语句，采用逐条中止策略）。
- 语句拆分：按 `;` 拆分并忽略空串/纯注释段（与前端导出预览生成的语句边界一致；不解析存储过程等含内嵌分号的语法——超出当前导出器产出范围）。
- 幂等性由调用方负责：导出器默认生成不含 `IF NOT EXISTS` 的 DDL（postgresql/generic），重复执行会在「表已存在」处 502，属预期行为并在错误消息中体现。

### 14.3 安全边界

- 与 §13.4 一致：`source` 仅由后端进程本机解析，不在前端持久化、不写导入日志表（连接串可能含口令）。
- 端点需登录态；不校验路径白名单（桌面/自托管场景，服务端即用户本机）。
- `ddl` 原样执行、不做内容白名单——与「用户对自己本机/自有库执行自己生成的 DDL」的工具定位一致；该端点不面向多租户共享部署。
