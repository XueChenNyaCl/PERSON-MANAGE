use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Extension, Json,
};
use serde::Deserialize;
use sqlx::PgPool;
use uuid::Uuid;

use crate::api::routes::AppState;
use crate::core::auth::{generate_special_user_token, Claims};
use crate::core::config::load_config;
use crate::core::error::AppError;
use crate::core::operation_logger::get_global_logger;
use crate::core::password::hash_password;
use crate::core::permission::{PermissionManager, PermissionResult};
use crate::models::special_user::{
    CreateSpecialUserRequest, LinkPersonRequest, SpecialUser,
    SpecialUserLoginRequest, SpecialUserLoginResponse, SpecialUserResponse, UpdateSpecialUserRequest,
};

/// 特殊用户列表查询参数
#[derive(Debug, Deserialize)]
pub struct ListSpecialUsersQuery {
    pub user_type: Option<String>,
    pub is_active: Option<bool>,
}

/// 检查用户是否有管理特殊用户的权限
async fn check_special_user_permission(pool: &PgPool, user_id: Uuid) -> Result<(), AppError> {
    let manager = PermissionManager::new(pool.clone());
    let result = manager.check_permission(user_id, "special_user.view").await;

    match result {
        PermissionResult::Allowed => Ok(()),
        _ => Err(AppError::Auth("没有权限管理特殊用户".to_string())),
    }
}

/// 获取特殊用户列表
pub async fn list_special_users(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Query(query): Query<ListSpecialUsersQuery>,
) -> Result<Json<Vec<SpecialUserResponse>>, AppError> {
    let pool = state.pool.ok_or_else(|| AppError::Internal)?;
    let user_id =
        Uuid::parse_str(&claims.sub).map_err(|_| AppError::Auth("无效的用户ID".to_string()))?;

    // 检查权限
    check_special_user_permission(&pool, user_id).await?;

    // 构建查询
    let mut sql = String::from(
        "SELECT su.*, p.name as linked_person_name 
         FROM special_users su 
         LEFT JOIN persons p ON su.linked_person_id = p.id 
         WHERE 1=1",
    );

    if let Some(user_type) = &query.user_type {
        sql.push_str(&format!(" AND su.user_type = '{}'", user_type));
    }

    if let Some(is_active) = query.is_active {
        sql.push_str(&format!(" AND su.is_active = {}", is_active));
    }

    sql.push_str(" ORDER BY su.created_at DESC");

    // 执行查询
    let rows = sqlx::query_as::<_, SpecialUserWithPersonName>(&sql)
        .fetch_all(&pool)
        .await
        .map_err(|e| AppError::InternalWithMessage(e.to_string()))?;

    let responses: Vec<SpecialUserResponse> = rows
        .into_iter()
        .map(|row| SpecialUserResponse {
            id: row.id,
            user_type: row.user_type,
            identifier: row.identifier,
            linked_person_id: row.linked_person_id,
            linked_person_name: row.linked_person_name,
            description: row.description,
            is_active: row.is_active,
            last_login_at: row.last_login_at,
            created_at: row.created_at,
        })
        .collect();

    Ok(Json(responses))
}

/// 辅助结构体用于查询
#[derive(Debug, sqlx::FromRow)]
struct SpecialUserWithPersonName {
    pub id: Uuid,
    pub user_type: String,
    pub identifier: String,
    pub linked_person_id: Option<Uuid>,
    pub linked_person_name: Option<String>,
    pub description: Option<String>,
    pub is_active: bool,
    pub last_login_at: Option<chrono::DateTime<chrono::Utc>>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// 创建特殊用户
pub async fn create_special_user(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Json(payload): Json<CreateSpecialUserRequest>,
) -> Result<Json<SpecialUserResponse>, AppError> {
    let pool = state.pool.ok_or_else(|| AppError::Internal)?;
    let user_id =
        Uuid::parse_str(&claims.sub).map_err(|_| AppError::Auth("无效的用户ID".to_string()))?;

    // 检查权限
    let manager = PermissionManager::new(pool.clone());
    let result = manager.check_permission(user_id, "special_user.create").await;
    match result {
        PermissionResult::Allowed => (),
        _ => return Err(AppError::Auth("没有权限创建特殊用户".to_string())),
    }

    // 验证用户类型
    let valid_types = ["iot", "scerm"];
    if !valid_types.contains(&payload.user_type.as_str()) {
        return Err(AppError::InvalidInput(format!(
            "无效的用户类型: {}，允许的类型: iot, scerm",
            payload.user_type
        )));
    }

    // 检查标识符是否已存在
    let existing = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM special_users WHERE user_type = $1 AND identifier = $2",
    )
    .bind(&payload.user_type)
    .bind(&payload.identifier)
    .fetch_one(&pool)
    .await
    .map_err(|e| AppError::InternalWithMessage(e.to_string()))?;

    if existing > 0 {
        return Err(AppError::InvalidInput(
            "该标识符已存在".to_string(),
        ));
    }

    // 哈希 API 密钥（如果提供）
    let api_key_hash = if let Some(api_key) = payload.api_key {
        Some(
            hash_password(&api_key)
                .map_err(|e| AppError::InternalWithMessage(e.to_string()))?,
        )
    } else {
        None
    };

    // 创建特殊用户
    let special_user = sqlx::query_as::<_, SpecialUser>(
        "INSERT INTO special_users (user_type, identifier, api_key_hash, description) 
         VALUES ($1, $2, $3, $4) 
         RETURNING *",
    )
    .bind(&payload.user_type)
    .bind(&payload.identifier)
    .bind(api_key_hash)
    .bind(&payload.description)
    .fetch_one(&pool)
    .await
    .map_err(|e| AppError::InternalWithMessage(e.to_string()))?;

    // 记录操作日志
    let user_id = Uuid::parse_str(&claims.sub).unwrap_or_else(|_| Uuid::nil());
    get_global_logger()
        .log_admin(
            user_id,
            &claims.username,
            "create special user",
            format!("created {} user: {}", payload.user_type, payload.identifier),
        )
        .await;

    Ok(Json(SpecialUserResponse {
        id: special_user.id,
        user_type: special_user.user_type,
        identifier: special_user.identifier,
        linked_person_id: special_user.linked_person_id,
        linked_person_name: None,
        description: special_user.description,
        is_active: special_user.is_active,
        last_login_at: special_user.last_login_at,
        created_at: special_user.created_at,
    }))
}

/// 删除特殊用户
pub async fn delete_special_user(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, AppError> {
    let pool = state.pool.ok_or_else(|| AppError::Internal)?;
    let user_id =
        Uuid::parse_str(&claims.sub).map_err(|_| AppError::Auth("无效的用户ID".to_string()))?;

    // 检查权限
    let manager = PermissionManager::new(pool.clone());
    let result = manager.check_permission(user_id, "special_user.delete").await;
    match result {
        PermissionResult::Allowed => (),
        _ => return Err(AppError::Auth("没有权限删除特殊用户".to_string())),
    }

    // 查询要删除的用户信息
    let special_user = sqlx::query_as::<_, SpecialUser>(
        "SELECT * FROM special_users WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(&pool)
    .await
    .map_err(|e| AppError::InternalWithMessage(e.to_string()))?;

    let special_user = match special_user {
        Some(u) => u,
        None => return Err(AppError::NotFound),
    };

    // 不允许删除 system, sysai, chatai 类型的用户
    let protected_types = ["system", "sysai", "chatai"];
    if protected_types.contains(&special_user.user_type.as_str()) {
        return Err(AppError::InvalidInput(
            "不能删除系统保留的特殊用户类型".to_string(),
        ));
    }

    // 删除特殊用户
    sqlx::query("DELETE FROM special_users WHERE id = $1")
        .bind(id)
        .execute(&pool)
        .await
        .map_err(|e| AppError::InternalWithMessage(e.to_string()))?;

    // 记录操作日志
    get_global_logger()
        .log_admin(
            user_id,
            &claims.username,
            "delete special user",
            format!(
                "deleted {} user: {}",
                special_user.user_type, special_user.identifier
            ),
        )
        .await;

    Ok(StatusCode::NO_CONTENT)
}

/// 关联人员到特殊用户
pub async fn link_person_to_special_user(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Path(id): Path<Uuid>,
    Json(payload): Json<LinkPersonRequest>,
) -> Result<Json<SpecialUserResponse>, AppError> {
    let pool = state.pool.ok_or_else(|| AppError::Internal)?;
    let user_id =
        Uuid::parse_str(&claims.sub).map_err(|_| AppError::Auth("无效的用户ID".to_string()))?;

    // 检查权限
    let manager = PermissionManager::new(pool.clone());
    let result = manager.check_permission(user_id, "special_user.link").await;
    match result {
        PermissionResult::Allowed => (),
        _ => return Err(AppError::Auth("没有权限关联特殊用户".to_string())),
    }

    // 验证人员是否存在
    let person_exists = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM persons WHERE id = $1")
        .bind(payload.person_id)
        .fetch_one(&pool)
        .await
        .map_err(|e| AppError::InternalWithMessage(e.to_string()))?;

    if person_exists == 0 {
        return Err(AppError::NotFound);
    }

    // 更新特殊用户的关联人员
    let special_user = sqlx::query_as::<_, SpecialUser>(
        "UPDATE special_users SET linked_person_id = $1 WHERE id = $2 RETURNING *",
    )
    .bind(payload.person_id)
    .bind(id)
    .fetch_one(&pool)
    .await
    .map_err(|e| AppError::InternalWithMessage(e.to_string()))?;

    // 获取关联人员名称
    let person_name: Option<String> = sqlx::query_scalar("SELECT name FROM persons WHERE id = $1")
        .bind(payload.person_id)
        .fetch_optional(&pool)
        .await
        .map_err(|e| AppError::InternalWithMessage(e.to_string()))?;

    // 记录操作日志
    get_global_logger()
        .log_admin(
            user_id,
            &claims.username,
            "link person to special user",
            format!(
                "linked person {} to {} user: {}",
                payload.person_id, special_user.user_type, special_user.identifier
            ),
        )
        .await;

    Ok(Json(SpecialUserResponse {
        id: special_user.id,
        user_type: special_user.user_type,
        identifier: special_user.identifier,
        linked_person_id: special_user.linked_person_id,
        linked_person_name: person_name,
        description: special_user.description,
        is_active: special_user.is_active,
        last_login_at: special_user.last_login_at,
        created_at: special_user.created_at,
    }))
}

/// 更新特殊用户
pub async fn update_special_user(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateSpecialUserRequest>,
) -> Result<Json<SpecialUserResponse>, AppError> {
    let pool = state.pool.ok_or_else(|| AppError::Internal)?;
    let user_id =
        Uuid::parse_str(&claims.sub).map_err(|_| AppError::Auth("无效的用户ID".to_string()))?;

    // 检查权限
    let manager = PermissionManager::new(pool.clone());
    let result = manager.check_permission(user_id, "special_user.create").await;
    match result {
        PermissionResult::Allowed => (),
        _ => return Err(AppError::Auth("没有权限更新特殊用户".to_string())),
    }

    // 构建更新 SQL
    let mut updates = Vec::new();
    
    if let Some(description) = payload.description {
        updates.push(format!("description = '{}'", description));
    }
    
    if let Some(is_active) = payload.is_active {
        updates.push(format!("is_active = {}", is_active));
    }

    if updates.is_empty() {
        return Err(AppError::InvalidInput("没有要更新的字段".to_string()));
    }

    let sql = format!(
        "UPDATE special_users SET {} WHERE id = $1 RETURNING *",
        updates.join(", ")
    );

    let special_user = sqlx::query_as::<_, SpecialUser>(&sql)
        .bind(id)
        .fetch_one(&pool)
        .await
        .map_err(|e| AppError::InternalWithMessage(e.to_string()))?;

    // 获取关联人员名称
    let person_name: Option<String> = if let Some(person_id) = special_user.linked_person_id {
        sqlx::query_scalar("SELECT name FROM persons WHERE id = $1")
            .bind(person_id)
            .fetch_optional(&pool)
            .await
            .map_err(|e| AppError::InternalWithMessage(e.to_string()))?
    } else {
        None
    };

    // 记录操作日志
    get_global_logger()
        .log_admin(
            user_id,
            &claims.username,
            "update special user",
            format!(
                "updated {} user: {}",
                special_user.user_type, special_user.identifier
            ),
        )
        .await;

    Ok(Json(SpecialUserResponse {
        id: special_user.id,
        user_type: special_user.user_type,
        identifier: special_user.identifier,
        linked_person_id: special_user.linked_person_id,
        linked_person_name: person_name,
        description: special_user.description,
        is_active: special_user.is_active,
        last_login_at: special_user.last_login_at,
        created_at: special_user.created_at,
    }))
}

/// 特殊用户登录（IoT/Scerm）
pub async fn special_user_login(
    State(state): State<AppState>,
    Json(payload): Json<SpecialUserLoginRequest>,
) -> Result<Json<SpecialUserLoginResponse>, AppError> {
    let pool = state.pool.ok_or_else(|| AppError::Internal)?;

    // 查询特殊用户
    let special_user = sqlx::query_as::<_, SpecialUser>(
        "SELECT * FROM special_users WHERE identifier = $1 AND is_active = true",
    )
    .bind(&payload.identifier)
    .fetch_optional(&pool)
    .await
    .map_err(|e| AppError::InternalWithMessage(e.to_string()))?;

    let special_user = match special_user {
        Some(u) => u,
        None => return Err(AppError::Auth("用户不存在或已禁用".to_string())),
    };

    // 验证 API 密钥
    if let Some(api_key_hash) = &special_user.api_key_hash {
        let valid = crate::core::password::verify_password(&payload.api_key, api_key_hash)
            .map_err(|e| AppError::InternalWithMessage(e.to_string()))?;

        if !valid {
            return Err(AppError::Auth("API密钥不正确".to_string()));
        }
    } else {
        return Err(AppError::Auth("该用户未设置API密钥".to_string()));
    }

    // 更新最后登录时间
    sqlx::query("UPDATE special_users SET last_login_at = NOW() WHERE id = $1")
        .bind(special_user.id)
        .execute(&pool)
        .await
        .map_err(|e| AppError::InternalWithMessage(e.to_string()))?;

    // 生成令牌
    let config = load_config().map_err(|e| AppError::InternalWithMessage(e.to_string()))?;
    let token = generate_special_user_token(
        &special_user.id.to_string(),
        &special_user.identifier,
        &special_user.user_type,
        &config.jwt_secret,
        24 * 30, // 30天有效期
    )
    .map_err(|e| AppError::InternalWithMessage(e.to_string()))?;

    // 记录登录日志
    get_global_logger()
        .info(
            &crate::models::special_user::OperatorInfo::user(
                special_user.id,
                &special_user.identifier,
            ),
            "special user login",
            format!("{} user: {} logged in", special_user.user_type, special_user.identifier),
        )
        .await;

    Ok(Json(SpecialUserLoginResponse {
        token,
        user_type: special_user.user_type,
        identifier: special_user.identifier,
        expires_in: 24 * 30 * 3600,
    }))
}

/// 获取操作日志列表查询参数
#[derive(Debug, Deserialize)]
pub struct ListOperationLogsQuery {
    pub limit: Option<i64>,
    pub offset: Option<i64>,
    pub date: Option<String>, // 格式: YYYY-MM-DD，默认为今天
}

/// 日志条目响应
#[derive(Debug, serde::Serialize)]
pub struct LogEntryResponse {
    pub timestamp: String,
    pub level: String,
    pub operator_type: String,
    pub operator_name: String,
    pub action: String,
    pub details: String,
}

pub async fn list_operation_logs(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Query(query): Query<ListOperationLogsQuery>,
) -> Result<Json<Vec<LogEntryResponse>>, AppError> {
    let pool = state.pool.ok_or_else(|| AppError::Internal)?;
    let user_id =
        Uuid::parse_str(&claims.sub).map_err(|_| AppError::Auth("无效的用户ID".to_string()))?;

    // 检查权限
    let manager = PermissionManager::new(pool.clone());
    let result = manager.check_permission(user_id, "operation_log.view").await;
    match result {
        PermissionResult::Allowed => (),
        _ => return Err(AppError::Auth("没有权限查看操作日志".to_string())),
    }

    let date = query.date.unwrap_or_else(|| chrono::Local::now().format("%Y-%m-%d").to_string());
    let limit = query.limit.unwrap_or(100) as usize;
    let offset = query.offset.unwrap_or(0) as usize;

    // 从文件读取日志
    let logs = get_global_logger().get_logs_from_file(&date);

    // 解析日志条目
    let mut responses: Vec<LogEntryResponse> = logs
        .into_iter()
        .filter_map(|line| parse_log_line(&line))
        .collect();

    // 倒序排列（最新的在前）
    responses.reverse();

    // 分页
    let paginated: Vec<LogEntryResponse> = responses
        .into_iter()
        .skip(offset)
        .take(limit)
        .collect();

    Ok(Json(paginated))
}

/// 解析日志行
fn parse_log_line(line: &str) -> Option<LogEntryResponse> {
    // 格式: [timestamp][level][operator_name][action][details]
    let parts: Vec<&str> = line.split(']').collect();
    if parts.len() >= 5 {
        let timestamp = parts[0].trim_start_matches('[').to_string();
        let level = parts[1].trim_start_matches('[').to_string();
        let operator_name = parts[2].trim_start_matches('[').to_string();
        let action = parts[3].trim_start_matches('[').to_string();
        let details = parts[4].trim_start_matches('[').to_string();

        Some(LogEntryResponse {
            timestamp,
            level,
            operator_type: "unknown".to_string(), // 从文件无法直接获取
            operator_name,
            action,
            details,
        })
    } else {
        None
    }
}
