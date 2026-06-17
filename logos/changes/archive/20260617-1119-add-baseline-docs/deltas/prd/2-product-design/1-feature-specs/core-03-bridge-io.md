## MODIFIED — 顶部元数据剥离

> 模块：core | 提案：add-baseline-docs
> 路径：`logos/resources/prd/2-product-design/1-feature-specs/core-03-bridge-io.md`
> 策略：移除文件开头的 `## ADDED — ...` / `## MODIFIED — ...` / `## REMOVED — ...` 标记块及其紧随的 `>` 元数据行，保留正文首个一级标题以下所有内容原样。

# 桥接导入 / 导出规格（V1）

## 1. Bridge API 端点（5 个）

| 端点 | 方法 | 用途 | 状态码 |
|---|---|---|---|
| `/api/v1/bridge/import/local` | POST | 从 SQL/DBML/JSON 内容导入 diagram | 201 / 400 |
| `/api/v1/bridge/import/local/logs` | GET | 列出所有导入日志 | 200 |
| `/api/v1/bridge/import/local/retry/{id}` | POST | 重试失败的导入任务 | 200 / 404 / 409 |
| `/api/v1/bridge/config` | GET | 读取桥接配置（默认引擎 / 缩进 / 命名风格） | 200 |
| `/api/v1/bridge/config` | PUT | 更新桥接配置 | 200 / 400 |

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

