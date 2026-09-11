# Delta: core-PC-import-export-test-cases.md

## MODIFIED — UT-PC-02 用例行（主表）

| UT-PC-02 | store 含表（含 UNIQUE/自增/DEFAULT/COMMENT 字段）+ 关系 | `export_diagram_sql(store, engine)` | 输出含 `CREATE TABLE` 与表名；列含 `UNIQUE`/`DEFAULT`/自增（generic/mysql → `AUTO_INCREMENT`，pg 由 SERIAL 承担不追加）/COMMENT（mysql 行内，其余 `COMMENT ON COLUMN`）；含 `FOREIGN KEY (c) REFERENCES t(c2)` |

## ADDED — UT-PC-07 用例行（主表）

| UT-PC-07 | DDL 文本（多表，含参数化类型/列级约束/DEFAULT/COMMENT/表级联合主键） | `parse_sql_import_tables(content)` | 表名/列名/类型整串（`VARCHAR(32)` 保留）正确；列级 PK/NOT NULL/UNIQUE/AUTO_INCREMENT（SERIAL→自增）/DEFAULT/COMMENT 落字段；表级 `PRIMARY KEY(a,b)` 双列置位；引号/方括号规范化；非 CREATE TABLE 语句跳过；无合法语句报错 |

## ADDED — UT-PC-08 用例行（主表）

| UT-PC-08 | DDL 含表级 `FOREIGN KEY (c) REFERENCES t(c2)` 与列级 `REFERENCES t(c2)` | `parse_sql_import_tables(content)` | 生成 references 连线（端点解析到正确表/字段 id；跨已存在表可解析）；无 REFERENCES 时 references 为空 |

## ADDED — UT-PC-07 — DDL 列级导入解析纯函数测试

- **位置**：`frontend-rs/src/editor_panels.rs`（`parse_sql_import_tables`）
- **步骤与预期**：
  1. 解析 `CREATE TABLE users (id UUID PRIMARY KEY, name VARCHAR(32) NOT NULL UNIQUE, age INT DEFAULT 18, bio TEXT COMMENT '简介');` → 表 users 4 字段：`id` pk+类型 UUID；`name` 类型 `VARCHAR(32)`（参数化整串保留）+ not_null + unique；`age` default="18"；`bio` comment="简介"
  2. `AUTO_INCREMENT` 列 → increment=true；PG `SERIAL`/`BIGSERIAL` 类型列 → increment=true
  3. 表级 `PRIMARY KEY(a, b)` → a、b 两列 primary=true（联合主键）
  4. 反引号/双引号/方括号标识符（`` `user` `` / `"user"` / `[user]`）→ 列名去引号
  5. `INSERT INTO ...` / `CREATE INDEX ...` 等非 CREATE TABLE 语句跳过；全文无合法 CREATE TABLE → Err
  6. 未知类型（如 `MONEY`）整串原样保留，不做方言映射

## ADDED — UT-PC-08 — DDL 导入外键生成 references 纯函数测试

- **位置**：`frontend-rs/src/editor_panels.rs`（`parse_sql_import_tables` 或拆出的 REFERENCES 收集函数）
- **步骤与预期**：
  1. 表级约束 `FOREIGN KEY (user_id) REFERENCES users(id)` → references 含 1 条，start=当前表.user_id，end=users.id（端点解析到表/字段实体 id）
  2. 列级 `user_id UUID REFERENCES users(id)` → 同样生成 1 条
  3. 无 REFERENCES 的 DDL → references 为空
  4. 指向不存在表/列的 FK → 跳过该条（不报错、不产悬空引用）
