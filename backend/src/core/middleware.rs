use axum::{http::{StatusCode, request::Parts}, extract::State, Extension, RequestPartsExt};use jsonwebtoken::{decode, DecodingKey, Validation};use serde::{Deserialize, Serialize};use tower_http::auth::AsyncRequireAuthorizationLayer;

use crate::core::error::AppError;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    pub sub: String,
    pub role: String,
    pub exp: u64,
}

#[derive(Debug, Clone)]
pub struct AuthState {
    pub secret: String,
}

pub async fn auth_middleware<B>(
    auth: String,
    State(auth_state): State<AuthState>,
    mut req: Parts,
) -> Result<Parts, (StatusCode, String)> {
    let token = auth.replace("Bearer ", "");
    
    match decode::<Claims>(
        &token,
        &DecodingKey::from_secret(auth_state.secret.as_ref()),
        &Validation::default(),
    ) {
        Ok(decoded) => {
            req.extensions.insert(decoded.claims);
            Ok(req)
        }
        Err(e) => Err((StatusCode::UNAUTHORIZED, e.to_string())),
    }
}

pub fn require_auth() -> AsyncRequireAuthorizationLayer<impl Fn(&str) -> Result<String, std::convert::Infallible> + Clone> {
    AsyncRequireAuthorizationLayer::new(|auth: &str| {
        // 这里只是一个简单的验证，实际应该使用上面的auth_middleware
        Ok(auth.to_string())
    })
}
