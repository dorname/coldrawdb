//! feat-db-connect-import-and-ddl-realdb-verify：连接真实数据库 introspect schema → 结构化 tables JSON（core-03 §13）
//!
//! 安全边界（core-03 §13.4）：`source` 仅由后端进程本机解析；不写入导入日志表。

use serde_json::Value;
use sqlx::Row;

/// introspection 内部 IR（core-03 §13.3；comment 为 fix-canvas-zoom-invite-comment-resize 新增）
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IntrospectedColumn {
    pub name: String,
    pub col_type: String,
    pub not_null: bool,
    pub default: Option<String>,
    pub pk: bool,
    /// PG `col_description()` 列注释；SQLite 无 COMMENT 语法，恒空串（引擎限制）
    pub comment: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IntrospectedFk {
    pub columns: Vec<String>,
    pub ref_table: String,
    pub ref_columns: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IntrospectedTable {
    pub name: String,
    /// PG `obj_description()` 表注释；SQLite 恒空串（引擎限制）
    pub comment: String,
    pub columns: Vec<IntrospectedColumn>,
    pub fks: Vec<IntrospectedFk>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IntrospectError {
    UnsupportedEngine,
    EmptySource,
    ConnectFailed(String),
}

/// 错误 → HTTP 状态码（core-03 §13.1：400 参数 / 502 连接失败）
pub fn map_connect_status(err: &IntrospectError) -> u16 {
    match err {
        IntrospectError::UnsupportedEngine | IntrospectError::EmptySource => 400,
        IntrospectError::ConnectFailed(_) => 502,
    }
}

/// 错误 → 前端文案（core-03 §13.1）
pub fn connect_error_message(err: &IntrospectError) -> &'static str {
    match err {
        IntrospectError::UnsupportedEngine => "不支持的数据库引擎",
        IntrospectError::EmptySource => "请填写连接信息",
        IntrospectError::ConnectFailed(_) => "连接数据库失败，请检查连接信息",
    }
}

/// 入口：按引擎分发 introspection
pub async fn introspect(
    engine: &str,
    source: &str,
    schema: Option<&str>,
) -> Result<Vec<IntrospectedTable>, IntrospectError> {
    if source.trim().is_empty() {
        return Err(IntrospectError::EmptySource);
    }
    match engine {
        "sqlite" => introspect_sqlite(source).await,
        // fix-dbimport-save-and-pg-schema：schema 仅 postgres 生效，缺省 public；sqlite 忽略
        "postgres" => introspect_postgres(source, schema.unwrap_or("public")).await,
        _ => Err(IntrospectError::UnsupportedEngine),
    }
}

// ─── 导出执行（core-03 §14，feat-room-recycle-bin-dropdown-and-db-export） ──

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExecuteError {
    UnsupportedEngine,
    EmptySource,
    EmptyDdl,
    ConnectFailed(String),
    ExecuteFailed(String),
}

/// 执行统计（core-03 §14.1）
#[derive(Debug)]
pub struct ExecuteOutcome {
    pub statements: usize,
    pub tables: usize,
}

/// 错误 → HTTP 状态码（core-03 §14.1：400 参数 / 502 连接或执行失败）
pub fn map_execute_status(err: &ExecuteError) -> u16 {
    match err {
        ExecuteError::UnsupportedEngine | ExecuteError::EmptySource | ExecuteError::EmptyDdl => 400,
        ExecuteError::ConnectFailed(_) | ExecuteError::ExecuteFailed(_) => 502,
    }
}

/// 错误 → 前端文案（core-03 §14.1：执行失败附引擎错误消息）
pub fn execute_error_message(err: &ExecuteError) -> String {
    match err {
        ExecuteError::UnsupportedEngine => "不支持的数据库引擎".into(),
        ExecuteError::EmptySource => "请填写连接信息".into(),
        ExecuteError::EmptyDdl => "没有可执行的 DDL".into(),
        ExecuteError::ConnectFailed(m) => format!("连接数据库失败：{m}"),
        ExecuteError::ExecuteFailed(m) => format!("导出执行失败：{m}"),
    }
}

/// 语句拆分（core-03 §14.2：按 `;` 拆分，忽略空串/纯注释段）
pub fn split_ddl_statements(ddl: &str) -> Vec<String> {
    ddl.split(';')
        .map(str::trim)
        .filter(|s| {
            !s.is_empty()
                && s.lines().any(|l| {
                    let t = l.trim();
                    !t.is_empty() && !t.starts_with("--")
                })
        })
        .map(|s| s.to_string())
        .collect()
}

/// CREATE TABLE 计数（大小写不敏感，含 IF NOT EXISTS 变体）
pub fn count_create_tables(ddl: &str) -> usize {
    ddl.to_uppercase().matches("CREATE TABLE").count()
}

/// 连接目标库并逐条执行 DDL（core-03 §14.2：失败即中止，不重试不跳过）
pub async fn execute_ddl(
    engine: &str,
    source: &str,
    ddl: &str,
) -> Result<ExecuteOutcome, ExecuteError> {
    if source.trim().is_empty() {
        return Err(ExecuteError::EmptySource);
    }
    let statements = split_ddl_statements(ddl);
    if statements.is_empty() {
        return Err(ExecuteError::EmptyDdl);
    }
    let tables = count_create_tables(ddl);
    match engine {
        "sqlite" => {
            // 导出语义下允许创建新文件（mode=rwc）
            let url = format!("sqlite://{source}?mode=rwc");
            let pool = sqlx::SqlitePool::connect(&url)
                .await
                .map_err(|e| ExecuteError::ConnectFailed(e.to_string()))?;
            for stmt in &statements {
                sqlx::query(stmt)
                    .execute(&pool)
                    .await
                    .map_err(|e| ExecuteError::ExecuteFailed(e.to_string()))?;
            }
            pool.close().await;
        }
        "postgres" => {
            let pool = sqlx::PgPool::connect(source)
                .await
                .map_err(|e| ExecuteError::ConnectFailed(e.to_string()))?;
            for stmt in &statements {
                sqlx::query(stmt)
                    .execute(&pool)
                    .await
                    .map_err(|e| ExecuteError::ExecuteFailed(e.to_string()))?;
            }
            pool.close().await;
        }
        _ => return Err(ExecuteError::UnsupportedEngine),
    }
    Ok(ExecuteOutcome {
        statements: statements.len(),
        tables,
    })
}

// ─── SQLite（core-03 §13.2：sqlite_master + PRAGMA） ───────────────────────

async fn introspect_sqlite(source: &str) -> Result<Vec<IntrospectedTable>, IntrospectError> {
    // sqlx 默认会创建不存在的文件——introspection 语义下必须拒绝
    if !std::path::Path::new(source).exists() {
        return Err(IntrospectError::ConnectFailed(format!(
            "sqlite file not found: {source}"
        )));
    }
    let url = format!("sqlite://{source}?mode=ro");
    let pool = sqlx::SqlitePool::connect(&url)
        .await
        .map_err(|e| IntrospectError::ConnectFailed(e.to_string()))?;

    let table_rows = sqlx::query(
        "SELECT name FROM sqlite_master WHERE type='table' AND name NOT LIKE 'sqlite_%' ORDER BY name",
    )
    .fetch_all(&pool)
    .await
    .map_err(|e| IntrospectError::ConnectFailed(e.to_string()))?;

    let mut tables = Vec::new();
    for row in &table_rows {
        let name: String = row.get("name");
        let col_rows = sqlx::query(&format!("PRAGMA table_info(\"{}\")", name.replace('"', "\"\"")))
            .fetch_all(&pool)
            .await
            .map_err(|e| IntrospectError::ConnectFailed(e.to_string()))?;
        let columns = col_rows
            .iter()
            .map(|c| IntrospectedColumn {
                name: c.get::<String, _>("name"),
                col_type: c.get::<String, _>("type"),
                not_null: c.get::<i64, _>("notnull") == 1,
                default: c.get::<Option<String>, _>("dflt_value"),
                pk: c.get::<i64, _>("pk") > 0,
                // SQLite 无 COMMENT 语法：comment 恒空串（引擎限制，core-03 §13.2）
                comment: String::new(),
            })
            .collect();

        let fk_rows = sqlx::query(&format!(
            "PRAGMA foreign_key_list(\"{}\")",
            name.replace('"', "\"\"")
        ))
        .fetch_all(&pool)
        .await
        .map_err(|e| IntrospectError::ConnectFailed(e.to_string()))?;
        // PRAGMA foreign_key_list：复合 FK 按 id 分组、seq 排序（结果本身按 id, seq 有序）
        let mut fks: Vec<IntrospectedFk> = Vec::new();
        let mut last_fk_id: Option<i64> = None;
        for f in &fk_rows {
            let fk_id: i64 = f.get("id");
            let ref_table: String = f.get("table");
            let from: String = f.get("from");
            let to: String = f.get("to");
            if last_fk_id == Some(fk_id) {
                if let Some(last) = fks.last_mut() {
                    last.columns.push(from);
                    last.ref_columns.push(to);
                    continue;
                }
            }
            fks.push(IntrospectedFk {
                columns: vec![from],
                ref_table,
                ref_columns: vec![to],
            });
            last_fk_id = Some(fk_id);
        }
        tables.push(IntrospectedTable {
            name,
            comment: String::new(),
            columns,
            fks,
        });
    }
    pool.close().await;
    Ok(tables)
}

// ─── PostgreSQL（core-03 §13.2：information_schema + pg_catalog） ──────────

async fn introspect_postgres(source: &str, schema: &str) -> Result<Vec<IntrospectedTable>, IntrospectError> {
    let pool = sqlx::PgPool::connect(source)
        .await
        .map_err(|e| IntrospectError::ConnectFailed(e.to_string()))?;

    // 表查询 JOIN pg_class/pg_namespace 按 schema 精确过滤（fix-dbimport-save-and-pg-schema），
    // 并经 obj_description 取表注释（fix-canvas-zoom-invite-comment-resize）
    let table_rows = sqlx::query(
        "SELECT t.table_name AS table_name, obj_description(c.oid, 'pg_class') AS table_comment \
         FROM information_schema.tables t \
         JOIN pg_class c ON c.relname = t.table_name \
         JOIN pg_namespace ns ON ns.oid = c.relnamespace AND ns.nspname = t.table_schema \
         WHERE t.table_type='BASE TABLE' AND t.table_schema=$1 \
         ORDER BY t.table_name",
    )
    .bind(schema)
    .fetch_all(&pool)
    .await
    .map_err(|e| IntrospectError::ConnectFailed(e.to_string()))?;

    let mut tables = Vec::new();
    for row in &table_rows {
        let name: String = row.get("table_name");
        let comment: Option<String> = row.get("table_comment");

        // 列查询 JOIN pg_class/pg_attribute 按 schema 精确过滤，
        // 并经 col_description 取列注释
        let col_rows = sqlx::query(
            "SELECT ic.column_name, ic.data_type, ic.is_nullable, ic.column_default, \
                    ic.character_maximum_length, ic.numeric_precision, ic.numeric_scale, \
                    col_description(c.oid, a.attnum) AS column_comment \
             FROM information_schema.columns ic \
             JOIN pg_class c ON c.relname = ic.table_name \
             JOIN pg_namespace ns ON ns.oid = c.relnamespace AND ns.nspname = ic.table_schema \
             JOIN pg_attribute a ON a.attrelid = c.oid AND a.attname = ic.column_name \
             WHERE ic.table_name=$1 AND ic.table_schema=$2 \
             ORDER BY ic.ordinal_position",
        )
        .bind(&name)
        .bind(schema)
        .fetch_all(&pool)
        .await
        .map_err(|e| IntrospectError::ConnectFailed(e.to_string()))?;

        // PK 列集合（pg_catalog，复合主键保序）
        let pk_rows = sqlx::query(
            "SELECT src.attname AS column_name \
             FROM pg_constraint con \
             JOIN pg_class tbl ON tbl.oid = con.conrelid \
             JOIN pg_namespace ns ON ns.oid = tbl.relnamespace \
             JOIN unnest(con.conkey) WITH ORDINALITY ord(attnum, n) ON true \
             JOIN pg_attribute src ON src.attrelid = con.conrelid AND src.attnum = ord.attnum \
             WHERE con.contype = 'p' AND tbl.relname = $1 \
               AND ns.nspname = $2 \
             ORDER BY ord.n",
        )
        .bind(&name)
        .bind(schema)
        .fetch_all(&pool)
        .await
        .map_err(|e| IntrospectError::ConnectFailed(e.to_string()))?;
        let pk_cols: Vec<String> = pk_rows.iter().map(|r| r.get("column_name")).collect();

        let columns = col_rows
            .iter()
            .map(|c| {
                let data_type: String = c.get("data_type");
                let char_len: Option<i32> = c.get("character_maximum_length");
                let num_prec: Option<i32> = c.get("numeric_precision");
                let num_scale: Option<i32> = c.get("numeric_scale");
                let default: Option<String> = c.get("column_default");
                let col_name: String = c.get("column_name");
                let (col_type, default) =
                    normalize_pg_type(&data_type, char_len, num_prec, num_scale, default);
                IntrospectedColumn {
                    pk: pk_cols.contains(&col_name),
                    name: col_name,
                    col_type,
                    not_null: c.get::<String, _>("is_nullable") == "NO",
                    default,
                    comment: c.get::<Option<String>, _>("column_comment").unwrap_or_default(),
                }
            })
            .collect();

        // FK（pg_catalog unnest conkey/confkey 保序配对，复合 FK 按 conname 分组）
        let fk_rows = sqlx::query(
            "SELECT con.conname AS fk_name, src.attname AS column_name, \
                    ref_tbl.relname AS ref_table, ref.attname AS ref_column \
             FROM pg_constraint con \
             JOIN pg_class tbl ON tbl.oid = con.conrelid \
             JOIN pg_namespace ns ON ns.oid = tbl.relnamespace \
             JOIN pg_class ref_tbl ON ref_tbl.oid = con.confrelid \
             JOIN unnest(con.conkey) WITH ORDINALITY ord(attnum, n) ON true \
             JOIN pg_attribute src ON src.attrelid = con.conrelid AND src.attnum = ord.attnum \
             JOIN unnest(con.confkey) WITH ORDINALITY ordf(attnum, n) ON ordf.n = ord.n \
             JOIN pg_attribute ref ON ref.attrelid = con.confrelid AND ref.attnum = ordf.attnum \
             WHERE con.contype = 'f' AND tbl.relname = $1 \
               AND ns.nspname = $2 \
             ORDER BY con.conname, ord.n",
        )
        .bind(&name)
        .bind(schema)
        .fetch_all(&pool)
        .await
        .map_err(|e| IntrospectError::ConnectFailed(e.to_string()))?;
        let mut fks: Vec<IntrospectedFk> = Vec::new();
        let mut last_fk_name: Option<String> = None;
        for f in &fk_rows {
            let fk_name: String = f.get("fk_name");
            let column: String = f.get("column_name");
            let ref_table: String = f.get("ref_table");
            let ref_column: String = f.get("ref_column");
            // 结果按 conname, ord.n 有序：同名约束连续，复合 FK 归并同组
            if last_fk_name.as_deref() == Some(fk_name.as_str()) {
                if let Some(last) = fks.last_mut() {
                    last.columns.push(column);
                    last.ref_columns.push(ref_column);
                    continue;
                }
            }
            fks.push(IntrospectedFk {
                columns: vec![column],
                ref_table,
                ref_columns: vec![ref_column],
            });
            last_fk_name = Some(fk_name);
        }
        tables.push(IntrospectedTable {
            name,
            comment: comment.unwrap_or_default(),
            columns,
            fks,
        });
    }
    pool.close().await;
    Ok(tables)
}

/// PG 类型规范化（core-03 §13.3）：`character varying(n)` → `VARCHAR(n)`；
/// `integer` + nextval 默认值 → `SERIAL`（并丢弃 nextval 默认值）
fn normalize_pg_type(
    data_type: &str,
    char_len: Option<i32>,
    num_prec: Option<i32>,
    num_scale: Option<i32>,
    default: Option<String>,
) -> (String, Option<String>) {
    let is_serial = default
        .as_deref()
        .map(|d| d.starts_with("nextval("))
        .unwrap_or(false);
    let ty = match data_type {
        "character varying" => match char_len {
            Some(n) => format!("VARCHAR({n})"),
            None => "VARCHAR".to_string(),
        },
        "character" => match char_len {
            Some(n) => format!("CHAR({n})"),
            None => "CHAR".to_string(),
        },
        "numeric" => match (num_prec, num_scale) {
            (Some(p), Some(s)) => format!("NUMERIC({p},{s})"),
            (Some(p), None) => format!("NUMERIC({p})"),
            _ => "NUMERIC".to_string(),
        },
        "integer" if is_serial => "SERIAL".to_string(),
        "bigint" if is_serial => "BIGSERIAL".to_string(),
        "timestamp without time zone" => "TIMESTAMP".to_string(),
        "boolean" => "BOOLEAN".to_string(),
        "text" => "TEXT".to_string(),
        other => other.to_uppercase(),
    };
    let default = if is_serial { None } else { default };
    (ty, default)
}

// ─── 结构化响应序列化（core-03 §13.1/§13.3，fix-canvas-zoom-invite-comment-resize） ──

/// IR → `tables` JSON：元素与 persistence / JSON 导入格式同构
/// （`name` / `comment` / `fields[{name,type,primary,unique,not_null,increment,default,comment}]`）。
/// 字段映射：`col_type` → `type`；`pk` → `primary`（联合主键各列均 true）；
/// `default=None` → 空串；`comment` 原样透传；标识符不加引号（结构化 JSON 无注入面）。
/// `unique` / `increment` introspection 不产出（IR 无对应信息），恒 false。
pub fn tables_to_json(tables: &[IntrospectedTable]) -> Vec<Value> {
    tables
        .iter()
        .map(|t| {
            serde_json::json!({
                "name": t.name,
                "comment": t.comment,
                "fields": t
                    .columns
                    .iter()
                    .map(|c| {
                        serde_json::json!({
                            "name": c.name,
                            "type": c.col_type,
                            "primary": c.pk,
                            "unique": false,
                            "not_null": c.not_null,
                            "increment": false,
                            "default": c.default.clone().unwrap_or_default(),
                            "comment": c.comment,
                        })
                    })
                    .collect::<Vec<_>>(),
            })
        })
        .collect()
}

/// `import/connect` 200 data 负载：`{engine, table_count, tables}`
/// （table_count 恒等于 tables 长度；0 表时 tables 为空数组）
pub fn connect_response_data(engine: &str, tables: &[IntrospectedTable]) -> Value {
    serde_json::json!({
        "engine": engine,
        "table_count": tables.len(),
        "tables": tables_to_json(tables),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture_ir() -> Vec<IntrospectedTable> {
        vec![
            IntrospectedTable {
                name: "orders".into(),
                comment: "订单表".into(),
                columns: vec![
                    IntrospectedColumn {
                        name: "a".into(),
                        col_type: "INT".into(),
                        not_null: true,
                        default: None,
                        pk: true,
                        comment: "联合主键列".into(),
                    },
                    IntrospectedColumn {
                        name: "b".into(),
                        col_type: "INT".into(),
                        not_null: true,
                        default: None,
                        pk: true,
                        comment: String::new(),
                    },
                ],
                fks: vec![],
            },
            IntrospectedTable {
                name: "items".into(),
                comment: String::new(),
                columns: vec![
                    IntrospectedColumn {
                        name: "id".into(),
                        col_type: "SERIAL".into(),
                        not_null: true,
                        default: None,
                        pk: true,
                        comment: String::new(),
                    },
                    IntrospectedColumn {
                        name: "order".into(), // 保留字
                        col_type: "INT".into(),
                        not_null: false,
                        default: None,
                        pk: false,
                        comment: "保留字列".into(),
                    },
                    IntrospectedColumn {
                        name: "orderB".into(), // 大小写混合
                        col_type: "INT".into(),
                        not_null: false,
                        default: Some("0".into()),
                        pk: false,
                        comment: String::new(),
                    },
                ],
                fks: vec![IntrospectedFk {
                    columns: vec!["order".into(), "orderB".into()],
                    ref_table: "orders".into(),
                    ref_columns: vec!["a".into(), "b".into()],
                }],
            },
        ]
    }

    /// UT-PC-12：响应序列化纯函数 + 端点错误映射
    #[tokio::test]
    async fn ut_pc_12_response_serialization_and_error_mapping() {
        // 序列化：SQLite/PG 两路同构（纯函数与引擎无关，输出一致）
        let mut serialized = None;
        for engine in ["sqlite", "postgres"] {
            let data = connect_response_data(engine, &fixture_ir());
            assert_eq!(data["engine"], engine);
            assert_eq!(data["table_count"], 2);
            let tables = data["tables"].as_array().unwrap();
            assert_eq!(tables.len(), 2);
            // 表名原样保留（排序由 introspect 查询 ORDER BY name 保证，见 UT-PC-11/13）
            let orders = tables
                .iter()
                .find(|t| t["name"] == "orders")
                .expect("UT-PC-12: 应含 orders 表");
            let items = tables
                .iter()
                .find(|t| t["name"] == "items")
                .expect("UT-PC-12: 应含 items 表");
            // 表 comment 透传
            assert_eq!(orders["comment"], "订单表");
            assert_eq!(items["comment"], "");
            // 联合主键各列 primary=true；default 缺省空串；comment 透传
            let orders_fields = orders["fields"].as_array().unwrap();
            for f in orders_fields {
                for k in [
                    "name", "type", "primary", "unique", "not_null", "increment", "default",
                    "comment",
                ] {
                    assert!(f.get(k).is_some(), "UT-PC-12: 字段缺键 {k}");
                }
            }
            assert_eq!(orders_fields[0]["name"], "a");
            assert_eq!(orders_fields[0]["primary"], true);
            assert_eq!(orders_fields[0]["not_null"], true);
            assert_eq!(orders_fields[0]["default"], "");
            assert_eq!(orders_fields[0]["comment"], "联合主键列");
            assert_eq!(orders_fields[1]["primary"], true);
            // 保留字/大小写混合标识符原样输出（结构化 JSON 不加引号）
            let items_fields = items["fields"].as_array().unwrap();
            let order_f = items_fields
                .iter()
                .find(|f| f["name"] == "order")
                .expect("UT-PC-12: 应含保留字列 order");
            assert_eq!(order_f["comment"], "保留字列");
            assert!(items_fields.iter().any(|f| f["name"] == "orderB"));
            // default 有值时透传
            let order_b = items_fields.iter().find(|f| f["name"] == "orderB").unwrap();
            assert_eq!(order_b["default"], "0");
            if let Some(prev) = &serialized {
                assert_eq!(&data["tables"], prev, "UT-PC-12: 两引擎序列化应一致");
            }
            serialized = Some(data["tables"].clone());
        }

        // IR 层 FK 端点（响应 JSON 不含 FK，IR 保留供断言）
        let ir = fixture_ir();
        let items = ir.iter().find(|t| t.name == "items").unwrap();
        assert_eq!(items.fks[0].columns, vec!["order", "orderB"]);
        assert_eq!(items.fks[0].ref_table, "orders");
        assert_eq!(items.fks[0].ref_columns, vec!["a", "b"]);

        // 0 表 → table_count=0 且 tables=[]
        let empty = connect_response_data("sqlite", &[]);
        assert_eq!(empty["table_count"], 0);
        assert!(empty["tables"].as_array().unwrap().is_empty());

        // 错误映射（core-03 §13.1）
        assert_eq!(map_connect_status(&IntrospectError::UnsupportedEngine), 400);
        assert_eq!(map_connect_status(&IntrospectError::EmptySource), 400);
        assert_eq!(
            map_connect_status(&IntrospectError::ConnectFailed("x".into())),
            502
        );
        assert_eq!(connect_error_message(&IntrospectError::UnsupportedEngine), "不支持的数据库引擎");
        assert_eq!(connect_error_message(&IntrospectError::ConnectFailed("x".into())), "连接数据库失败，请检查连接信息");

        // engine 非法 → UnsupportedEngine；source 空 → EmptySource
        assert_eq!(
            introspect("mysql", "/tmp/x.db", None).await.unwrap_err(),
            IntrospectError::UnsupportedEngine
        );
        assert_eq!(
            introspect("sqlite", "  ", None).await.unwrap_err(),
            IntrospectError::EmptySource
        );
        // 不存在的 SQLite 路径 → ConnectFailed（且不得创建文件）
        let ghost = format!("{}/no_such_{}.db", std::env::temp_dir().display(), uuid::Uuid::new_v4());
        assert!(matches!(
            introspect("sqlite", &ghost, None).await.unwrap_err(),
            IntrospectError::ConnectFailed(_)
        ));
        assert!(!std::path::Path::new(&ghost).exists(), "UT-PC-12: introspection 不得创建文件");
        // 空库 → 0 表
        let empty = format!("{}/empty_{}.db", std::env::temp_dir().display(), uuid::Uuid::new_v4());
        std::fs::File::create(&empty).unwrap();
        let tables = introspect("sqlite", &empty, None).await.unwrap();
        assert!(tables.is_empty(), "UT-PC-12: 空库应返回 0 表");
        let _ = std::fs::remove_file(&empty);

        crate::verify_reporter::report_pass("UT-PC-12", 0);
    }

    /// UT-PC-11：SQLite introspection 真实临时库 → 结构化 tables（comment 恒空串）
    #[tokio::test]
    async fn ut_pc_11_sqlite_introspect() {
        let db_path = format!(
            "{}/cdb_introspect_{}.db",
            std::env::temp_dir().display(),
            uuid::Uuid::new_v4()
        );
        let pool = sqlx::SqlitePool::connect(&format!("sqlite://{db_path}?mode=rwc"))
            .await
            .unwrap();
        sqlx::query("PRAGMA foreign_keys = ON")
            .execute(&pool)
            .await
            .unwrap();
        sqlx::query("CREATE TABLE users (id INTEGER PRIMARY KEY, name TEXT NOT NULL)")
            .execute(&pool)
            .await
            .unwrap();
        sqlx::query(
            "CREATE TABLE posts (id INTEGER PRIMARY KEY, user_id INTEGER REFERENCES users(id), title TEXT)",
        )
        .execute(&pool)
        .await
        .unwrap();
        pool.close().await;

        let tables = introspect("sqlite", &db_path, None).await.unwrap();
        assert_eq!(tables.len(), 2, "UT-PC-11: 应 introspect 到 2 张表");
        // tables 按表名排序
        let names: Vec<&str> = tables.iter().map(|t| t.name.as_str()).collect();
        assert_eq!(names, vec!["posts", "users"]);

        // IR：FK 端点（user_id → users.id）
        let posts = tables.iter().find(|t| t.name == "posts").unwrap();
        assert_eq!(posts.fks.len(), 1, "UT-PC-11: posts 应有 1 条 FK");
        assert_eq!(posts.fks[0].columns, vec!["user_id"]);
        assert_eq!(posts.fks[0].ref_table, "users");
        assert_eq!(posts.fks[0].ref_columns, vec!["id"]);

        // 响应：结构化 tables（persistence 同构）
        let data = connect_response_data("sqlite", &tables);
        assert_eq!(data["table_count"], 2);
        let resp_tables = data["tables"].as_array().unwrap();
        let users = resp_tables
            .iter()
            .find(|t| t["name"] == "users")
            .expect("UT-PC-11: 响应应含 users 表");
        let id = users["fields"]
            .as_array()
            .unwrap()
            .iter()
            .find(|f| f["name"] == "id")
            .unwrap();
        assert_eq!(id["primary"], true, "UT-PC-11: id 应为主键");
        assert_eq!(id["type"], "INTEGER");
        let name = users["fields"]
            .as_array()
            .unwrap()
            .iter()
            .find(|f| f["name"] == "name")
            .unwrap();
        assert_eq!(name["not_null"], true, "UT-PC-11: name 应为 NOT NULL");
        let resp_posts = resp_tables
            .iter()
            .find(|t| t["name"] == "posts")
            .unwrap();
        assert!(
            resp_posts["fields"].as_array().unwrap().iter().any(|f| f["name"] == "user_id"),
            "UT-PC-11: posts 应含 FK 端点列 user_id"
        );

        // SQLite 无 COMMENT 语法：表/列 comment 恒空串（引擎限制，不报错）
        for t in &tables {
            assert!(t.comment.is_empty(), "UT-PC-11: SQLite 表 comment 应恒空串");
            for c in &t.columns {
                assert!(c.comment.is_empty(), "UT-PC-11: SQLite 列 comment 应恒空串");
            }
        }
        for t in resp_tables {
            assert_eq!(t["comment"], "");
            for f in t["fields"].as_array().unwrap() {
                assert_eq!(f["comment"], "");
            }
        }

        let _ = std::fs::remove_file(&db_path);

        crate::verify_reporter::report_pass("UT-PC-11", 0);
    }

    /// UT-PC-13：PG introspection 嵌入式真实库 → 结构化 tables
    #[tokio::test]
    async fn ut_pc_13_pg_introspect() {
        use crate::embedded_pg::EmbeddedPg;

        let pg = EmbeddedPg::start().expect("UT-PC-13: 嵌入式 PG 启动（首次需下载二进制）");
        pg.wait_ready().await.expect("UT-PC-13: PG 就绪等待");

        let url = pg.url.clone();
        let pool = sqlx::PgPool::connect(&url).await.unwrap();
        sqlx::query(
            "CREATE TABLE orders (a INT, b INT, name character varying(32), PRIMARY KEY (a, b))",
        )
        .execute(&pool)
        .await
        .unwrap();
        sqlx::query(
            "CREATE TABLE items (id SERIAL PRIMARY KEY, order_a INT, order_b INT, \
             FOREIGN KEY (order_a, order_b) REFERENCES orders (a, b))",
        )
        .execute(&pool)
        .await
        .unwrap();
        pool.close().await;

        let tables = introspect("postgres", &url, None).await.unwrap();
        assert_eq!(tables.len(), 2, "UT-PC-13: 应 introspect 到 2 张业务表（不含系统表）");
        let names: Vec<&str> = tables.iter().map(|t| t.name.as_str()).collect();
        assert_eq!(names, vec!["items", "orders"], "UT-PC-13: 不含系统表且按表名排序");

        // IR：复合 FK 端点（order_a, order_b → orders.a, b）
        let items = tables.iter().find(|t| t.name == "items").unwrap();
        assert_eq!(items.fks.len(), 1);
        assert_eq!(items.fks[0].columns, vec!["order_a", "order_b"]);
        assert_eq!(items.fks[0].ref_table, "orders");
        assert_eq!(items.fks[0].ref_columns, vec!["a", "b"]);

        // 响应：结构化 tables
        let data = connect_response_data("postgres", &tables);
        assert_eq!(data["table_count"], 2);
        let resp_tables = data["tables"].as_array().unwrap();
        let orders = resp_tables
            .iter()
            .find(|t| t["name"] == "orders")
            .expect("UT-PC-13: 响应应含 orders");
        let orders_fields = orders["fields"].as_array().unwrap();
        for col in ["a", "b"] {
            let f = orders_fields
                .iter()
                .find(|f| f["name"] == col)
                .unwrap_or_else(|| panic!("UT-PC-13: orders 应含列 {col}"));
            assert_eq!(f["primary"], true, "UT-PC-13: 联合主键列 {col} primary=true");
        }
        // character varying(32) → VARCHAR(32)（方言规范化）
        let name_f = orders_fields
            .iter()
            .find(|f| f["name"] == "name")
            .expect("UT-PC-13: orders 应含 name 列");
        assert_eq!(name_f["type"], "VARCHAR(32)", "UT-PC-13: 类型应规范化为 VARCHAR(32)");
        let resp_items = resp_tables.iter().find(|t| t["name"] == "items").unwrap();
        let id_f = resp_items["fields"]
            .as_array()
            .unwrap()
            .iter()
            .find(|f| f["name"] == "id")
            .unwrap();
        assert_eq!(id_f["primary"], true);
        assert_eq!(id_f["type"], "SERIAL", "UT-PC-13: nextval 默认值的 integer 应规范化为 SERIAL");

        drop(pg);
        crate::verify_reporter::report_pass("UT-PC-13", 0);
    }

    /// UT-PC-21：PG introspection 按 schema 精确过滤（fix-dbimport-save-and-pg-schema）
    #[tokio::test]
    async fn ut_pc_21_introspect_pg_schema_scope() {
        use crate::embedded_pg::EmbeddedPg;

        let pg = EmbeddedPg::start().expect("UT-PC-21: 嵌入式 PG 启动");
        pg.wait_ready().await.expect("UT-PC-21: PG 就绪等待");

        let url = pg.url.clone();
        let pool = sqlx::PgPool::connect(&url).await.unwrap();
        // 双 schema 同名表：public.users(id, name) vs app.users(uid, email)
        sqlx::query("CREATE TABLE public.users (id SERIAL PRIMARY KEY, name TEXT NOT NULL)")
            .execute(&pool)
            .await
            .unwrap();
        sqlx::query("CREATE SCHEMA app").execute(&pool).await.unwrap();
        sqlx::query("CREATE TABLE app.users (uid SERIAL PRIMARY KEY, email TEXT NOT NULL UNIQUE)")
            .execute(&pool)
            .await
            .unwrap();
        sqlx::query(
            "CREATE TABLE app.orders (id SERIAL PRIMARY KEY, uid INT NOT NULL, \
             FOREIGN KEY (uid) REFERENCES app.users (uid))",
        )
        .execute(&pool)
        .await
        .unwrap();
        pool.close().await;

        // 1. 缺省 schema → 仅 public.users（public 列集合）
        let tables = introspect("postgres", &url, None).await.unwrap();
        assert_eq!(tables.len(), 1, "UT-PC-21: 缺省应仅 public.users，实际 {:?}", tables.iter().map(|t| &t.name).collect::<Vec<_>>());
        let cols: Vec<&str> = tables[0].columns.iter().map(|c| c.name.as_str()).collect();
        assert_eq!(cols, ["id", "name"], "UT-PC-21: 缺省应为 public 版列集合");

        // 2. schema=app → 仅 app 两表，列集合为 app 版本，FK 指向 users
        let tables_app = introspect("postgres", &url, Some("app")).await.unwrap();
        assert_eq!(tables_app.len(), 2, "UT-PC-21: schema=app 应有 users+orders");
        let app_users = tables_app.iter().find(|t| t.name == "users").unwrap();
        let app_cols: Vec<&str> = app_users.columns.iter().map(|c| c.name.as_str()).collect();
        assert_eq!(app_cols, ["uid", "email"], "UT-PC-21: app.users 列集合不得串 public");
        let orders = tables_app.iter().find(|t| t.name == "orders").unwrap();
        assert_eq!(orders.fks.len(), 1, "UT-PC-21: app.orders 应有 1 条 FK");
        assert_eq!(orders.fks[0].ref_table, "users", "UT-PC-21: FK 指向 app.users");
        assert!(!tables_app.iter().any(|t| t.columns.iter().any(|c| c.name == "name")),
            "UT-PC-21: public.users 的 name 列不得泄漏进 app 结果");

        // 3. 不存在的 schema → 0 张表（不报错）
        let tables_none = introspect("postgres", &url, Some("no_such_schema")).await.unwrap();
        assert!(tables_none.is_empty(), "UT-PC-21: 不存在 schema 应返回 0 表");

        drop(pg);
        crate::verify_reporter::report_pass("UT-PC-21", 0);
    }

    /// UT-PC-24：PG introspection 注释直采（obj_description / col_description）
    #[tokio::test]
    async fn ut_pc_24_introspect_pg_comments() {
        use crate::embedded_pg::EmbeddedPg;

        let pg = EmbeddedPg::start().expect("UT-PC-24: 嵌入式 PG 启动（首次需下载二进制）");
        pg.wait_ready().await.expect("UT-PC-24: PG 就绪等待");

        let url = pg.url.clone();
        let pool = sqlx::PgPool::connect(&url).await.unwrap();
        sqlx::query("CREATE TABLE users (id SERIAL PRIMARY KEY, status TEXT)")
            .execute(&pool)
            .await
            .unwrap();
        sqlx::query("COMMENT ON TABLE users IS '用户表'")
            .execute(&pool)
            .await
            .unwrap();
        sqlx::query("COMMENT ON COLUMN users.status IS '状态'")
            .execute(&pool)
            .await
            .unwrap();
        // 同一列二次设置：后者生效，且验证单引号转义
        sqlx::query("COMMENT ON COLUMN users.status IS 'it''s fine'")
            .execute(&pool)
            .await
            .unwrap();
        sqlx::query("CREATE TABLE bare (id INT)")
            .execute(&pool)
            .await
            .unwrap();
        pool.close().await;

        let tables = introspect("postgres", &url, None).await.unwrap();

        // IR 层：表注释 / 列注释 / 无注释空串
        let users = tables
            .iter()
            .find(|t| t.name == "users")
            .expect("UT-PC-24: 应含 users 表");
        assert_eq!(users.comment, "用户表", "UT-PC-24: 表注释应来自 obj_description");
        let status = users
            .columns
            .iter()
            .find(|c| c.name == "status")
            .expect("UT-PC-24: users 应含 status 列");
        assert_eq!(status.comment, "it's fine", "UT-PC-24: 列注释应来自 col_description（含转义）");
        let id = users.columns.iter().find(|c| c.name == "id").unwrap();
        assert_eq!(id.comment, "", "UT-PC-24: 无注释列应返回空串");
        let bare = tables
            .iter()
            .find(|t| t.name == "bare")
            .expect("UT-PC-24: 应含 bare 表");
        assert_eq!(bare.comment, "", "UT-PC-24: 无注释表应返回空串");

        // 响应 JSON 层：comment 原样透传
        let data = connect_response_data("postgres", &tables);
        let resp_users = data["tables"]
            .as_array()
            .unwrap()
            .iter()
            .find(|t| t["name"] == "users")
            .expect("UT-PC-24: 响应应含 users");
        assert_eq!(resp_users["comment"], "用户表");
        let resp_status = resp_users["fields"]
            .as_array()
            .unwrap()
            .iter()
            .find(|f| f["name"] == "status")
            .unwrap();
        assert_eq!(resp_status["comment"], "it's fine");

        drop(pg);
        crate::verify_reporter::report_pass("UT-PC-24", 0);
    }

    /// UT-PC-25：SQLite 注释引擎限制 + 响应 schema 完整性（schema 参数忽略）
    #[tokio::test]
    async fn ut_pc_25_introspect_sqlite_response_schema() {
        let db_path = format!(
            "{}/cdb_introspect_schema_{}.db",
            std::env::temp_dir().display(),
            uuid::Uuid::new_v4()
        );
        let pool = sqlx::SqlitePool::connect(&format!("sqlite://{db_path}?mode=rwc"))
            .await
            .unwrap();
        sqlx::query("CREATE TABLE users (id INTEGER PRIMARY KEY, name TEXT NOT NULL)")
            .execute(&pool)
            .await
            .unwrap();
        sqlx::query("CREATE TABLE posts (id INTEGER PRIMARY KEY, title TEXT)")
            .execute(&pool)
            .await
            .unwrap();
        pool.close().await;

        let tables = introspect("sqlite", &db_path, None).await.unwrap();

        // 响应 schema：table_count == tables.len()；每表含 name/comment/fields；
        // 每字段含 name/type/primary/unique/not_null/increment/default/comment
        let data = connect_response_data("sqlite", &tables);
        assert_eq!(
            data["table_count"].as_u64().unwrap() as usize,
            tables.len(),
            "UT-PC-25: table_count 应等于 tables 长度"
        );
        for t in data["tables"].as_array().unwrap() {
            for k in ["name", "comment", "fields"] {
                assert!(t.get(k).is_some(), "UT-PC-25: 表缺键 {k}");
            }
            for f in t["fields"].as_array().unwrap() {
                for k in [
                    "name", "type", "primary", "unique", "not_null", "increment", "default",
                    "comment",
                ] {
                    assert!(f.get(k).is_some(), "UT-PC-25: 字段缺键 {k}");
                }
                assert_eq!(f["comment"], "", "UT-PC-25: SQLite 字段 comment 应恒空串");
            }
            assert_eq!(t["comment"], "", "UT-PC-25: SQLite 表 comment 应恒空串");
        }

        // IR 层同样断言（不报错，引擎限制）
        for t in &tables {
            assert!(t.comment.is_empty());
            for c in &t.columns {
                assert!(c.comment.is_empty());
            }
        }

        // 0 表 → table_count=0 且 tables=[]
        let empty_data = connect_response_data("sqlite", &[]);
        assert_eq!(empty_data["table_count"], 0);
        assert!(empty_data["tables"].as_array().unwrap().is_empty());

        // schema 参数对 sqlite 忽略（不报错、结果不变）
        let with_schema = introspect("sqlite", &db_path, Some("no_such_schema")).await.unwrap();
        assert_eq!(with_schema.len(), 2, "UT-PC-25: sqlite 应忽略 schema 参数");

        let _ = std::fs::remove_file(&db_path);
        crate::verify_reporter::report_pass("UT-PC-25", 0);
    }

    /// UT-PC-16：导出执行 SQLite 真实库（core-03 §14）
    #[tokio::test]
    async fn ut_pc_16_export_execute_sqlite() {
        let db_path = format!(
            "{}/cdb_export_exec_{}.db",
            std::env::temp_dir().display(),
            uuid::Uuid::new_v4()
        );
        let ddl = "CREATE TABLE users (id INTEGER PRIMARY KEY, name TEXT NOT NULL);\n\
                   CREATE TABLE posts (id INTEGER PRIMARY KEY, user_id INTEGER REFERENCES users(id), title TEXT);";

        let out = execute_ddl("sqlite", &db_path, ddl).await.unwrap();
        assert_eq!(out.statements, 2, "UT-PC-16: 应执行 2 条语句");
        assert_eq!(out.tables, 2, "UT-PC-16: 应计数 2 张 CREATE TABLE");

        // 直查：2 表落库且 FK 子句存在
        let pool = sqlx::SqlitePool::connect(&format!("sqlite://{db_path}?mode=ro"))
            .await
            .unwrap();
        let rows = sqlx::query(
            "SELECT name, sql FROM sqlite_master WHERE type='table' AND name NOT LIKE 'sqlite_%' ORDER BY name",
        )
        .fetch_all(&pool)
        .await
        .unwrap();
        assert_eq!(rows.len(), 2, "UT-PC-16: 库内应有 2 张表");
        let posts_sql: String = rows
            .iter()
            .find(|r| r.get::<String, _>("name") == "posts")
            .map(|r| r.get("sql"))
            .unwrap();
        assert!(
            posts_sql.contains("REFERENCES users(id)"),
            "UT-PC-16: posts 应含 FK 子句，实际：{posts_sql}"
        );
        pool.close().await;

        // 重复执行同一 DDL（无 IF NOT EXISTS）→ 502 含引擎错误（表已存在）
        let err = execute_ddl("sqlite", &db_path, ddl).await.unwrap_err();
        assert!(
            matches!(err, ExecuteError::ExecuteFailed(ref m) if m.contains("already exists")),
            "UT-PC-16: 重复执行应 ExecuteFailed 且含 already exists，实际：{err:?}"
        );
        assert_eq!(map_execute_status(&err), 502);
        assert!(execute_error_message(&err).contains("导出执行失败"));

        // 拆分/计数纯函数
        assert_eq!(split_ddl_statements("-- 注释\n\n;").len(), 0);
        assert_eq!(count_create_tables("create table a (id int); CREATE TABLE IF NOT EXISTS b (id int);"), 2);

        let _ = std::fs::remove_file(&db_path);
        crate::verify_reporter::report_pass("UT-PC-16", 0);
    }

    /// UT-PC-17：导出执行嵌入式 PG + 错误映射（core-03 §14）
    #[tokio::test]
    async fn ut_pc_17_export_execute_pg() {
        use crate::embedded_pg::EmbeddedPg;

        // 错误映射（无需实例）
        assert_eq!(map_execute_status(&ExecuteError::UnsupportedEngine), 400);
        assert_eq!(map_execute_status(&ExecuteError::EmptySource), 400);
        assert_eq!(map_execute_status(&ExecuteError::EmptyDdl), 400);
        assert_eq!(
            execute_ddl("mysql", "postgres://x", "CREATE TABLE a (id INT);")
                .await
                .unwrap_err(),
            ExecuteError::UnsupportedEngine
        );
        assert_eq!(
            execute_ddl("postgres", "  ", "CREATE TABLE a (id INT);")
                .await
                .unwrap_err(),
            ExecuteError::EmptySource
        );
        assert_eq!(
            execute_ddl("postgres", "postgres://x", "  -- 只有注释\n; ")
                .await
                .unwrap_err(),
            ExecuteError::EmptyDdl
        );

        let pg = EmbeddedPg::start().expect("UT-PC-17: 嵌入式 PG 启动（首次需下载二进制）");
        pg.wait_ready().await.expect("UT-PC-17: PG 就绪等待");

        let ddl = "CREATE TABLE orders (a INT, b INT, PRIMARY KEY (a, b));\n\
                   CREATE TABLE items (id SERIAL PRIMARY KEY, order_a INT, order_b INT, \
                   FOREIGN KEY (order_a, order_b) REFERENCES orders (a, b));";
        let out = execute_ddl("postgres", &pg.url, ddl).await.unwrap();
        assert_eq!(out.statements, 2);
        assert_eq!(out.tables, 2, "UT-PC-17: 应计数 2 张 CREATE TABLE");

        let pool = sqlx::PgPool::connect(&pg.url).await.unwrap();
        let rows = sqlx::query(
            "SELECT table_name FROM information_schema.tables \
             WHERE table_type='BASE TABLE' AND table_schema='public' ORDER BY table_name",
        )
        .fetch_all(&pool)
        .await
        .unwrap();
        let names: Vec<String> = rows.iter().map(|r| r.get("table_name")).collect();
        assert_eq!(names, vec!["items", "orders"], "UT-PC-17: PG 库内应有 2 表");
        pool.close().await;

        // 语法错误 DDL → 502 含 PG 引擎错误片段
        let err = execute_ddl("postgres", &pg.url, "CREATE TABL broken (")
            .await
            .unwrap_err();
        assert!(
            matches!(err, ExecuteError::ExecuteFailed(ref m) if m.contains("syntax error")),
            "UT-PC-17: 语法错误应 ExecuteFailed 且含 PG 错误，实际：{err:?}"
        );
        assert_eq!(map_execute_status(&err), 502);
        assert!(execute_error_message(&err).contains("导出执行失败"));

        drop(pg);
        crate::verify_reporter::report_pass("UT-PC-17", 0);
    }
}
