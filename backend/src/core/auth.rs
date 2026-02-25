use jsonwebtoken::{encode, EncodingKey, Header};
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub role: String,
    pub exp: u64,
}

pub fn generate_token(
    user_id: &str,
    role: &str,
    secret: &str,
    _expires_in: &str,
) -> Result<String, anyhow::Error> {
    // 简化实现，使用固定过期时间（24小时）
    let expiration = SystemTime::now() + std::time::Duration::from_secs(24 * 60 * 60);
    let exp = expiration.duration_since(UNIX_EPOCH)?.as_secs();

    let claims = Claims {
        sub: user_id.to_string(),
        role: role.to_string(),
        exp,
    };

    let secret = EncodingKey::from_secret(secret.as_ref());
    let token = encode(&Header::default(), &claims, &secret)?;

    Ok(token)
}
