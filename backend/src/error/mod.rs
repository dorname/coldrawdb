use actix_web::{HttpResponse, ResponseError};

/// 错误处理模块
/// 定义自己的错误类型
#[derive(Debug, thiserror::Error)]
pub enum DrawDBError {
    /// 数据库错误
    #[error("数据库错误: {0}")]
    DatabaseError(#[from] sea_orm::DbErr),
    /// IO错误
    #[error("IO错误: {0}")]
    IoError(#[from] std::io::Error),
    /// 配置错误
    #[error("配置错误: {0}")]
    ConfigError(#[from] toml::de::Error),

    /// 其他错误
    #[error("其他错误: {0}")]
    OtherError(String),
}
impl ResponseError for DrawDBError {
    fn error_response(&self) -> HttpResponse {
        match self {
            DrawDBError::DatabaseError(e) => {
                HttpResponse::InternalServerError().json(format!("数据库错误: {}", e))
            }
            DrawDBError::IoError(e) => {
                HttpResponse::InternalServerError().json(format!("IO错误: {}", e))
            }
            DrawDBError::ConfigError(e) => {
                HttpResponse::InternalServerError().json(format!("配置错误: {}", e))
            }
            DrawDBError::OtherError(e) => {
                HttpResponse::InternalServerError().json(format!("其他错误: {}", e))
            }
        }
    }
}
