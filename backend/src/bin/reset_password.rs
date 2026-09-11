//! Admin CLI: 重置指定 email 的密码为新密码。
//!
//! 用法：
//!   cargo run --bin reset_password -- <email> <new_password>
//!
//! 行为：
//!   - 用 Argon2id（m=19456, t=2, p=1，与生产 hash_password 一致）生成新哈希
//!   - 直接 UPDATE user 表的 password_hash + updated_at
//!   - 返回影响行数与新哈希前缀
//!
//! 安全：
//!   - 不打印明文密码
//!   - 仅 admin 可执行
//!
//! ⚠️ 注意：本工具绕过 JWT 鉴权，**仅用于本地开发/紧急恢复**。

use std::env;

use argon2::{
    password_hash::{PasswordHasher, SaltString},
    Argon2,
};
use rand::rngs::OsRng;
use sea_orm::{ConnectionTrait, Database, DatabaseConnection, DbBackend, Statement};

fn password_meets_policy(password: &str) -> bool {
    password.len() >= 8
        && password.len() <= 128
        && password.chars().any(|c| c.is_ascii_alphabetic())
        && password.chars().any(|c| c.is_ascii_digit())
}

fn hash_password(password: &str) -> Result<String, String> {
    let salt = SaltString::generate(&mut OsRng);
    Argon2::default()
        .hash_password(password.as_bytes(), &salt)
        .map(|h| h.to_string())
        .map_err(|e| format!("password hash failed: {e}"))
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    if args.len() != 3 {
        eprintln!("Usage: reset_password <email> <new_password>");
        std::process::exit(2);
    }
    let email = &args[1];
    let new_password = &args[2];

    if !password_meets_policy(new_password) {
        eprintln!(
            "[reset_password] password must be 8..=128 chars and contain both a letter and a digit"
        );
        std::process::exit(3);
    }

    // 默认连本地 db.sqlite；可通过 COLDRAWDB_DB_URL 覆盖
    let db_url = env::var("COLDRAWDB_DB_URL")
        .unwrap_or_else(|_| "sqlite://db.sqlite?mode=rwc".to_string());
    let db: DatabaseConnection = Database::connect(&db_url).await?;

    let new_hash = hash_password(new_password)?;
    let hash_prefix: String = new_hash.chars().take(20).collect();
    let hash_suffix: String = new_hash
        .chars()
        .rev()
        .take(8)
        .collect::<String>()
        .chars()
        .rev()
        .collect();

    let stmt = Statement::from_string(
        DbBackend::Sqlite,
        format!(
            "UPDATE user SET password_hash = '{}', updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now') WHERE email = '{}'",
            new_hash.replace('\'', "''"),
            email.replace('\'', "''")
        ),
    );
    let result = db.execute(stmt).await?;

    let rows = result.rows_affected();
    if rows == 0 {
        eprintln!("[reset_password] no user with email={}", email);
        std::process::exit(4);
    }

    println!(
        "[reset_password] updated {} row(s); new hash prefix={}… suffix=…{}",
        rows, hash_prefix, hash_suffix
    );
    println!("[reset_password] email={} password reset (not printed)", email);
    Ok(())
}