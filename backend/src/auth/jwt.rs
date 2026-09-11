use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};

use crate::error::DrawDBError;

const ACCESS_TTL_SECS: i64 = 900;

#[derive(Debug, Serialize, Deserialize)]
pub struct AccessClaims {
    pub sub: String,
    pub exp: i64,
    pub iat: i64,
}

fn jwt_secret() -> String {
    std::env::var("COLDRAWDB_JWT_SECRET")
        .unwrap_or_else(|_| "coldrawdb-dev-jwt-secret-change-me".to_string())
}

pub fn sign_access_token(user_id: &str) -> Result<(String, i64), DrawDBError> {
    let now = chrono::Utc::now().timestamp();
    let exp = now + ACCESS_TTL_SECS;
    let claims = AccessClaims {
        sub: user_id.to_string(),
        exp,
        iat: now,
    };
    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(jwt_secret().as_bytes()),
    )
    .map_err(|e| DrawDBError::OtherError(format!("jwt sign failed: {e}")))?;
    Ok((token, ACCESS_TTL_SECS))
}

pub fn verify_access_token(token: &str) -> Result<AccessClaims, DrawDBError> {
    decode::<AccessClaims>(
        token,
        &DecodingKey::from_secret(jwt_secret().as_bytes()),
        &Validation::default(),
    )
    .map(|data| data.claims)
    .map_err(|e| DrawDBError::OtherError(format!("jwt verify failed: {e}")))
}

pub fn access_ttl_secs() -> i64 {
    ACCESS_TTL_SECS
}
