//! UT-PC-14：导出 DDL 真实库执行验证（feat-db-connect-import-and-ddl-realdb-verify）
//!
//! Spec: `logos/resources/test/core-PC-import-export-test-cases.md` UT-PC-14
//!       + `logos/resources/implementation/core-01d.md` §4.6
//! Source: `frontend-rs/src/editor_panels.rs::export_diagram_sql`
//!
//! 口径（§4.6）：fixture 为 2 表 + 1 关系，含参数化类型 / DEFAULT / 自增主键；
//! 导出 DDL 分别在 PostgreSQL（嵌入式实例）与 SQLite（临时文件）真实执行，
//! 断言建表数 = 2、外键数 = 1。**生成 DDL 执行失败视为导出器缺陷，不得跳过断言。**
//!
//! 嵌入式 PG helper 复制自 `backend/src/embedded_pg.rs`（本变更自研方案，
//! postgresql_embedded crate 因 rustc 1.94 要求与 root 限制不可用），
//! 去掉 uuid 依赖（改用 pid+纳秒唯一后缀）。

mod verify_reporter;

use frontend_rs::editor_core::types::{Field, Reference, Table};
use frontend_rs::editor_panels::export_diagram_sql;

use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};

// ── fixture：users / orders 2 表 + orders.user_id → users.id 1 关系 ─────────

fn field(id: &str, name: &str, type_: &str, primary: bool, not_null: bool, increment: bool, default: &str, unique: bool) -> Field {
    Field {
        id: id.to_string(),
        name: name.to_string(),
        type_: type_.to_string(),
        default: default.to_string(),
        check: String::new(),
        primary,
        unique,
        not_null,
        increment,
        comment: String::new(),
        tag: String::new(),
        dict_code: String::new(),
    }
}

fn fixture_users_orders() -> (Vec<Table>, Vec<Reference>) {
    let users = Table {
        id: "t-users".to_string(),
        name: "users".to_string(),
        x: 0.0,
        y: 0.0,
        color: String::new(),
        comment: String::new(),
        fields: vec![
            // 自增主键：sqlite 引擎导出须为 INTEGER PRIMARY KEY AUTOINCREMENT
            field("f-u-id", "id", "INT", true, true, true, "", false),
            // 参数化类型 + NOT NULL + UNIQUE
            field("f-u-name", "name", "VARCHAR(32)", false, true, false, "", true),
            // DEFAULT
            field("f-u-age", "age", "INT", false, false, false, "18", false),
        ],
        indices: vec![],
        width: None,
        min_height: None,
    };
    let orders = Table {
        id: "t-orders".to_string(),
        name: "orders".to_string(),
        x: 0.0,
        y: 0.0,
        color: String::new(),
        comment: String::new(),
        fields: vec![
            field("f-o-id", "id", "INT", true, true, true, "", false),
            field("f-o-uid", "user_id", "INT", false, true, false, "", false),
            field("f-o-amt", "amount", "DECIMAL", false, false, false, "", false),
        ],
        indices: vec![],
        width: None,
        min_height: None,
    };
    let reference = Reference {
        id: "r-1".to_string(),
        name: String::new(),
        // start_table = 外键持有方（orders.user_id → users.id）
        start_table_id: "t-orders".to_string(),
        end_table_id: "t-users".to_string(),
        start_field_id: "f-o-uid".to_string(),
        end_field_id: "f-u-id".to_string(),
        type_: "one_to_many".to_string(),
        on_delete: "RESTRICT".to_string(),
        on_update: "RESTRICT".to_string(),
    };
    (vec![users, orders], vec![reference])
}

/// 导出 DDL 按 `;` 切分为可执行语句（fixture 字符串内无分号）
fn split_statements(ddl: &str) -> Vec<String> {
    ddl.split(';')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect()
}

fn unique_tag() -> String {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    format!("{}-{}", std::process::id(), nanos)
}

// ── 嵌入式 PG helper（复制自 backend/src/embedded_pg.rs，去 uuid）────────────

const PG_VERSION: &str = "17.5.0";
const NOBODY: &str = "65534";

struct EmbeddedPg {
    child: Child,
    url: String,
    data_dir: PathBuf,
}

impl EmbeddedPg {
    fn start() -> Result<Self, String> {
        let install = ensure_binaries()?;
        let data_dir = std::env::temp_dir().join(format!("cdb-pg-data-{}", unique_tag()));
        std::fs::create_dir_all(&data_dir).map_err(|e| e.to_string())?;
        // initdb/postgres 以 nobody 运行：目录所有权移交
        run_ok("chown", &["-R", &format!("{NOBODY}:{NOBODY}"), &data_dir.to_string_lossy()])?;

        run_as_nobody(
            &install.join("bin/initdb"),
            &[
                "-D", &data_dir.to_string_lossy(),
                "-U", "postgres",
                "--auth=trust",
                "-E", "UTF8",
            ],
            &install,
        )?;

        let port = free_port()?;
        let child = Command::new("setpriv")
            .args([
                &format!("--reuid={NOBODY}"),
                &format!("--regid={NOBODY}"),
                "--clear-groups",
            ])
            // zonky 二进制的 libpq 等动态库在 <install>/lib，须显式 LD_LIBRARY_PATH
            .env("LD_LIBRARY_PATH", install.join("lib"))
            .arg(install.join("bin/postgres"))
            .args([
                "-D", &data_dir.to_string_lossy(),
                "-p", &port.to_string(),
                "-k", &data_dir.to_string_lossy(), // unix socket 目录（nobody 可写）
                "-c", "listen_addresses=127.0.0.1",
            ])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .map_err(|e| format!("spawn postgres: {e}"))?;

        let url = format!("postgres://postgres@127.0.0.1:{port}/postgres");
        Ok(Self { child, url, data_dir })
    }

    async fn wait_ready(&self) -> Result<(), String> {
        for _ in 0..60 {
            if sqlx::PgPool::connect(&self.url).await.is_ok() {
                return Ok(());
            }
            tokio::time::sleep(std::time::Duration::from_millis(500)).await;
        }
        Err("embedded pg 30s 内未就绪".into())
    }
}

impl Drop for EmbeddedPg {
    fn drop(&mut self) {
        // SIGINT = fast shutdown
        let _ = Command::new("kill").args(["-INT", &self.child.id().to_string()]).output();
        let _ = self.child.wait();
        let _ = std::fs::remove_dir_all(&self.data_dir);
    }
}

fn free_port() -> Result<u16, String> {
    let listener = std::net::TcpListener::bind("127.0.0.1:0").map_err(|e| e.to_string())?;
    Ok(listener.local_addr().map_err(|e| e.to_string())?.port())
}

fn run_ok(cmd: &str, args: &[&str]) -> Result<(), String> {
    let out = Command::new(cmd)
        .args(args)
        .output()
        .map_err(|e| format!("{cmd} spawn: {e}"))?;
    if out.status.success() {
        Ok(())
    } else {
        Err(format!("{cmd} {args:?} 失败：{}", String::from_utf8_lossy(&out.stderr)))
    }
}

fn run_as_nobody(bin: &Path, args: &[&str], install: &Path) -> Result<(), String> {
    let out = Command::new("setpriv")
        .args([
            &format!("--reuid={NOBODY}"),
            &format!("--regid={NOBODY}"),
            "--clear-groups",
        ])
        .env("LD_LIBRARY_PATH", install.join("lib"))
        .arg(bin)
        .args(args)
        .output()
        .map_err(|e| format!("initdb spawn: {e}"))?;
    if out.status.success() {
        Ok(())
    } else {
        Err(format!(
            "initdb 失败：{}{}",
            String::from_utf8_lossy(&out.stdout),
            String::from_utf8_lossy(&out.stderr)
        ))
    }
}

/// 下载并解压 zonky PG 二进制（jar 内嵌 .txz），缓存复用（与 backend 同一缓存目录）
fn ensure_binaries() -> Result<PathBuf, String> {
    // 必须放在 nobody 可遍历的位置（/root 为 700，降权后 EACCES）；/tmp 全局可写
    let base = std::env::temp_dir().join(format!(".cache/cdb-pg/{PG_VERSION}"));
    let marker = base.join("bin/postgres");
    if marker.exists() {
        return Ok(base);
    }
    std::fs::create_dir_all(&base).map_err(|e| e.to_string())?;
    let jar = base.join("pg.jar");
    let url = format!(
        "https://repo1.maven.org/maven2/io/zonky/test/postgres/embedded-postgres-binaries-linux-amd64/{PG_VERSION}/embedded-postgres-binaries-linux-amd64-{PG_VERSION}.jar"
    );
    run_ok("curl", &["-fsSL", &url, "-o", &jar.to_string_lossy()])?;
    run_ok("unzip", &["-o", "-q", &jar.to_string_lossy(), "-d", &base.to_string_lossy()])?;
    let txz = std::fs::read_dir(&base)
        .map_err(|e| e.to_string())?
        .filter_map(|e| e.ok().map(|e| e.path()))
        .find(|p| p.extension().map(|e| e == "txz").unwrap_or(false))
        .ok_or("zonky jar 内未找到 .txz")?;
    run_ok("tar", &["-xJf", &txz.to_string_lossy(), "-C", &base.to_string_lossy()])?;
    // txz 解压出 pgsql/ 子目录：摊平到 base
    let inner = base.join("pgsql");
    if inner.exists() {
        for entry in std::fs::read_dir(&inner).map_err(|e| e.to_string())? {
            let entry = entry.map_err(|e| e.to_string())?;
            let target = base.join(entry.file_name());
            if !target.exists() {
                std::fs::rename(entry.path(), target).map_err(|e| e.to_string())?;
            }
        }
    }
    if !marker.exists() {
        return Err(format!("PG 二进制解压后缺 bin/postgres：{}", base.display()));
    }
    Ok(base)
}

// ── UT-PC-14 ────────────────────────────────────────────────────────────────

#[tokio::test]
async fn ut_pc_14_export_ddl_executes_on_real_databases() {
    let (tables, references) = fixture_users_orders();

    // ── PostgreSQL 路：导出 DDL 在嵌入式真实 PG 执行 ──
    let pg = EmbeddedPg::start().expect("embedded pg start");
    pg.wait_ready().await.expect("embedded pg ready");
    let pool = sqlx::PgPool::connect(&pg.url).await.expect("pg connect");
    let ddl = export_diagram_sql(&tables, &references, "postgresql");
    for stmt in split_statements(&ddl) {
        sqlx::query(&stmt)
            .execute(&pool)
            .await
            .unwrap_or_else(|e| panic!("PG 执行导出 DDL 失败（导出器缺陷，§4.6 禁止跳过）: {e}\n语句:\n{stmt}"));
    }
    let (tables_count,): (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM information_schema.tables \
         WHERE table_schema = 'public' AND table_type = 'BASE TABLE'",
    )
    .fetch_one(&pool)
    .await
    .expect("pg table count");
    assert_eq!(tables_count, 2, "PG 建表数应为 2（users + orders）");
    let (fk_count,): (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM pg_constraint \
         WHERE contype = 'f' AND connamespace = (SELECT oid FROM pg_namespace WHERE nspname = 'public')",
    )
    .fetch_one(&pool)
    .await
    .expect("pg fk count");
    assert_eq!(fk_count, 1, "PG 外键数应为 1（orders.user_id → users.id）");
    drop(pool);
    drop(pg);

    // ── SQLite 路：导出 DDL 在临时文件真实执行 ──
    let db_path = std::env::temp_dir().join(format!("cdb-ut-pc14-{}.db", unique_tag()));
    // mode=rwc：不存在则创建（sqlx 0.7 默认 create_if_missing=false）
    let url = format!("sqlite://{}?mode=rwc", db_path.display());
    let pool = sqlx::SqlitePool::connect(&url).await.expect("sqlite connect");
    let ddl = export_diagram_sql(&tables, &references, "sqlite");
    for stmt in split_statements(&ddl) {
        sqlx::query(&stmt)
            .execute(&pool)
            .await
            .unwrap_or_else(|e| panic!("SQLite 执行导出 DDL 失败（导出器缺陷，§4.6 禁止跳过）: {e}\n语句:\n{stmt}"));
    }
    let (tables_count,): (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM sqlite_master WHERE type = 'table' AND name NOT LIKE 'sqlite_%'",
    )
    .fetch_one(&pool)
    .await
    .expect("sqlite table count");
    assert_eq!(tables_count, 2, "SQLite 建表数应为 2（users + orders）");
    let (fk_count,): (i64,) =
        sqlx::query_as("SELECT COUNT(*) FROM pragma_foreign_key_list('orders')")
            .fetch_one(&pool)
            .await
            .expect("sqlite fk count");
    assert_eq!(fk_count, 1, "SQLite 外键数应为 1（orders.user_id → users.id）");
    drop(pool);
    let _ = std::fs::remove_file(&db_path);

    verify_reporter::report_pass("UT-PC-14", 0);
}
