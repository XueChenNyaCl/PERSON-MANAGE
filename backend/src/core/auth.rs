use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

/// 操作者类型
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "lowercase")]
pub enum OperatorType {
    System,   // 系统用户
    Admin,    // Admin 用户
    User,     // 普通用户
    ChatAI,   // 聊天AI
    SysAI,    // 系统AI
    IoT,      // 物联网设备
    Scerm,    // 大屏用户
}

impl std::fmt::Display for OperatorType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OperatorType::System => write!(f, "system"),
            OperatorType::Admin => write!(f, "admin"),
            OperatorType::User => write!(f, "user"),
            OperatorType::ChatAI => write!(f, "chatai"),
            OperatorType::SysAI => write!(f, "sysai"),
            OperatorType::IoT => write!(f, "iot"),
            OperatorType::Scerm => write!(f, "scerm"),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    pub sub: String,           // 用户ID
    pub username: String,      // 用户名
    pub role: String,          // 用户角色
    pub operator_type: String, // 操作者类型: system, admin, user, chatai, sysai, iot, scerm
    pub operator_name: String, // 操作者名称: admin:zhangsan, 501001, system
    pub exp: u64,              // 过期时间
}

/// 生成普通用户 Token
pub fn generate_token(
    user_id: &str,
    username: &str,
    role: &str,
    secret: &str,
    expires_in_hours: u64,
) -> Result<String, anyhow::Error> {
    generate_token_with_operator(
        user_id,
        username,
        role,
        "user",
        username,
        secret,
        expires_in_hours,
    )
}

/// 生成带操作者信息的 Token
pub fn generate_token_with_operator(
    user_id: &str,
    username: &str,
    role: &str,
    operator_type: &str,
    operator_name: &str,
    secret: &str,
    expires_in_hours: u64,
) -> Result<String, anyhow::Error> {
    let expiration = SystemTime::now() + Duration::from_secs(expires_in_hours * 3600);
    let exp = expiration.duration_since(UNIX_EPOCH)?.as_secs();

    let claims = Claims {
        sub: user_id.to_string(),
        username: username.to_string(),
        role: role.to_string(),
        operator_type: operator_type.to_string(),
        operator_name: operator_name.to_string(),
        exp,
    };

    let secret = EncodingKey::from_secret(secret.as_ref());
    let token = encode(&Header::default(), &claims, &secret)?;

    Ok(token)
}

/// 生成 Admin 用户 Token
pub fn generate_admin_token(
    user_id: &str,
    username: &str,
    role: &str,
    secret: &str,
    expires_in_hours: u64,
) -> Result<String, anyhow::Error> {
    let operator_name = format!("admin:{}", username);
    generate_token_with_operator(
        user_id,
        username,
        role,
        "admin",
        &operator_name,
        secret,
        expires_in_hours,
    )
}

/// 生成特殊用户 Token (IoT/Scerm)
pub fn generate_special_user_token(
    user_id: &str,
    identifier: &str,
    user_type: &str,
    secret: &str,
    expires_in_hours: u64,
) -> Result<String, anyhow::Error> {
    generate_token_with_operator(
        user_id,
        identifier,
        user_type,
        user_type,
        identifier,
        secret,
        expires_in_hours,
    )
}

pub fn verify_token(token: &str, secret: &str) -> Result<Claims, anyhow::Error> {
    let secret = DecodingKey::from_secret(secret.as_ref());
    let validation = Validation::default();

    let token_data = decode::<Claims>(token, &secret, &validation)?;
    Ok(token_data.claims)
}

/// 获取操作者显示名称（用于日志）
/// 格式: [time][operator_name][action][details]
pub fn get_operator_display_name(claims: &Claims) -> String {
    claims.operator_name.clone()
}

/// 检查是否是 Admin 操作者
pub fn is_admin_operator(claims: &Claims) -> bool {
    claims.operator_type == "admin" || claims.role == "admin"
}

/// 检查是否是 System 操作者
pub fn is_system_operator(claims: &Claims) -> bool {
    claims.operator_type == "system"
}

/// 检查是否是特殊用户 (IoT/Scerm)
pub fn is_special_user(claims: &Claims) -> bool {
    matches!(claims.operator_type.as_str(), "iot" | "scerm" | "chatai" | "sysai")
}
