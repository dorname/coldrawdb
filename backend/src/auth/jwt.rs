use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicI64, Ordering};

use crate::error::DrawDBError;

/// issue #16（fix-remote-github-issues-7-18 批次 E）：access token TTL 配置化。
/// - 默认值 3600 秒（1 小时，用户已拍板）；
/// - 合法区间 300～86400 秒（见 logos/resources/api/auth.yaml 尾部「TTL 配置化不变量」）；
/// - 生效值由 init() 在启动时从 `[auth] access_ttl_secs`（或环境变量
///   `COLDRAWDB_ACCESS_TTL_SECS` 覆盖）校验后下发；
/// - 未调用 init() 的测试进程保持默认值 3600。
pub const DEFAULT_ACCESS_TTL_SECS: i64 = 3600;
pub const MIN_ACCESS_TTL_SECS: i64 = 300;
pub const MAX_ACCESS_TTL_SECS: i64 = 86400;

/// 运行时生效的 access TTL（秒）。进程内唯一权威来源，
/// 保证 `/auth/login` 与 `/auth/refresh` 的 expiresIn 与 JWT 的 exp - iat 始终一致。
static RUNTIME_ACCESS_TTL_SECS: AtomicI64 = AtomicI64::new(DEFAULT_ACCESS_TTL_SECS);

/// 由 init() 在配置校验通过后调用，下发运行时 TTL
pub fn set_access_ttl_secs(ttl_secs: i64) {
    RUNTIME_ACCESS_TTL_SECS.store(ttl_secs, Ordering::SeqCst);
}

/// 当前生效的 access TTL（秒）
pub fn access_ttl_secs() -> i64 {
    RUNTIME_ACCESS_TTL_SECS.load(Ordering::SeqCst)
}

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

/// 按当前生效配置签发 access token，返回 (token, ttl_secs)
pub fn sign_access_token(user_id: &str) -> Result<(String, i64), DrawDBError> {
    sign_access_token_with_ttl(user_id, access_ttl_secs())
}

/// 显式 TTL 版本：供配置层单测断言「签发 JWT 的 exp - iat == 指定 TTL」（UT-S03-08），
/// 以及构造已过期 token（ST-S03-02）。生产路径应走 sign_access_token。
pub fn sign_access_token_with_ttl(
    user_id: &str,
    ttl_secs: i64,
) -> Result<(String, i64), DrawDBError> {
    let now = chrono::Utc::now().timestamp();
    let exp = now + ttl_secs;
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
    Ok((token, ttl_secs))
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
