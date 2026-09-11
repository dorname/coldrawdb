# Delta: core-01d-import-export.md

> 模块：core | 提案：fix-canvas-zoom-invite-comment-resize（问题3：数据库导入注释丢失 + 注释展示）

背景：数据库导入原为「后端 introspect → 渲染 DDL 文本 → 前端 `parse_sql_import_tables`」纯文本管道，表/列注释在 introspection 查询、IR、DDL 渲染、DDL 解析四个环节全部丢失。修复：bridge 导入改返回**结构化 tables JSON**（含 comment），前端复用 JSON 导入路径；导出补 `COMMENT ON TABLE`；SQL 解析器补 `COMMENT ON` 语句解析以支持导出→导入 round-trip。

## MODIFIED — §4.2 格式 Tab

| format | 引擎选择 | 解析预览 |
|--------|----------|----------|
| `sql` | 必填（默认 `generic`） | `parse_sql_statements` 语句数 + 简单表名启发式（可选） |
| `dbml` | 隐藏 | 行数 / `Table` 块计数（纯函数 `count_dbml_tables`） |
| `json` | 隐藏 | `serde_json` 校验 + tables 数组长度 |
| `database` | 必填（`sqlite` / `postgres`，默认 `sqlite`） | 「连接并解析」→ `POST /api/v1/bridge/import/connect` 返回**结构化 tables JSON**（含表/列 comment）→ 复用 JSON 导入摘要口径（见 §4.4 第 3.5 条） |

## MODIFIED — §4.4 提交行为（第 2 条解析口径 + 第 3.5 条数据库连接导入）

2. **SQL 导入解析（relation-inspector-and-ddl-io：列级 DDL 解析）**——`parse_sql_import_tables` 口径：
   - 列定义：列名、类型整串原样保留（含 `VARCHAR(32)` / `DECIMAL(10,2)` 参数化）、列级约束 `PRIMARY KEY` / `NOT NULL` / `UNIQUE` / `AUTO_INCREMENT`（PG `SERIAL`/`BIGSERIAL` 类型 → `increment=true`）/ `DEFAULT <值>` / MySQL 行内 `COMMENT '...'`
   - **PG 风格 `COMMENT ON TABLE t IS '...'` 语句（fix-canvas-zoom-invite-comment-resize 新增）**：解析并回填对应表 `Table.comment`；**`COMMENT ON COLUMN t.c IS '...'`**：回填对应字段 `Field.comment`。目标表/列在本文本中不存在 → 跳过该语句（不报错，与 REFERENCES 悬空口径一致）。两种语句不计入「非 CREATE TABLE 语句」之外的错误路径
   - 表级约束：`PRIMARY KEY(a[, b])`（含联合主键）、`FOREIGN KEY (c) REFERENCES t(c2)` 与列级 `c ... REFERENCES t(c2)` → 生成 `references` 连线（cardinality 按 core-01b §2 推导口径：目标列唯一/主键 → one_to_one，否则 one_to_many）
   - 标识符规范化：反引号 / 双引号 / 方括号去除；未知类型整串原样保留（不做方言映射）
   - 非 CREATE TABLE 语句跳过（**`COMMENT ON` 除外，见上**）；无合法 CREATE TABLE → 报错「未检测到数据表」
   - **V1 不做**：CHECK、INDEX/KEY 索引定义、ALTER TABLE 序列、视图/触发器、引擎方言全量映射

3.5. **数据库连接导入（feat-db-connect-import-and-ddl-realdb-verify；fix-canvas-zoom-invite-comment-resize 改结构化 JSON 直传）**——`database` Tab 提交流程：
   - 「连接并解析」：校验引擎与连接信息非空 → `POST /api/v1/bridge/import/connect`（`{ engine, source }`；引擎为 `postgres` 时携带 `schema`——输入框值去空白、为空则 `public`；`sqlite` 不携带，fix-dbimport-save-and-pg-schema）→ 200 取 `data.tables`（**结构化表数组**，含表/列 comment，契约见 `core-03-bridge-io.md` §13.1）→ 前端**复用 JSON 导入解析路径**（`parse_json_import_tables` 或对其的轻适配：把 `data.tables` 序列化为 JSON 导入同构文本后走同一纯函数）→ 解析摘要显示「已连接 · 检测到 N 张表」（N = `data.table_count`）；`table_count=0` → inline 错误「未检测到数据表」
   - **不再经 DDL 文本中转**：后端不再渲染 `ddl` 字符串，前端不再调用 `parse_sql_import_tables` 消费连接导入结果（SQL Tab 粘贴路径不变）
   - 端点错误文案：400 → 「不支持的数据库引擎」/「请填写连接信息」；502 → 「连接数据库失败，请检查连接信息」（inline 错误，不关抽屉）
   - 解析成功后「导入到画布」提交：与第 3 条**完全相同的本地合并管道**（`merge_import_into_store` 避让合并 → schedule_save PUT 落账 → Toast「已导入」→ 关闭抽屉），不调 `bridge/import/local`、不跳转页面
   - **注释保留**：导入产生的表/列 comment 原样进入 `Table.comment` / `Field.comment`，经 §3 合并与保存链路落库；展示/编辑入口见 `core-01a-table-and-field.md` §1.4
   - 连接信息（文件路径/连接串）仅用于当次请求，不写入 localStorage / 导入日志
   - 只读（Viewer）下 ImportDrawer 整体不可达（入口已禁用），数据库 Tab 不单独授权

## ADDED — §4.7 导入注释保留与引擎限制

> 提案：fix-canvas-zoom-invite-comment-resize

| 导入源 | 表注释 | 列注释 | 说明 |
|---|---|---|---|
| JSON（persistence 同构） | ✅ 透传 | ✅ 透传 | 既有能力，本提案补展示/编辑入口 |
| SQL 粘贴（MySQL 行内 `COMMENT '...'`） | ⚠️ 不支持 | ✅ 解析 | MySQL 行内表注释无标准语法位置（表选项 `COMMENT='...'` 解析器 V1 不解析），表注释丢弃属已知边界 |
| SQL 粘贴（PG `COMMENT ON TABLE/COLUMN`） | ✅ 解析（本提案新增） | ✅ 解析（本提案新增） | 支撑导出→导入 round-trip |
| 数据库连接（PostgreSQL） | ✅ `obj_description()` introspect | ✅ `col_description()` introspect | 结构化 JSON 直传 |
| 数据库连接（SQLite） | ❌ 引擎限制 | ❌ 引擎限制 | SQLite 无 COMMENT 语法；`comment` 字段返回空串，前端展示为空、不报错 |
| MySQL 数据库连接导入 | — | — | introspection 当前不支持 MySQL 引擎（`engine` 枚举仅 `sqlite`/`postgres`），不在本提案范围 |

- 引擎限制的口径：SQLite 导入结果中 comment 恒为空串，属**引擎能力缺失**而非缺陷；文档与 UI 均不承诺 SQLite 注释保留
- 数据库连接导入的注释完整性以 PostgreSQL 为准验收（见 §4.6 与测试用例 UT-PC-24）

## MODIFIED — §4.6 DDL 真实库验证口径

> 提案：feat-db-connect-import-and-ddl-realdb-verify（fix-canvas-zoom-invite-comment-resize 扩充注释断言）

导出 DDL 的正确性验证从「文本断言」升级为「真实库执行」：

- 验证范围：本次承诺 **PostgreSQL + SQLite** 两路（MySQL/Oracle 等其余引擎仍文本断言，不在本变更）
- 验证方式：frontend-rs 集成测试（native target）调 `export_diagram_sql(store, engine)` 生成 DDL → 在**真实数据库实例**执行全部语句 → 断言执行成功且 `information_schema.tables` / `sqlite_master` 中建表数量、外键数量与模型一致
- **注释 round-trip（fix-canvas-zoom-invite-comment-resize 新增）**：PG 验证夹具中模型含表/列注释 → 执行导出 DDL 后直查 `obj_description()` / `col_description()` 与模型 `Table.comment` / `Field.comment` 一致；`COMMENT ON TABLE` 语句本身可被真实 PG 执行成功（语法验证）
- PG 实例：`postgresql_embedded`（dev-dependency，运行时从 Maven Central 拉取二进制，首次运行有下载耗时）
- SQLite 实例：sqlx-sqlite 临时文件（测试结束清理）
- 若生成 DDL 在真实库执行失败，视为导出器缺陷，须在本变更内修复生成逻辑（不接受跳过断言）

## MODIFIED — §5.2 预览生成（客户端）

| format | 函数 | V1 范围 |
|--------|------|---------|
| SQL | `export_diagram_sql(store, engine)` | `CREATE TABLE` + 列定义（类型整串 + PK/NOT NULL/**UNIQUE**/**DEFAULT**/**自增**（引擎映射：mysql/generic → `AUTO_INCREMENT`；postgresql 由 `SERIAL` 类型承担不追加）/**COMMENT**：mysql 行内 `COMMENT 'x'`，其余引擎后置 `COMMENT ON COLUMN t.c IS 'x'`）+ FK 引用 `references`（表末 `FOREIGN KEY (c) REFERENCES t(c2)`）+ **表注释（fix-canvas-zoom-invite-comment-resize 新增）**：mysql 表选项 `COMMENT='x'`（`CREATE TABLE ... ) COMMENT='x';`），postgresql/generic 后置 `COMMENT ON TABLE t IS 'x';`；表注释为空则不输出 |
| DBML | `export_diagram_dbml(store)` | 表 + 字段 + `ref:` 关系 |
| JSON | `serde_json::to_string_pretty(diagram)` | 与 persistence JSON 同构 |

- `COMMENT ON TABLE` 与 `COMMENT ON COLUMN` 按表分组输出在该表 `CREATE TABLE` 之后；单引号按既有列注释口径转义（`'` → `''`）
- 方言形态：

```sql
-- postgresql / generic
CREATE TABLE users ( ... );
COMMENT ON TABLE users IS '用户表';
COMMENT ON COLUMN users.status IS '状态';

-- mysql（表选项，置于 CREATE TABLE 末尾）
CREATE TABLE users ( ... ) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COMMENT='用户表';
```
