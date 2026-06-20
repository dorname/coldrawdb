mod jwt;
mod password;

pub use jwt::{access_ttl_secs, sign_access_token, verify_access_token};
pub use password::{hash_password, password_meets_policy, verify_password};

use sea_orm::{ConnectionTrait, DatabaseBackend, DatabaseConnection, Statement, TransactionTrait};
use sha2::{Digest, Sha256};
use uuid::Uuid;

use crate::error::DrawDBError;
use crate::next_id;

#[derive(Debug, Clone)]
pub struct UserRow {
    pub id: String,
    pub email: String,
    pub display_name: Option<String>,
    pub email_verified_at: Option<String>,
}

#[derive(Debug, Clone)]
pub struct RegisterResult {
    pub user_id: String,
    pub email: String,
    pub status: &'static str,
}

#[derive(Debug, Clone)]
pub struct LoginResult {
    pub access_token: String,
    pub expires_in: i64,
    pub refresh_token: String,
    pub refresh_max_age_secs: i64,
}


fn is_valid_email(email: &str) -> bool {
    if email.is_empty() || email.len() > 255 {
        return false;
    }
    let Some(at) = email.find('@') else {
        return false;
    };
    at > 0 && at + 1 < email.len() && email[at + 1..].contains('.')
}

fn token_hash(raw: &str) -> String {
    let digest = Sha256::digest(raw.as_bytes());
    hex::encode(digest)
}

fn refresh_ttl_secs(remember_device: bool) -> i64 {
    if remember_device {
        30 * 24 * 3600
    } else {
        7 * 24 * 3600
    }
}

fn generate_refresh_token() -> String {
    Uuid::new_v4().to_string().replace('-', "") + &Uuid::new_v4().to_string().replace('-', "")
}

pub async fn register_user(
    db: &DatabaseConnection,
    email: &str,
    password: &str,
    display_name: Option<&str>,
) -> Result<RegisterResult, AuthServiceError> {
    if !is_valid_email(email) {
        return Err(AuthServiceError::Validation {
            fields: vec![("email".into(), "邮箱格式无效".into())],
        });
    }
    if !password_meets_policy(password) {
        return Err(AuthServiceError::Validation {
            fields: vec![(
                "password".into(),
                "密码至少 8 位且包含字母和数字".into(),
            )],
        });
    }
    if let Some(name) = display_name {
        if name.len() > 32 {
            return Err(AuthServiceError::Validation {
                fields: vec![("displayName".into(), "显示名称最多 32 字符".into())],
            });
        }
    }

    let existing = db
        .query_one(Statement::from_sql_and_values(
            DatabaseBackend::Sqlite,
            "SELECT id FROM user WHERE email = ? LIMIT 1",
            vec![email.into()],
        ))
        .await
        .map_err(|e| AuthServiceError::Db(DrawDBError::DatabaseError(e)))?;

    if existing.is_some() {
        return Err(AuthServiceError::EmailExists);
    }

    let user_id = Uuid::new_v4().to_string();
    let password_hash = hash_password(password).map_err(|e| AuthServiceError::Internal(e.to_string()))?;

    let tx = db.begin().await.map_err(|e| AuthServiceError::Db(DrawDBError::DatabaseError(e)))?;
    tx.execute(Statement::from_sql_and_values(
        DatabaseBackend::Sqlite,
        "INSERT INTO user(id, email, password_hash, display_name, email_verified_at, created_at, updated_at) VALUES(?, ?, ?, ?, NULL, strftime('%Y-%m-%dT%H:%M:%fZ', 'now'), strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))",
        vec![
            user_id.clone().into(),
            email.into(),
            password_hash.into(),
            display_name.map(|s| s.to_string()).into(),
        ],
    ))
    .await
    .map_err(|e| AuthServiceError::Db(DrawDBError::DatabaseError(e)))?;
    tx.commit().await.map_err(|e| AuthServiceError::Db(DrawDBError::DatabaseError(e)))?;

    Ok(RegisterResult {
        user_id,
        email: email.to_string(),
        status: "active",
    })
}

pub async fn login_user(
    db: &DatabaseConnection,
    email: &str,
    password: &str,
    remember_device: bool,
) -> Result<LoginResult, AuthServiceError> {
    if !is_valid_email(email) {
        return Err(AuthServiceError::InvalidCredentials);
    }

    let row = db
        .query_one(Statement::from_sql_and_values(
            DatabaseBackend::Sqlite,
            "SELECT id, password_hash FROM user WHERE email = ? LIMIT 1",
            vec![email.into()],
        ))
        .await
        .map_err(|e| AuthServiceError::Db(DrawDBError::DatabaseError(e)))?;

    let Some(row) = row else {
        tokio::time::sleep(std::time::Duration::from_millis(200)).await;
        return Err(AuthServiceError::InvalidCredentials);
    };

    let user_id: String = row.try_get("", "id").map_err(|e| AuthServiceError::Internal(e.to_string()))?;
    let password_hash: String = row
        .try_get("", "password_hash")
        .map_err(|e| AuthServiceError::Internal(e.to_string()))?;

    let ok = verify_password(password, &password_hash).map_err(|e| AuthServiceError::Internal(e.to_string()))?;
    if !ok {
        tokio::time::sleep(std::time::Duration::from_millis(200)).await;
        return Err(AuthServiceError::InvalidCredentials);
    }

    issue_tokens(db, &user_id, remember_device).await
}

async fn issue_tokens(
    db: &DatabaseConnection,
    user_id: &str,
    remember_device: bool,
) -> Result<LoginResult, AuthServiceError> {
    let (access_token, expires_in) =
        sign_access_token(user_id).map_err(|e| AuthServiceError::Internal(e.to_string()))?;
    let refresh_token = generate_refresh_token();
    let hash = token_hash(&refresh_token);
    let token_id = next_id();
    let ttl = refresh_ttl_secs(remember_device);
    let expires_at = (chrono::Utc::now() + chrono::Duration::seconds(ttl)).to_rfc3339();

    db.execute(Statement::from_sql_and_values(
        DatabaseBackend::Sqlite,
        "INSERT INTO auth_token(id, user_id, token_hash, expires_at, revoked_at, created_at) VALUES(?, ?, ?, ?, NULL, strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))",
        vec![
            token_id.into(),
            user_id.into(),
            hash.into(),
            expires_at.into(),
        ],
    ))
    .await
    .map_err(|e| AuthServiceError::Db(DrawDBError::DatabaseError(e)))?;

    Ok(LoginResult {
        access_token,
        expires_in,
        refresh_token,
        refresh_max_age_secs: ttl,
    })
}

pub async fn refresh_access_token(
    db: &DatabaseConnection,
    raw_refresh_token: &str,
) -> Result<LoginResult, AuthServiceError> {
    let hash = token_hash(raw_refresh_token);
    let row = db
        .query_one(Statement::from_sql_and_values(
            DatabaseBackend::Sqlite,
            "SELECT id, user_id, expires_at, revoked_at FROM auth_token WHERE token_hash = ? LIMIT 1",
            vec![hash.into()],
        ))
        .await
        .map_err(|e| AuthServiceError::Db(DrawDBError::DatabaseError(e)))?;

    let Some(row) = row else {
        return Err(AuthServiceError::RefreshInvalid);
    };

    let token_id: String = row.try_get("", "id").map_err(|e| AuthServiceError::Internal(e.to_string()))?;
    let user_id: String = row.try_get("", "user_id").map_err(|e| AuthServiceError::Internal(e.to_string()))?;
    let expires_at: String = row.try_get("", "expires_at").map_err(|e| AuthServiceError::Internal(e.to_string()))?;
    let revoked_at: Option<String> = row.try_get("", "revoked_at").ok();

    if revoked_at.is_some() {
        return Err(AuthServiceError::RefreshInvalid);
    }

    let exp = chrono::DateTime::parse_from_rfc3339(&expires_at)
        .map_err(|e| AuthServiceError::Internal(e.to_string()))?
        .with_timezone(&chrono::Utc);
    if exp < chrono::Utc::now() {
        return Err(AuthServiceError::RefreshInvalid);
    }

    // rotate: revoke old token
    db.execute(Statement::from_sql_and_values(
        DatabaseBackend::Sqlite,
        "UPDATE auth_token SET revoked_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now') WHERE id = ?",
        vec![token_id.into()],
    ))
    .await
    .map_err(|e| AuthServiceError::Db(DrawDBError::DatabaseError(e)))?;

    issue_tokens(db, &user_id, false).await
}

pub async fn logout_user(db: &DatabaseConnection, raw_refresh_token: Option<&str>) -> Result<(), AuthServiceError> {
    if let Some(raw) = raw_refresh_token {
        let hash = token_hash(raw);
        db.execute(Statement::from_sql_and_values(
            DatabaseBackend::Sqlite,
            "UPDATE auth_token SET revoked_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now') WHERE token_hash = ? AND revoked_at IS NULL",
            vec![hash.into()],
        ))
        .await
        .map_err(|e| AuthServiceError::Db(DrawDBError::DatabaseError(e)))?;
    }
    Ok(())
}

pub async fn get_user_by_id(db: &DatabaseConnection, user_id: &str) -> Result<UserRow, AuthServiceError> {
    let row = db
        .query_one(Statement::from_sql_and_values(
            DatabaseBackend::Sqlite,
            "SELECT id, email, display_name, email_verified_at FROM user WHERE id = ? LIMIT 1",
            vec![user_id.into()],
        ))
        .await
        .map_err(|e| AuthServiceError::Db(DrawDBError::DatabaseError(e)))?;

    let Some(row) = row else {
        return Err(AuthServiceError::Unauthorized);
    };

    Ok(UserRow {
        id: row.try_get("", "id").map_err(|e| AuthServiceError::Internal(e.to_string()))?,
        email: row.try_get("", "email").map_err(|e| AuthServiceError::Internal(e.to_string()))?,
        display_name: row.try_get("", "display_name").ok(),
        email_verified_at: row.try_get("", "email_verified_at").ok(),
    })
}

#[derive(Debug)]
pub enum AuthServiceError {
    EmailExists,
    InvalidCredentials,
    RefreshInvalid,
    Unauthorized,
    Validation { fields: Vec<(String, String)> },
    Internal(String),
    Db(DrawDBError),
}

impl From<DrawDBError> for AuthServiceError {
    fn from(e: DrawDBError) -> Self {
        AuthServiceError::Db(e)
    }
}
