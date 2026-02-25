use axum::{extract::State, http::StatusCode, Json};
use serde::{Deserialize, Serialize};

use crate::api::routes::AppState;
use crate::core::auth::generate_token;
use crate::core::config::load_config;

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Serialize)]
pub struct LoginResponse {
    pub token: String,
    pub user: UserInfo,
}

#[derive(Debug, Serialize)]
pub struct UserInfo {
    pub id: String,
    pub username: String,
    pub role: String,
}

pub async fn login(
    State(_state): State<AppState>,
    Json(login_req): Json<LoginRequest>,
) -> Result<Json<LoginResponse>, (StatusCode, String)> {
    // 模拟登录验证
    if login_req.username != "admin" || login_req.password != "admin123" {
        return Err((
            StatusCode::UNAUTHORIZED,
            "Invalid username or password".to_string(),
        ));
    }

    // 加载配置
    let config = load_config().map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    // 生成JWT token
    let token = generate_token("1", "admin", &config.jwt_secret, &config.jwt_expires_in)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    // 构建响应
    let response = LoginResponse {
        token,
        user: UserInfo {
            id: "1".to_string(),
            username: login_req.username,
            role: "admin".to_string(),
        },
    };

    Ok(Json(response))
}
