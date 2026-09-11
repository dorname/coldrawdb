use crate::error::DrawDBError;
use once_cell::sync::OnceCell;
use sea_orm::{ConnectionTrait, Database, DatabaseConnection, TransactionTrait};
use serde::{Deserialize, Serialize};
use std::sync::RwLock;

static SERVER_CONFIG: OnceCell<RwLock<ServerConfig>> = OnceCell::new();

/// 执行一段 SQL 脚本（按 ; 分隔）
async fn execute_sql_script(db: &DatabaseConnection, sql: &str) -> Result<(), DrawDBError> {
    let statements: Vec<&str> = sql
        .split(';')
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .collect();

    let tx = db.begin().await?;
    for statement in statements {
        tx.execute(sea_orm::Statement::from_sql_and_values(
            sea_orm::DatabaseBackend::Sqlite,
            statement,
            vec![],
        ))
        .await?;
    }
    tx.commit().await?;
    Ok(())
}

/// 初始化数据库（基线 schema）
pub async fn init_table(init_sql_path: &str, db: &DatabaseConnection) -> Result<(), DrawDBError> {
    let init_sql = std::fs::read_to_string(init_sql_path)?;
    execute_sql_script(db, &init_sql).await?;
    Ok(())
}

/// 应用目录下的 *.up.sql 迁移文件（按文件名升序）
pub async fn apply_migrations(
    migration_dir: &str,
    db: &DatabaseConnection,
) -> Result<(), DrawDBError> {
    db.execute(sea_orm::Statement::from_sql_and_values(
        sea_orm::DatabaseBackend::Sqlite,
        "CREATE TABLE IF NOT EXISTS schema_migrations (version TEXT PRIMARY KEY, applied_at TEXT NOT NULL DEFAULT (datetime('now')))",
        vec![],
    ))
    .await?;

    let mut migration_files = std::fs::read_dir(migration_dir)?
        .filter_map(|entry| entry.ok())
        .filter_map(|entry| {
            let path = entry.path();
            let file_name = path.file_name()?.to_string_lossy().to_string();
            if file_name.ends_with(".up.sql") {
                Some(path)
            } else {
                None
            }
        })
        .collect::<Vec<_>>();

    migration_files.sort();

    for file in migration_files {
        let file_name = file
            .file_name()
            .ok_or_else(|| DrawDBError::OtherError("invalid migration file name".to_string()))?
            .to_string_lossy()
            .to_string();
        let version = file_name.trim_end_matches(".up.sql");
        let exists_sql = format!(
            "SELECT 1 FROM schema_migrations WHERE version = '{}' LIMIT 1",
            version.replace('\'', "''")
        );

        let already_applied = db
            .query_one(sea_orm::Statement::from_sql_and_values(
                sea_orm::DatabaseBackend::Sqlite,
                exists_sql,
                vec![],
            ))
            .await?
            .is_some();

        if already_applied {
            continue;
        }

        let sql = std::fs::read_to_string(&file)?;
        execute_sql_script(db, &sql).await?;

        let insert_sql = format!(
            "INSERT INTO schema_migrations(version) VALUES('{}')",
            version.replace('\'', "''")
        );

        db.execute(sea_orm::Statement::from_sql_and_values(
            sea_orm::DatabaseBackend::Sqlite,
            insert_sql,
            vec![],
        ))
        .await?;
    }

    Ok(())
}

async fn table_exists(db: &DatabaseConnection, table_name: &str) -> Result<bool, DrawDBError> {
    let sql = format!(
        "SELECT 1 FROM sqlite_master WHERE type='table' AND name='{}' LIMIT 1",
        table_name.replace('\'', "''")
    );
    let row = db
        .query_one(sea_orm::Statement::from_sql_and_values(
            sea_orm::DatabaseBackend::Sqlite,
            sql,
            vec![],
        ))
        .await?;
    Ok(row.is_some())
}

/// 配置文件结构体
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Config {
    pub database: DatabaseConfig,
    pub server: ServerConfig,
    pub options: OptionsConfig,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct DatabaseConfig {
    pub path: String,
    pub init_sql_path: String,
    pub test_path: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ServerConfig {
    pub port: u16,
    pub host: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct OptionsConfig {
    pub init_db: bool,
}

/// 读取配置文件config.toml
/// 返回配置文件的配置全局变量
pub fn read_config(config_path: &str) -> Config {
    let config = toml::from_str::<Config>(&std::fs::read_to_string(config_path).unwrap()).unwrap();
    config
}

/// feat-docker-compose-deploy（core-01 部署方案 §4.3）：环境变量覆盖纯函数。
///
/// `COLDRAWDB_BIND_ADDR` 形如 `host:port`（取最后一个冒号切分，端口必须可解析为 u16）。
pub fn parse_bind_addr(addr: &str) -> Option<(String, u16)> {
    let (host, port) = addr.rsplit_once(':')?;
    let port = port.parse::<u16>().ok()?;
    if host.is_empty() {
        return None;
    }
    Some((host.to_string(), port))
}

/// `sqlite:///data/x.db?mode=rwc` → `/data/x.db`；`sqlite://data/db.sqlite` → `data/db.sqlite`。
/// 不带 `sqlite://` 前缀的按裸路径处理；空结果返回 None。
pub fn db_path_from_url(url: &str) -> Option<String> {
    let stripped = url.strip_prefix("sqlite://").unwrap_or(url);
    let path = stripped.split('?').next().unwrap_or(stripped);
    if path.is_empty() {
        None
    } else {
        Some(path.to_string())
    }
}

/// 应用 `COLDRAWDB_BIND_ADDR` / `COLDRAWDB_DB_URL` 环境变量覆盖（仅修改内存中的 Config 副本，
/// 不回写 config.toml）。非法值静默忽略并保留文件配置。
pub fn apply_env_overrides(config: &mut Config) {
    apply_env_overrides_from(
        config,
        std::env::var("COLDRAWDB_BIND_ADDR").ok().as_deref(),
        std::env::var("COLDRAWDB_DB_URL").ok().as_deref(),
    );
}

/// 显式参数版本（便于单测，避免 set_var 的跨测试竞态）。
pub fn apply_env_overrides_from(config: &mut Config, bind_addr: Option<&str>, db_url: Option<&str>) {
    if let Some(addr) = bind_addr {
        if let Some((host, port)) = parse_bind_addr(addr) {
            config.server.host = host;
            config.server.port = port;
        }
    }
    if let Some(url) = db_url {
        if let Some(path) = db_path_from_url(url) {
            config.database.path = path;
        }
    }
}

/// 获取服务器配置实例
pub fn get_config() -> &'static RwLock<ServerConfig> {
    SERVER_CONFIG.get().expect("Config not initialized")
}

/// 初始化全局配置
/// mode: true 测试模式，false 生产模式
pub async fn init(mode: bool) -> Result<Option<DatabaseConnection>, DrawDBError> {
    let mut config = read_config("config.toml");
    // feat-docker-compose-deploy：env 覆盖只作用于运行时副本，回写 config.toml 时用原始值
    let file_config = config.clone();
    apply_env_overrides(&mut config);
    let server_config = config.server.clone();
    SERVER_CONFIG
        .set(RwLock::new(server_config))
        .expect("Failed to initialize config");
    let path = if mode {
        config.database.test_path.clone()
    } else {
        config.database.path.clone()
    };
    // 如果数据库文件不存在或者初始化开关为true，则创建数据库文件
    if !std::path::Path::new(&path).exists() || config.options.init_db {
        // 创建数据库文件
        std::fs::File::create(&path)?;
    }

    // 配置连接池
    let db = Database::connect(format!("sqlite://{}?", &path)).await?;

    // 初始化基线 schema：
    // 1) 配置显式要求初始化
    // 2) 或者数据库中尚不存在基线表（首次启动且未置 init_db）
    let baseline_exists = table_exists(&db, "diagram").await?;
    if config.options.init_db || !baseline_exists {
        init_table(&config.database.init_sql_path, &db).await?;
        // 若由配置触发 init_db，则回写关闭初始化开关（回写原始文件配置，不落 env 覆盖值）
        if config.options.init_db {
            let mut persist = file_config;
            persist.options.init_db = false;
            std::fs::write("config.toml", toml::to_string(&persist).unwrap())?;
        }
    }

    // 统一执行 migration（幂等）
    if std::path::Path::new("migrations").exists() {
        apply_migrations("migrations", &db).await?;
    }

    Ok(Some(db))
}

#[cfg(test)]
mod test {
    use super::*;

    #[actix_web::test]
    async fn test_init() {
        init(true).await.unwrap();
    }

    /// UT-DP-01：环境变量覆盖纯函数（feat-docker-compose-deploy，core-PV §5）
    #[test]
    fn ut_dp_01_env_override_parsers() {
        assert_eq!(
            parse_bind_addr("0.0.0.0:3000"),
            Some(("0.0.0.0".to_string(), 3000))
        );
        assert_eq!(
            parse_bind_addr("127.0.0.1:8080"),
            Some(("127.0.0.1".to_string(), 8080))
        );
        assert_eq!(parse_bind_addr("no-colon"), None);
        assert_eq!(parse_bind_addr("0.0.0.0:not-a-port"), None);
        assert_eq!(parse_bind_addr("0.0.0.0:99999"), None);
        assert_eq!(parse_bind_addr(":3000"), None);

        assert_eq!(
            db_path_from_url("sqlite:///data/coldrawdb.db?mode=rwc"),
            Some("/data/coldrawdb.db".to_string())
        );
        assert_eq!(
            db_path_from_url("sqlite://data/db.sqlite"),
            Some("data/db.sqlite".to_string())
        );
        assert_eq!(
            db_path_from_url("plain/path.db"),
            Some("plain/path.db".to_string())
        );
        assert_eq!(db_path_from_url("sqlite://?mode=rwc"), None);

        // apply_env_overrides_from 显式参数版本：覆盖生效
        let mut config = Config {
            database: DatabaseConfig {
                path: "db.sqlite".to_string(),
                init_sql_path: "init.sql".to_string(),
                test_path: "test.sqlite".to_string(),
            },
            server: ServerConfig {
                port: 3000,
                host: "127.0.0.1".to_string(),
            },
            options: OptionsConfig { init_db: false },
        };
        apply_env_overrides_from(
            &mut config,
            Some("0.0.0.0:4000"),
            Some("sqlite:///data/coldrawdb.db?mode=rwc"),
        );
        assert_eq!(config.server.host, "0.0.0.0");
        assert_eq!(config.server.port, 4000);
        assert_eq!(config.database.path, "/data/coldrawdb.db");
        // 非法值静默忽略
        apply_env_overrides_from(&mut config, Some("bad-addr"), Some("sqlite://"));
        assert_eq!(config.server.host, "0.0.0.0");
        assert_eq!(config.server.port, 4000);
        assert_eq!(config.database.path, "/data/coldrawdb.db");
        crate::verify_reporter::report_pass("UT-DP-01", 0);
    }

    #[actix_web::test]
    async fn test_phase1_migration_applied_and_idempotent() {
        let db_path = format!(
            "{}/drawdb_phase1_{}.sqlite",
            std::env::temp_dir().display(),
            uuid::Uuid::new_v4()
        );

        if std::path::Path::new(&db_path).exists() {
            std::fs::remove_file(&db_path).unwrap();
        }

        std::fs::File::create(&db_path).unwrap();
        let db = Database::connect(format!("sqlite://{}?", db_path)).await.unwrap();

        init_table("init.sql", &db).await.unwrap();
        apply_migrations("migrations", &db).await.unwrap();
        // second apply should be no-op
        apply_migrations("migrations", &db).await.unwrap();

        let migration_count = db
            .query_one(sea_orm::Statement::from_sql_and_values(
                sea_orm::DatabaseBackend::Sqlite,
                "SELECT COUNT(1) as c FROM schema_migrations WHERE version='0001_phase1_schema'",
                vec![],
            ))
            .await
            .unwrap()
            .unwrap()
            .try_get::<i64>("", "c")
            .unwrap();
        assert_eq!(migration_count, 1);

        let updated_at_exists = db
            .query_one(sea_orm::Statement::from_sql_and_values(
                sea_orm::DatabaseBackend::Sqlite,
                "SELECT COUNT(1) as c FROM pragma_table_info('diagram') WHERE name='updated_at'",
                vec![],
            ))
            .await
            .unwrap()
            .unwrap()
            .try_get::<i64>("", "c")
            .unwrap();
        assert_eq!(updated_at_exists, 1);

        let renamed_reference_exists = db
            .query_one(sea_orm::Statement::from_sql_and_values(
                sea_orm::DatabaseBackend::Sqlite,
                "SELECT COUNT(1) as c FROM pragma_table_info('diagram_link') WHERE name='reference_id'",
                vec![],
            ))
            .await
            .unwrap()
            .unwrap()
            .try_get::<i64>("", "c")
            .unwrap();
        assert_eq!(renamed_reference_exists, 1);

        let idx_exists = db
            .query_one(sea_orm::Statement::from_sql_and_values(
                sea_orm::DatabaseBackend::Sqlite,
                "SELECT COUNT(1) as c FROM sqlite_master WHERE type='index' AND name='idx_diagram_link_diagram_id'",
                vec![],
            ))
            .await
            .unwrap()
            .unwrap()
            .try_get::<i64>("", "c")
            .unwrap();
        assert_eq!(idx_exists, 1);

        std::fs::remove_file(&db_path).unwrap();
    }
}
