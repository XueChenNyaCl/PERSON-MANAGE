use axum::{extract::Request, middleware::Next, response::Response, http::StatusCode};
use axum_extra::{TypedHeader, headers::{authorization::Bearer, Authorization}};

use crate::core::auth::{verify_token, Claims};
use crate::core::config::load_config;

pub async fn auth_middleware(
    TypedHeader(auth): TypedHeader<Authorization<Bearer>>,
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let token = auth.token();
    
    // 加载配置获取JWT密钥
    let config = load_config().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    // 验证令牌
    let claims = verify_token(token, &config.jwt_secret)
        .map_err(|_| StatusCode::UNAUTHORIZED)?;
    
    // 将用户信息添加到请求扩展中
    let mut request = request;
    request.extensions_mut().insert(claims);
    
    Ok(next.run(request).await)
}

// 旧的require_auth函数，保留兼容性
use tower_http::auth::AsyncRequireAuthorizationLayer;

pub fn require_auth() -> AsyncRequireAuthorizationLayer<impl Fn(&str) -> Result<String, std::convert::Infallible> + Clone> {
    AsyncRequireAuthorizationLayer::new(|auth: &str| {
        // 这里只是一个简单的验证，实际应该使用上面的auth_middleware
        Ok(auth.to_string())
    })
}