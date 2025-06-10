use crate::error::DrawDBError;
use once_cell::sync::OnceCell;
use sea_orm::{ConnectionTrait, Database, DatabaseConnection, TransactionTrait};
use serde::{Deserialize, Serialize};
use std::sync::RwLock;
static SERVER_CONFIG: OnceCell<RwLock<ServerConfig>> = OnceCell::new();

/// 初始化数据库
pub async fn init_table(init_sql_path: &str, db: &DatabaseConnection) -> Result<(), DrawDBError> {
    let init_sql = std::fs::read_to_string(init_sql_path)?;

    // 按分号分割 SQL 语句
    let statements: Vec<&str> = init_sql
        .split(';')
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .collect();

    // 开始事务
    let tx = db.begin().await?;

    // 逐条执行 SQL 语句
    for statement in statements {
        tx.execute(sea_orm::Statement::from_sql_and_values(
            sea_orm::DatabaseBackend::Sqlite,
            statement,
            vec![],
        ))
        .await?;
    }

    // 提交事务
    tx.commit().await?;
    Ok(())
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
pub async fn init() -> Result<Option<DatabaseConnection>, DrawDBError> {
    let mut config = read_config("config.toml");
    let server_config = config.server.clone();
    SERVER_CONFIG
        .set(RwLock::new(server_config))
        .expect("Failed to initialize config");
    // 如果数据库文件不存在，则创建数据库文件
    if !std::path::Path::new(&config.database.path).exists() {
        // 创建数据库文件
        std::fs::File::create(&config.database.path)?;
    }
    
    // 配置连接池
    let db = Database::connect(format!(
        "sqlite://{}?",
        &config.database.path,
    )).await?;
    
    // 如果初始化开关为true，则初始化数据库
    if config.options.init_db {
        init_table(&config.database.init_sql_path, &db).await?;
        // 初始化数据库成功
        // 修改配置文件
        config.options.init_db = false;
        std::fs::write("config.toml", toml::to_string(&config).unwrap())?;
    }
    Ok(Some(db))
}

#[cfg(test)]
mod test {
    use super::*;
    #[actix_web::test]
    async fn test_init() {
        let db = Database::connect(format!("sqlite://{}", "test.sqlite")).await.unwrap();
        init_table("init.sql", &db).await.unwrap();
    }
}