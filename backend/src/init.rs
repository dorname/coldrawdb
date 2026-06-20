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

/// 获取服务器配置实例
pub fn get_config() -> &'static RwLock<ServerConfig> {
    SERVER_CONFIG.get().expect("Config not initialized")
}

/// 初始化全局配置
/// mode: true 测试模式，false 生产模式
pub async fn init(mode: bool) -> Result<Option<DatabaseConnection>, DrawDBError> {
    let mut config = read_config("config.toml");
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
        // 若由配置触发 init_db，则回写关闭初始化开关
        if config.options.init_db {
            config.options.init_db = false;
            std::fs::write("config.toml", toml::to_string(&config).unwrap())?;
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
