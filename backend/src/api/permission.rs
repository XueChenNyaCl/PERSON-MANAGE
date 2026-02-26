use axum::{extract::{State, Path, Query}, http::StatusCode, Json, Extension};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::api::routes::AppState;
use crate::core::auth::Claims;
use crate::core::error::AppError;
use crate::core::permission::{PermissionManager, PermissionResult};

/// 权限列表响应
#[derive(Debug, Serialize)]
pub struct PermissionListResponse {
    pub role: String,
    pub permissions: Vec<PermissionItem>,
}

#[derive(Debug, Serialize)]
pub struct PermissionItem {
    pub permission: String,
    pub priority: i32,
    pub created_at: String,
}

/// 添加权限请求
#[derive(Debug, Deserialize)]
pub struct AddPermissionRequest {
    pub role: String,
    pub permission: String,
    pub priority: Option<i32>,
}

/// 移除权限请求
#[derive(Debug, Deserialize)]
pub struct RemovePermissionRequest {
    pub role: String,
    pub permission: String,
}

/// 用户权限列表响应
#[derive(Debug, Serialize)]
pub struct UserPermissionListResponse {
    pub user_id: Uuid,
    pub permissions: Vec<UserPermissionItem>,
}

#[derive(Debug, Serialize)]
pub struct UserPermissionItem {
    pub permission: String,
    pub value: bool,
    pub priority: i32,
    pub created_at: String,
}

/// 添加用户权限请求
#[derive(Debug, Deserialize)]
pub struct AddUserPermissionRequest {
    pub permission: String,
    pub value: bool,
    pub priority: Option<i32>,
}

/// 检查权限请求
#[derive(Debug, Deserialize)]
pub struct CheckPermissionRequest {
    pub permission: String,
}

/// 检查权限响应
#[derive(Debug, Serialize)]
pub struct CheckPermissionResponse {
    pub has_permission: bool,
    pub result: String,
}

/// 权限翻译请求
#[derive(Debug, Deserialize)]
pub struct PermissionTranslationRequest {
    pub permissions: Vec<String>,
}

/// 权限翻译响应项
#[derive(Debug, Serialize)]
pub struct PermissionTranslationItem {
    pub permission_key: String,
    pub translation: String,
}

/// YAML应用请求
#[derive(Debug, Deserialize)]
pub struct YamlApplyRequest {
    pub yaml_content: String,
    pub target_type: String, // "user", "role", "all"
    pub target_ids: Option<Vec<Uuid>>,
    pub role: Option<String>,
    pub merge_strategy: String, // "overwrite", "merge"
}

/// YAML应用响应
#[derive(Debug, Serialize)]
pub struct YamlApplyResponse {
    pub success: bool,
    pub message: String,
    pub applied_count: i32,
}

/// 获取所有权限键
#[derive(Debug, Serialize)]
pub struct PermissionKeysResponse {
    pub keys: Vec<String>,
}

/// 获取所有角色权限
pub async fn list_role_permissions(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
) -> Result<Json<Vec<PermissionListResponse>>, AppError> {
    // 只有管理员可以查看所有权限
    let pool = state.pool.ok_or_else(|| AppError::Internal)?;
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| AppError::Auth("无效的用户ID".to_string()))?;
    
    let manager = PermissionManager::new(pool.clone());
    let has_admin_permission = manager.check_permission(user_id, "system.settings").await;
    
    match has_admin_permission {
        PermissionResult::Allowed => {
            // 获取所有角色权限
            let roles = vec!["admin", "teacher", "student", "parent"];
            let mut result = Vec::new();
            
            for role in roles {
                let permissions = manager.get_role_permissions(role).await
                    .unwrap_or_else(|_| Vec::new());
                
                let items: Vec<PermissionItem> = permissions.into_iter().map(|node| {
                    PermissionItem {
                        permission: node.permission,
                        priority: node.priority,
                        created_at: chrono::Utc::now().to_rfc3339(), // 注意：这里应该从数据库获取
                    }
                }).collect();
                
                result.push(PermissionListResponse {
                    role: role.to_string(),
                    permissions: items,
                });
            }
            
            Ok(Json(result))
        }
        _ => Err(AppError::Auth("没有权限查看权限列表".to_string())),
    }
}

/// 添加角色权限
pub async fn add_role_permission(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Json(payload): Json<AddPermissionRequest>,
) -> Result<StatusCode, AppError> {
    let pool = state.pool.ok_or_else(|| AppError::Internal)?;
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| AppError::Auth("无效的用户ID".to_string()))?;
    
    let manager = PermissionManager::new(pool.clone());
    let has_admin_permission = manager.check_permission(user_id, "system.settings").await;
    
    match has_admin_permission {
        PermissionResult::Allowed => {
            let priority = payload.priority.unwrap_or(0);
            manager.add_role_permission(&payload.role, &payload.permission, priority).await?;
            
            Ok(StatusCode::CREATED)
        }
        _ => Err(AppError::Auth("没有权限添加权限".to_string())),
    }
}

/// 移除角色权限
pub async fn remove_role_permission(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Json(payload): Json<RemovePermissionRequest>,
) -> Result<StatusCode, AppError> {
    let pool = state.pool.ok_or_else(|| AppError::Internal)?;
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| AppError::Auth("无效的用户ID".to_string()))?;
    
    let manager = PermissionManager::new(pool.clone());
    let has_admin_permission = manager.check_permission(user_id, "system.settings").await;
    
    match has_admin_permission {
        PermissionResult::Allowed => {
            manager.remove_role_permission(&payload.role, &payload.permission).await?;
            
            Ok(StatusCode::NO_CONTENT)
        }
        _ => Err(AppError::Auth("没有权限移除权限".to_string())),
    }
}

/// 获取用户特定权限
pub async fn list_user_permissions(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Path(user_id): Path<Uuid>,
) -> Result<Json<UserPermissionListResponse>, AppError> {
    let pool = state.pool.ok_or_else(|| AppError::Internal)?;
    let current_user_id = Uuid::parse_str(&claims.sub).map_err(|_| AppError::Auth("无效的用户ID".to_string()))?;
    
    let manager = PermissionManager::new(pool.clone());
    
    // 检查权限：用户只能查看自己的权限，或者管理员可以查看所有
    let is_admin = match manager.check_permission(current_user_id, "system.settings").await {
        PermissionResult::Allowed => true,
        _ => false,
    };
    
    if !is_admin && current_user_id != user_id {
        return Err(AppError::Auth("没有权限查看其他用户的权限".to_string()));
    }
    
    let permissions = manager.get_user_specific_permissions(user_id).await?;
    
    let items: Vec<UserPermissionItem> = permissions.into_iter().map(|node| {
        UserPermissionItem {
            permission: node.permission,
            value: node.value,
            priority: node.priority,
            created_at: chrono::Utc::now().to_rfc3339(), // 注意：这里应该从数据库获取
        }
    }).collect();
    
    Ok(Json(UserPermissionListResponse {
        user_id,
        permissions: items,
    }))
}

/// 添加用户特定权限
pub async fn add_user_permission(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Path(user_id): Path<Uuid>,
    Json(payload): Json<AddUserPermissionRequest>,
) -> Result<StatusCode, AppError> {
    let pool = state.pool.ok_or_else(|| AppError::Internal)?;
    let current_user_id = Uuid::parse_str(&claims.sub).map_err(|_| AppError::Auth("无效的用户ID".to_string()))?;
    
    let manager = PermissionManager::new(pool.clone());
    
    // 检查权限：用户只能管理自己的权限，或者管理员可以管理所有
    let is_admin = match manager.check_permission(current_user_id, "system.settings").await {
        PermissionResult::Allowed => true,
        _ => false,
    };
    
    if !is_admin && current_user_id != user_id {
        return Err(AppError::Auth("没有权限管理其他用户的权限".to_string()));
    }
    
    let priority = payload.priority.unwrap_or(100);
    manager.add_user_permission(user_id, &payload.permission, payload.value, priority).await?;
    
    Ok(StatusCode::CREATED)
}

/// 移除用户特定权限
pub async fn remove_user_permission(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Path(user_id): Path<Uuid>,
    Query(params): Query<std::collections::HashMap<String, String>>,
) -> Result<StatusCode, AppError> {
    let permission = params.get("permission")
        .ok_or_else(|| AppError::InvalidInput("缺少权限参数".to_string()))?;
    
    let pool = state.pool.ok_or_else(|| AppError::Internal)?;
    let current_user_id = Uuid::parse_str(&claims.sub).map_err(|_| AppError::Auth("无效的用户ID".to_string()))?;
    
    let manager = PermissionManager::new(pool.clone());
    
    // 检查权限：用户只能管理自己的权限，或者管理员可以管理所有
    let is_admin = match manager.check_permission(current_user_id, "system.settings").await {
        PermissionResult::Allowed => true,
        _ => false,
    };
    
    if !is_admin && current_user_id != user_id {
        return Err(AppError::Auth("没有权限管理其他用户的权限".to_string()));
    }
    
    manager.remove_user_permission(user_id, permission).await?;
    
    Ok(StatusCode::NO_CONTENT)
}

/// 检查当前用户权限
pub async fn check_permission(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Json(payload): Json<CheckPermissionRequest>,
) -> Result<Json<CheckPermissionResponse>, AppError> {
    let pool = state.pool.ok_or_else(|| AppError::Internal)?;
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| AppError::Auth("无效的用户ID".to_string()))?;
    
    let manager = PermissionManager::new(pool.clone());
    let result = manager.check_permission(user_id, &payload.permission).await;
    
    let (has_permission, result_str) = match result {
        PermissionResult::Allowed => (true, "allowed".to_string()),
        PermissionResult::Denied => (false, "denied".to_string()),
        PermissionResult::NotSet => (false, "not_set".to_string()),
    };
    
    Ok(Json(CheckPermissionResponse {
        has_permission,
        result: result_str,
    }))
}

/// 获取权限翻译
pub async fn get_permission_translations(
    State(_state): State<AppState>,
    Extension(_claims): Extension<Claims>,
    Json(payload): Json<PermissionTranslationRequest>,
) -> Result<Json<Vec<PermissionTranslationItem>>, AppError> {
    // 这里应该从翻译文件或数据库加载翻译
    // 目前使用硬编码的翻译映射
    let translation_map: std::collections::HashMap<&str, &str> = [
        ("system.settings", "系统设置"),
        ("system.permissions", "权限管理"),
        ("person.view", "人员查看"),
        ("person.manage", "人员管理"),
        ("person.sensitive.view", "查看敏感信息"),
        ("person.delete", "删除人员"),
        ("class.view", "班级查看"),
        ("class.manage", "班级管理"),
        ("class.update_teacher", "更新班主任"),
        ("department.view", "部门查看"),
        ("department.manage", "部门管理"),
        ("department.update", "更新部门"),
        ("attendance.view", "考勤查看"),
        ("attendance.manage", "考勤管理"),
        ("score.view", "成绩查看"),
        ("score.manage", "成绩管理"),
        ("notice.view", "通知查看"),
        ("notice.manage", "通知管理"),
        ("dashboard.view", "仪表板查看"),
        ("person.*", "所有人员权限"),
        ("class.*", "所有班级权限"),
        ("department.*", "所有部门权限"),
        ("attendance.*", "所有考勤权限"),
        ("score.*", "所有成绩权限"),
        ("notice.*", "所有通知权限"),
        ("*.view", "所有查看权限"),
    ].iter().cloned().collect();
    
    let mut result = Vec::new();
    for permission_key in &payload.permissions {
        let translation = translation_map.get(permission_key.as_str())
            .copied()
            .unwrap_or(permission_key.as_str())
            .to_string();
        result.push(PermissionTranslationItem {
            permission_key: permission_key.clone(),
            translation,
        });
    }
    
    Ok(Json(result))
}

/// 获取所有权限键
pub async fn get_all_permission_keys(
    State(_state): State<AppState>,
    Extension(_claims): Extension<Claims>,
) -> Result<Json<PermissionKeysResponse>, AppError> {
    // 返回所有已知的权限键
    let keys = vec![
        "system.settings".to_string(),
        "system.permissions".to_string(),
        "person.view".to_string(),
        "person.manage".to_string(),
        "person.sensitive.view".to_string(),
        "person.delete".to_string(),
        "class.view".to_string(),
        "class.manage".to_string(),
        "class.update_teacher".to_string(),
        "department.view".to_string(),
        "department.manage".to_string(),
        "department.update".to_string(),
        "attendance.view".to_string(),
        "attendance.manage".to_string(),
        "score.view".to_string(),
        "score.manage".to_string(),
        "notice.view".to_string(),
        "notice.manage".to_string(),
        "dashboard.view".to_string(),
        "person.*".to_string(),
        "class.*".to_string(),
        "department.*".to_string(),
        "attendance.*".to_string(),
        "score.*".to_string(),
        "notice.*".to_string(),
        "*.view".to_string(),
    ];
    
    Ok(Json(PermissionKeysResponse { keys }))
}

/// 应用YAML模板
pub async fn apply_yaml_template(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Json(payload): Json<YamlApplyRequest>,
) -> Result<Json<YamlApplyResponse>, AppError> {
    println!("=== YAML TEMPLATE DEBUG ===");
    println!("Received payload: {:?}", payload);
    println!("YAML content length: {}", payload.yaml_content.len());
    println!("YAML content preview: {}", 
        if payload.yaml_content.len() > 100 { 
            // 安全地切片UTF-8字符串，确保在字符边界处切片
            let mut end = 100;
            while !payload.yaml_content.is_char_boundary(end) && end > 0 {
                end -= 1;
            }
            if end > 0 {
                &payload.yaml_content[..end]
            } else {
                &payload.yaml_content
            }
        } else { 
            &payload.yaml_content 
        });
    
    let pool = state.pool.ok_or_else(|| AppError::Internal)?;
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| AppError::Auth("无效的用户ID".to_string()))?;
    
    let manager = PermissionManager::new(pool.clone());
    let has_admin_permission = manager.check_permission(user_id, "system.settings").await;
    
    match has_admin_permission {
        PermissionResult::Allowed => {
            // 解析YAML内容
            let template = match crate::core::permission::PermissionTemplate::from_yaml_str(&payload.yaml_content) {
                Ok(template) => template,
                Err(e) => return Err(AppError::InvalidInput(format!("YAML解析失败: {}", e))),
            };
            
            let mut applied_count = 0;
            
            match payload.target_type.as_str() {
                "user" => {
                    if let Some(target_ids) = payload.target_ids {
                        for target_id in target_ids {
                            if let Err(e) = template.apply_to_user(&pool, target_id).await {
                                // 记录错误但继续处理其他用户
                                eprintln!("应用模板到用户 {} 失败: {}", target_id, e);
                            } else {
                                applied_count += 1;
                            }
                        }
                    }
                },
                "role" => {
                    if let Some(role) = payload.role {
                        if let Err(e) = template.apply_to_role(&pool, &role).await {
                            // 记录错误
                            eprintln!("应用模板到角色 {} 失败: {}", role, e);
                        } else {
                            applied_count += 1;
                        }
                    }
                },
                "all" => {
                    // 应用模板到所有用户
                    // 获取所有用户ID
                    use sqlx::Row;
                    let user_rows = sqlx::query("SELECT id FROM persons")
                        .fetch_all(&pool)
                        .await
                        .map_err(|e| AppError::InvalidInput(format!("获取用户列表失败: {}", e)))?;
                    
                    for row in user_rows {
                        let user_id = row.get::<Uuid, _>("id");
                        if let Err(e) = template.apply_to_user(&pool, user_id).await {
                            // 记录错误但继续处理其他用户
                            eprintln!("应用模板到用户 {} 失败: {}", user_id, e);
                        } else {
                            applied_count += 1;
                        }
                    }
                },
                _ => return Err(AppError::InvalidInput("无效的目标类型".to_string())),
            }
            
            Ok(Json(YamlApplyResponse {
                success: applied_count > 0,
                message: format!("成功应用到 {} 个目标", applied_count),
                applied_count,
            }))
        }
        _ => Err(AppError::Auth("没有权限应用YAML模板".to_string())),
    }
}