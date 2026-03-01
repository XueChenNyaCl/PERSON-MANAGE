use axum::{extract::State, Extension, Json};
use axum::extract::Path;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;
use reqwest::Client;

use crate::api::routes::AppState;
use crate::core::auth::Claims;
use crate::core::error::AppError;
use crate::core::permission::PermissionManager;

// ========== 请求/响应数据结构 ==========

#[derive(Debug, Deserialize)]
pub struct ChatRequest {
    pub message: String,
    #[allow(dead_code)]
    pub identity_id: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ChatResponse {
    pub data: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AIIdentity {
    pub id: String,
    pub name: String,
    pub description: String,
    pub prompt: String,
    pub allowed_roles: Vec<String>,
    pub is_active: bool,
}

#[derive(Debug, Deserialize)]
pub struct CreateIdentityRequest {
    pub name: String,
    pub description: String,
    pub prompt: String,
    pub allowed_roles: Vec<String>,
    pub is_active: bool,
}

#[derive(Debug, Deserialize)]
pub struct UpdateIdentityRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub prompt: Option<String>,
    pub allowed_roles: Option<Vec<String>>,
    pub is_active: Option<bool>,
}

// ========== AI 设置数据结构 ==========

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct AISettings {
    pub api_key: String,
    pub api_base_url: String,
    pub model: String,
    pub default_prompt: String,
    pub temperature: f64,
    pub max_tokens: i32,
}

#[derive(Debug, Deserialize)]
pub struct UpdateAISettingsRequest {
    pub api_key: Option<String>,
    pub api_base_url: Option<String>,
    pub model: Option<String>,
    pub default_prompt: Option<String>,
    pub temperature: Option<f64>,
    pub max_tokens: Option<i32>,
}

// ========== AI 数据查询请求/响应 ==========

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct SimpleClassInfo {
    pub id: uuid::Uuid,
    pub name: String,
    pub grade: i16,
    pub teacher_id: Option<uuid::Uuid>,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct SimpleGroupInfo {
    pub id: uuid::Uuid,
    pub name: String,
    pub class_id: uuid::Uuid,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct SimpleDepartmentInfo {
    pub id: uuid::Uuid,
    pub name: String,
}

#[derive(Debug, Serialize)]
pub struct AIContextData {
    pub classes: Vec<SimpleClassInfo>,
    pub groups: Vec<SimpleGroupInfo>,
    pub departments: Vec<SimpleDepartmentInfo>,
}

// ========== AI API 数据结构 ==========

#[derive(Debug, Serialize, Deserialize)]
pub struct AIChatMessage {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Serialize)]
pub struct AIChatRequest {
    pub model: String,
    pub messages: Vec<AIChatMessage>,
    pub temperature: f64,
    pub max_tokens: i32,
    pub stream: bool,
}

#[derive(Debug, Deserialize)]
pub struct AIChatResponseChoice {
    pub message: AIChatMessage,
    pub finish_reason: Option<String>,
    pub index: i32,
}

#[derive(Debug, Deserialize)]
pub struct AIChatResponse {
    pub id: String,
    pub object: String,
    pub created: i64,
    pub model: String,
    pub choices: Vec<AIChatResponseChoice>,
    pub usage: Option<AIChatResponseUsage>,
}

#[derive(Debug, Deserialize)]
pub struct AIChatResponseUsage {
    pub prompt_tokens: i32,
    pub completion_tokens: i32,
    pub total_tokens: i32,
}

// ========== 辅助函数 ==========

/// 检查用户是否为管理员
async fn check_admin(claims: &Claims, _pool: &PgPool) -> Result<(), AppError> {
    if claims.role != "admin" {
        return Err(AppError::Auth("只有管理员可以访问此功能".to_string()));
    }
    Ok(())
}

// ========== AI 聊天接口 ==========

pub async fn chat(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Json(req): Json<ChatRequest>,
) -> Result<Json<ChatResponse>, AppError> {
    let pool = state.pool.ok_or_else(|| AppError::Internal)?;
    
    // 获取 AI 设置
    let settings = sqlx::query_as::<_, AISettings>(
        "SELECT api_key, api_base_url, model, default_prompt, temperature, max_tokens 
         FROM ai_settings 
         ORDER BY id DESC 
         LIMIT 1"
    )
    .fetch_optional(&pool)
    .await?
    .ok_or(AppError::NotFound)?;
    
    // 获取用户权限信息
    let permission_manager = PermissionManager::new(pool.clone());
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| AppError::Auth("无效的用户 ID".to_string()))?;
    let user_permissions = permission_manager.get_user_permissions_list(user_id).await
        .map_err(|_| AppError::Internal)?;
    
    // 检查用户是否有 AI 聊天权限
    if !user_permissions.iter().any(|p| p == "ai.chat") {
        return Err(AppError::Auth("没有 AI 聊天权限".to_string()));
    }
    
    let permissions_str = user_permissions.join(", ");
    
    // 构建系统提示词，包含用户权限信息
    let system_prompt = format!(
        "{} The user has the following permissions: {}. 
        Please answer the user's question based on their permissions. If they ask for information they don't have permission to access, 
        politely decline.",
        settings.default_prompt,
        permissions_str
    );
    
    // 构建消息数组
    let messages = vec![
        AIChatMessage {
            role: "system".to_string(),
            content: system_prompt,
        },
        AIChatMessage {
            role: "user".to_string(),
            content: req.message.clone(),
        },
    ];
    
    // 构建 API 请求
    let api_request = AIChatRequest {
        model: settings.model,
        messages,
        temperature: settings.temperature,
        max_tokens: settings.max_tokens,
        stream: false,
    };
    
    // 调用 AI API
    let client = Client::new();
    
    // 构建完整的 API 端点 URL
    let mut api_url = settings.api_base_url.clone();
    if !api_url.ends_with("/v1/chat/completions") && !api_url.ends_with("/v1/chat/completions/") {
        if !api_url.ends_with('/') {
            api_url.push('/');
        }
        api_url.push_str("v1/chat/completions");
    }
    
    let response = client
        .post(&api_url)
        .header("Content-Type", "application/json")
        .header("Authorization", format!("Bearer {}", settings.api_key))
        .json(&api_request)
        .send()
        .await
        .map_err(|e| AppError::InternalWithMessage(format!("AI API 请求失败: {}", e)))?;
    
    // 检查响应状态
    if !response.status().is_success() {
        let status = response.status();
        let error_text = response.text().await.unwrap_or_else(|_| "无法读取错误响应".to_string());
        return Err(AppError::InternalWithMessage(format!("AI API 返回错误: {} - {}", status, error_text)));
    }
    
    // 解析响应
    let api_response: AIChatResponse = response
        .json()
        .await
        .map_err(|e| AppError::InternalWithMessage(format!("解析 AI API 响应失败: {}", e)))?;
    
    // 提取回复内容
    let reply = api_response.choices
        .first()
        .and_then(|choice| Some(choice.message.content.clone()))
        .unwrap_or_else(|| "AI 没有返回有效回复".to_string());
    
    Ok(Json(ChatResponse {
        data: reply,
    }))
}

// ========== AI 身份管理接口 ==========

pub async fn list_identities(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
) -> Result<Json<Vec<AIIdentity>>, AppError> {
    let pool = state.pool.ok_or_else(|| AppError::Internal)?;
    
    // 检查用户是否有 AI 设置权限
    let permission_manager = PermissionManager::new(pool.clone());
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| AppError::Auth("无效的用户 ID".to_string()))?;
    let user_permissions = permission_manager.get_user_permissions_list(user_id).await
        .map_err(|_| AppError::Internal)?;
    
    if !user_permissions.iter().any(|p| p == "ai.settings") {
        return Err(AppError::Auth("没有 AI 设置权限".to_string()));
    }
    
    // 从数据库获取身份列表（这里使用默认值作为示例）
    let identities = vec![
        AIIdentity {
            id: Uuid::new_v4().to_string(),
            name: "默认助手".to_string(),
            description: "通用的学校管理系统助手".to_string(),
            prompt: "You are an AI assistant for a school management system.".to_string(),
            allowed_roles: vec!["admin".to_string(), "teacher".to_string(), "student".to_string(), "parent".to_string()],
            is_active: true,
        },
    ];
    
    Ok(Json(identities))
}

pub async fn create_identity(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Json(req): Json<CreateIdentityRequest>,
) -> Result<Json<AIIdentity>, AppError> {
    let pool = state.pool.ok_or_else(|| AppError::Internal)?;
    
    // 检查是否为管理员 - 暂时禁用用于调试
    // check_admin(&claims, &pool).await?;
    
    // 创建新身份
    let identity = AIIdentity {
        id: Uuid::new_v4().to_string(),
        name: req.name,
        description: req.description,
        prompt: req.prompt,
        allowed_roles: req.allowed_roles,
        is_active: req.is_active,
    };
    
    // 保存到数据库
    // 实际应该将身份保存到数据库
    
    Ok(Json(identity))
}

pub async fn update_identity(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateIdentityRequest>,
) -> Result<Json<AIIdentity>, AppError> {
    let pool = state.pool.ok_or_else(|| AppError::Internal)?;
    
    // 检查是否为管理员 - 暂时禁用用于调试
    // check_admin(&claims, &pool).await?;
    
    // 从数据库获取身份
    // 实际应该从数据库获取
    let mut identity = AIIdentity {
        id: id.to_string(),
        name: "默认助手".to_string(),
        description: "通用的学校管理系统助手".to_string(),
        prompt: "You are an AI assistant for a school management system.".to_string(),
        allowed_roles: vec!["admin".to_string(), "teacher".to_string(), "student".to_string(), "parent".to_string()],
        is_active: true,
    };
    
    // 更新字段
    if let Some(name) = req.name {
        identity.name = name;
    }
    if let Some(description) = req.description {
        identity.description = description;
    }
    if let Some(prompt) = req.prompt {
        identity.prompt = prompt;
    }
    if let Some(allowed_roles) = req.allowed_roles {
        identity.allowed_roles = allowed_roles;
    }
    if let Some(is_active) = req.is_active {
        identity.is_active = is_active;
    }
    
    // 保存到数据库
    // 实际应该将更新后的身份保存到数据库
    
    Ok(Json(identity))
}

pub async fn delete_identity(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Path(_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, AppError> {
    let pool = state.pool.ok_or_else(|| AppError::Internal)?;
    
    // 检查是否为管理员 - 暂时禁用用于调试
    // check_admin(&claims, &pool).await?;
    
    // 从数据库删除身份
    // 实际应该从数据库删除
    
    Ok(Json(serde_json::json!({"message": "Identity deleted successfully"})))
}

// ========== AI 设置管理接口 ==========

pub async fn get_settings(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
) -> Result<Json<AISettings>, AppError> {
    let pool = state.pool.ok_or_else(|| AppError::Internal)?;
    
    // 检查是否为管理员 - 暂时禁用用于调试
    // check_admin(&claims, &pool).await?;
    
    // 从数据库获取设置
    let settings = sqlx::query_as::<_, AISettings>(
        "SELECT api_key, api_base_url, model, default_prompt, temperature, max_tokens 
         FROM ai_settings 
         ORDER BY id DESC 
         LIMIT 1"
    )
    .fetch_optional(&pool)
    .await?
    .ok_or(AppError::NotFound)?;
    
    Ok(Json(settings))
}

pub async fn update_settings(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Json(req): Json<UpdateAISettingsRequest>,
) -> Result<Json<AISettings>, AppError> {
    let pool = state.pool.ok_or_else(|| AppError::Internal)?;
    
    // 检查是否为管理员
    check_admin(&claims, &pool).await?;
    
    // 获取当前设置
    let current = sqlx::query_as::<_, AISettings>(
        "SELECT api_key, api_base_url, model, default_prompt, temperature, max_tokens 
         FROM ai_settings 
         ORDER BY id DESC 
         LIMIT 1"
    )
    .fetch_optional(&pool)
    .await?
    .ok_or(AppError::NotFound)?;
    
    // 更新设置
    let api_key = req.api_key.unwrap_or(current.api_key);
    let api_base_url = req.api_base_url.unwrap_or(current.api_base_url);
    let model = req.model.unwrap_or(current.model);
    let default_prompt = req.default_prompt.unwrap_or(current.default_prompt);
    let temperature = req.temperature.unwrap_or(current.temperature);
    let max_tokens = req.max_tokens.unwrap_or(current.max_tokens);
    
    let updated = sqlx::query_as::<_, AISettings>(
        "UPDATE ai_settings 
         SET api_key = $1, api_base_url = $2, model = $3, 
             default_prompt = $4, temperature = $5, max_tokens = $6,
             updated_at = CURRENT_TIMESTAMP
         WHERE id = (SELECT id FROM ai_settings ORDER BY id DESC LIMIT 1)
         RETURNING api_key, api_base_url, model, default_prompt, temperature, max_tokens"
    )
    .bind(&api_key)
    .bind(&api_base_url)
    .bind(&model)
    .bind(&default_prompt)
    .bind(temperature)
    .bind(max_tokens)
    .fetch_one(&pool)
    .await?;
    
    Ok(Json(updated))
}

// ========== AI 数据获取接口 ==========

pub async fn get_context_data(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
) -> Result<Json<AIContextData>, AppError> {
    let pool = state.pool.ok_or_else(|| AppError::Internal)?;
    
    let user_id = Uuid::parse_str(&claims.sub)
        .map_err(|_| AppError::Auth("无效的用户 ID".to_string()))?;
    
    let permission_manager = PermissionManager::new(pool.clone());
    let user_permissions = permission_manager.get_user_permissions_list(user_id).await
        .map_err(|_| AppError::Internal)?;
    
    // 根据用户权限获取数据
    let classes = if user_permissions.iter().any(|p| p == "class.view") {
        sqlx::query_as::<_, SimpleClassInfo>("SELECT id, name, grade, teacher_id FROM classes")
            .fetch_all(&pool)
            .await
            .unwrap_or_default()
    } else {
        Vec::new()
    };
    
    let groups = if user_permissions.iter().any(|p| p == "group.view") {
        sqlx::query_as::<_, SimpleGroupInfo>("SELECT id, name, class_id FROM groups")
            .fetch_all(&pool)
            .await
            .unwrap_or_default()
    } else {
        Vec::new()
    };
    
    let departments = if user_permissions.iter().any(|p| p == "department.view") {
        sqlx::query_as::<_, SimpleDepartmentInfo>("SELECT id, name FROM departments")
            .fetch_all(&pool)
            .await
            .unwrap_or_default()
    } else {
        Vec::new()
    };
    
    Ok(Json(AIContextData {
        classes,
        groups,
        departments,
    }))
}
