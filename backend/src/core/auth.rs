use jsonwebtoken::{encode, decode, EncodingKey, DecodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH, Duration};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    pub sub: String,      // 用户ID
    pub username: String, // 用户名
    pub role: String,     // 用户角色
    pub exp: u64,         // 过期时间
}

pub fn generate_token(
    user_id: &str,
    username: &str,
    role: &str,
    secret: &str,
    expires_in_hours: u64,
) -> Result<String, anyhow::Error> {
    let expiration = SystemTime::now() + Duration::from_secs(expires_in_hours * 3600);
    let exp = expiration.duration_since(UNIX_EPOCH)?.as_secs();
    
    let claims = Claims {
        sub: user_id.to_string(),
        username: username.to_string(),
        role: role.to_string(),
        exp,
    };
    
    let secret = EncodingKey::from_secret(secret.as_ref());
    let token = encode(&Header::default(), &claims, &secret)?;
    
    Ok(token)
}

pub fn verify_token(token: &str, secret: &str) -> Result<Claims, anyhow::Error> {
    let secret = DecodingKey::from_secret(secret.as_ref());
    let validation = Validation::default();
    
    let token_data = decode::<Claims>(token, &secret, &validation)?;
    Ok(token_data.claims)
}