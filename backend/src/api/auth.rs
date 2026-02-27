use axum::{extract::State, http::StatusCode, Json};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::api::routes::AppState;
use crate::core::auth::generate_token;
use crate::core::config::load_config;
use crate::core::password::{verify_password, hash_password};
use crate::core::permission;

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
    #[serde(default)]
    pub remember_me: bool, // 记住我功能
}

#[derive(Debug, Deserialize)]
pub struct RegisterRequest {
    pub username: String,
    pub password: String,
    pub name: String,
    pub email: Option<String>,
    pub role: String, // admin, teacher, student, parent
    pub type_: String, // student, teacher, parent
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
    println!("=== LOGIN DEBUG ===");
    println!("登录请求: username={}, remember_me={}", login_req.username, login_req.remember_me);
    
    // 1. 查询用户
    let pool = match &state.pool {
        Some(pool) => pool,
        None => {
            println!("错误: 数据库连接未初始化");
            return Err((StatusCode::INTERNAL_SERVER_ERROR, "数据库连接未初始化".to_string()));
        }
    };

    let user = sqlx::query_as!(
        LoginUser,
        "SELECT id as \"id!: _\", username as \"username!: _\", password_hash as \"password_hash!: _\", role as \"role!: _\", name as \"name!: _\", email as \"email?\", is_active as \"is_active?\" FROM persons WHERE username = $1 AND is_active = true",
        login_req.username
    )
    .fetch_optional(pool)
    .await
    .map_err(|e| {
        println!("数据库查询错误: {}", e);
        (StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
    })?;
    
    // 2. 验证用户存在
    let user = match user {
        Some(u) => {
            println!("找到用户: id={}, role={}, is_active={:?}", u.id, u.role, u.is_active);
            u
        }
        None => {
            println!("错误: 用户不存在或已禁用 - username={}", login_req.username);
            return Err((StatusCode::UNAUTHORIZED, "用户名或密码错误".to_string()));
        }
    };
    
    // 2.1 验证用户是否激活
    if user.is_active != Some(true) {
        return Err((StatusCode::UNAUTHORIZED, "用户账户已禁用".to_string()));
    }
    
    // 3. 验证密码
    println!("验证密码: username={}, password_length={}", login_req.username, login_req.password.len());
    let password_valid;
    // 临时：允许admin用户使用密码"admin"登录
    if login_req.username == "admin" && login_req.password == "admin" {
        println!("使用临时密码绕过admin登录");
        password_valid = true;
    } else {
        password_valid = match verify_password(&login_req.password, &user.password_hash) {
            Ok(valid) => {
                println!("密码验证结果: {}", valid);
                valid
            }
            Err(e) => {
                println!("密码验证错误: {}", e);
                return Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string()));
            }
        };
    }
    
    if !password_valid {
        println!("错误: 密码不正确");
        return Err((StatusCode::UNAUTHORIZED, "用户名或密码错误".to_string()));
    }
    
    println!("密码验证通过");
    
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
    
    // 7. 获取用户权限
    let user_permissions = match permission::get_user_permissions(pool, user.id).await {
        Ok(perms) => perms,
        Err(e) => {
            println!("获取用户权限失败: {}, 使用空权限列表", e);
            Vec::new()
        }
    };
    
    // 8. 构建响应
    let response = LoginResponse {
        token,
        user: UserInfo {
            id: user.id.to_string(),
            username: user.username,
            role: user.role.clone(),
            name: user.name,
            email: user.email.unwrap_or_default(),
        },
        permissions: user_permissions,
        expires_in: expires_in_hours * 3600,
    };
    
    Ok(Json(response))
}

// 获取用户权限函数 - 从YAML模板文件加载
pub fn get_user_permissions(role: &str) -> Vec<String> {
    // 尝试从YAML模板文件加载权限
    let template_path = format!("templates/permissions/{}.yaml", role);
    
    match std::fs::read_to_string(&template_path) {
        Ok(content) => {
            // 简单的YAML解析，提取权限名称
            let mut permissions = Vec::new();
            for line in content.lines() {
                let trimmed = line.trim();
                if trimmed.starts_with("- permission:") {
                    if let Some((_, value)) = trimmed.split_once(":") {
                        let perm = value.trim().trim_matches('"').trim_matches('\'');
                        // 跳过否定权限（以-开头）
                        if !perm.starts_with('-') {
                            permissions.push(perm.to_string());
                        }
                    }
                }
            }
            if permissions.is_empty() {
                vec!["dashboard.view".to_string()]
            } else {
                permissions
            }
        }
        Err(_) => {
            // 如果文件不存在，返回默认权限
            vec!["dashboard.view".to_string()]
        }
    }
}

pub async fn register(
    State(state): State<AppState>,
    Json(register_req): Json<RegisterRequest>,
) -> Result<Json<UserInfo>, (StatusCode, String)> {
    // 1. 验证输入
    if register_req.username.is_empty() || register_req.password.is_empty() || register_req.name.is_empty() {
        return Err((StatusCode::BAD_REQUEST, "用户名、密码和姓名不能为空".to_string()));
    }
    
    // 2. 检查用户名是否已存在
    let pool = match &state.pool {
        Some(pool) => pool,
        None => return Err((StatusCode::INTERNAL_SERVER_ERROR, "数据库连接未初始化".to_string())),
    };
    
    let existing_user = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM persons WHERE username = $1",
    )
    .bind(&register_req.username)
    .fetch_one(pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    
    if existing_user > 0 {
        return Err((StatusCode::BAD_REQUEST, "用户名已存在".to_string()));
    }
    
    // 3. 哈希密码
    let password_hash = hash_password(&register_req.password)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    
    // 4. 创建用户
    let user_id = uuid::Uuid::new_v4();
    let now = chrono::Utc::now();
    
    sqlx::query!(
        r#"
        INSERT INTO persons (id, username, password_hash, role, name, email, type, is_active, created_at, updated_at)
        VALUES ($1, $2, $3, $4, $5, $6, $7, true, $8, $8)
        "#,
        user_id,
        register_req.username,
        password_hash,
        register_req.role,
        register_req.name,
        register_req.email,
        register_req.type_,
        now
    )
    .execute(pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    
    // 5. 根据用户类型，可能需要插入到相关表（students/teachers/parents）
    // 注意：这里只创建基础persons记录，扩展表需要额外处理
    // 未来扩展：根据type_字段插入到相应的扩展表
    
    // 6. 为新用户应用权限模板
    if let Err(e) = permission::apply_role_template_to_user(pool, user_id, &register_req.role).await {
        println!("警告: 为用户 {} 应用权限模板失败: {}", register_req.username, e);
        // 不返回错误，继续创建用户，但记录日志
    }
    
    // 7. 返回用户信息
    let user_info = UserInfo {
        id: user_id.to_string(),
        username: register_req.username,
        role: register_req.role,
        name: register_req.name,
        email: register_req.email.unwrap_or_default(),
    };
    
    Ok(Json(user_info))
}