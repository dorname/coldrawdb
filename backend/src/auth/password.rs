use argon2::{
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use rand::rngs::OsRng;

use crate::error::DrawDBError;

pub fn hash_password(password: &str) -> Result<String, DrawDBError> {
    let salt = SaltString::generate(&mut OsRng);
    Argon2::default()
        .hash_password(password.as_bytes(), &salt)
        .map(|h| h.to_string())
        .map_err(|e| DrawDBError::OtherError(format!("password hash failed: {e}")))
}

pub fn verify_password(password: &str, password_hash: &str) -> Result<bool, DrawDBError> {
    let parsed = PasswordHash::new(password_hash)
        .map_err(|e| DrawDBError::OtherError(format!("invalid password hash: {e}")))?;
    Ok(Argon2::default()
        .verify_password(password.as_bytes(), &parsed)
        .is_ok())
}

pub fn password_meets_policy(password: &str) -> bool {
    password.len() >= 8
        && password.len() <= 128
        && password.chars().any(|c| c.is_ascii_alphabetic())
        && password.chars().any(|c| c.is_ascii_digit())
}
