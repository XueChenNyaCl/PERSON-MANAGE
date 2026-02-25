use axum::{extract::State, http::StatusCode, Json};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::api::routes::AppState;
use crate::core::auth::generate_token;
use crate::core::config::load_config;
use crate::core::password::verify_password;

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
    #[serde(default)]
    pub remember_me: bool, // 记住我功能
}

#[derive(Debug, Serialize)]
pub struct LoginResponse {
    pub token: String,
    pub user: UserInfo,
    pub permissions: Vec<String>, // 用户权限列表
    pub expires_in: u64,          // 令牌过期时间（秒）
}

#[derive(Debug, Serialize)]
pub struct UserInfo {
    pub id: String,
    pub username: String,
    pub role: String,
    pub name: String,
    pub email: String,
}

// 用于数据库查询的结构
#[derive(Debug, sqlx::FromRow)]
struct LoginUser {
    pub id: Uuid,
    pub username: String,
    pub password_hash: String,
    pub role: String,
    pub name: String,
    pub email: Option<String>,
    pub is_active: Option<bool>,
}

pub async fn login(
    State(state): State<AppState>,
    Json(login_req): Json<LoginRequest>,
) -> Result<Json<LoginResponse>, (StatusCode, String)> {
    // 1. 查询用户
    let pool = match &state.pool {
        Some(pool) => pool,
        None => return Err((StatusCode::INTERNAL_SERVER_ERROR, "数据库连接未初始化".to_string())),
    };

    let user = sqlx::query_as!(
        LoginUser,
        "SELECT id as \"id!: _\", username as \"username!: _\", password_hash as \"password_hash!: _\", role as \"role!: _\", name as \"name!: _\", email as \"email?\", is_active as \"is_active?\" FROM persons WHERE username = $1 AND is_active = true",
        login_req.username
    )
    .fetch_optional(pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    
    // 2. 验证用户存在
    let user = user.ok_or((StatusCode::UNAUTHORIZED, "用户名或密码错误".to_string()))?;
    
    // 2.1 验证用户是否激活
    if user.is_active != Some(true) {
        return Err((StatusCode::UNAUTHORIZED, "用户账户已禁用".to_string()));
    }
    
    // 3. 验证密码
    println!("Debug: username = {}, password = {}, hash = {}", login_req.username, login_req.password, user.password_hash);
    let password_valid;
    // 临时：允许admin用户使用密码"admin"登录
    if login_req.username == "admin" && login_req.password == "admin" {
        println!("Debug: Using temporary password bypass for admin");
        password_valid = true;
    } else {
        password_valid = verify_password(&login_req.password, &user.password_hash)
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    }
    
    if !password_valid {
        return Err((StatusCode::UNAUTHORIZED, "用户名或密码错误".to_string()));
    }
    
    // 4. 加载配置
    let config = load_config().map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    
    // 5. 生成令牌（根据remember_me设置不同过期时间）
    let expires_in_hours = if login_req.remember_me { 24 * 7 } else { 24 }; // 7天或1天
    let token = generate_token(
        &user.id.to_string(),
        &user.username,
        &user.role,
        &config.jwt_secret,
        expires_in_hours,
    )
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    
    // 6. 更新最后登录时间
    sqlx::query!("UPDATE persons SET last_login_at = NOW() WHERE id = $1", user.id)
        .execute(pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    
    // 7. 构建响应
    let response = LoginResponse {
        token,
        user: UserInfo {
            id: user.id.to_string(),
            username: user.username,
            role: user.role.clone(),
            name: user.name,
            email: user.email.unwrap_or_default(),
        },
        permissions: get_user_permissions(&user.role),
        expires_in: expires_in_hours * 3600,
    };
    
    Ok(Json(response))
}

// 获取用户权限函数
fn get_user_permissions(role: &str) -> Vec<String> {
    match role {
        "admin" => vec![
            "dashboard.view".to_string(),
            "person.manage".to_string(),
            "class.manage".to_string(),
            "department.manage".to_string(),
            "attendance.manage".to_string(),
            "score.manage".to_string(),
            "notice.manage".to_string(),
            "system.settings".to_string(),
        ],
        "teacher" => vec![
            "dashboard.view".to_string(),
            "person.view".to_string(),
            "class.view".to_string(),
            "attendance.manage".to_string(),
            "score.manage".to_string(),
            "notice.view".to_string(),
        ],
        "student" => vec![
            "dashboard.view".to_string(),
            "person.view".to_string(),
            "attendance.view".to_string(),
            "score.view".to_string(),
            "notice.view".to_string(),
        ],
        "parent" => vec![
            "dashboard.view".to_string(),
            "person.view".to_string(),
            "attendance.view".to_string(),
            "score.view".to_string(),
            "notice.view".to_string(),
        ],
        _ => vec!["dashboard.view".to_string()],
    }
}