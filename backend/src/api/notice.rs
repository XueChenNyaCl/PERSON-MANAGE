use axum::{
    extract::{Query, State, Path, Extension},
    Json,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::api::routes::AppState;
use crate::core::error::AppError;
use crate::core::auth::Claims;
use crate::core::permission::PermissionManager;

#[derive(Debug, Deserialize)]
pub struct NoticeQuery {
    pub page: Option<i64>,
    pub limit: Option<i64>,
    pub target_type: Option<String>,
    pub target_id: Option<Uuid>,
    pub search: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreateNoticeRequest {
    pub title: String,
    pub content: String,
    pub target_type: String,
    pub target_id: Option<Uuid>,
    pub is_important: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateNoticeRequest {
    pub title: Option<String>,
    pub content: Option<String>,
    pub is_important: Option<bool>,
}

#[derive(Debug, Serialize)]
pub struct NoticeResponse {
    pub id: Uuid,
    pub title: String,
    pub content: String,
    pub author_id: Uuid,
    pub author_name: String,
    pub target_type: String,
    pub target_id: Option<Uuid>,
    pub is_important: bool,
    pub created_at: String,
}

#[derive(Debug, Serialize)]
pub struct ListResponse<T> {
    pub items: Vec<T>,
    pub total: i64,
    pub page: i64,
    pub limit: i64,
}

pub async fn list(
    State(state): State<AppState>,
    Query(query): Query<NoticeQuery>,
) -> Result<Json<ListResponse<NoticeResponse>>, AppError> {
    let pool = state.pool.ok_or_else(|| AppError::Internal)?;
    
    let page = query.page.unwrap_or(1);
    let limit = query.limit.unwrap_or(20);
    let offset = (page - 1) * limit;
    
    // 如果有搜索条件，使用简化查询
    if let Some(ref search) = query.search {
        let search_pattern = format!("%{}%", search);
        
        // 查询总数
        let total: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM notices n WHERE (n.title ILIKE $1 OR n.content ILIKE $1)"
        )
        .bind(&search_pattern)
        .fetch_one(&pool)
        .await
        .map_err(|e| AppError::Database(e))?;
        
        // 查询数据
        let notices = sqlx::query_as::<_, NoticeRow>(
            "SELECT n.id, n.title, n.content, n.author_id, p.name as author_name, 
             n.target_type, n.target_id, n.is_important, n.created_at 
             FROM notices n 
             JOIN persons p ON n.author_id = p.id 
             WHERE (n.title ILIKE $1 OR n.content ILIKE $1)
             ORDER BY n.is_important DESC, n.created_at DESC 
             LIMIT $2 OFFSET $3"
        )
        .bind(&search_pattern)
        .bind(limit)
        .bind(offset)
        .fetch_all(&pool)
        .await
        .map_err(|e| AppError::Database(e))?;
        
        let items: Vec<NoticeResponse> = notices.into_iter().map(|row| row.into()).collect();
        
        return Ok(Json(ListResponse {
            items,
            total,
            page,
            limit,
        }));
    }
    
    // 构建查询条件
    let mut conditions = vec!["1=1".to_string()];
    let mut param_index = 1;
    
    if query.target_type.is_some() {
        conditions.push(format!("n.target_type = ${}", param_index));
        param_index += 1;
    }
    
    if query.target_id.is_some() {
        conditions.push(format!("n.target_id = ${}", param_index));
        param_index += 1;
    }
    
    let where_clause = conditions.join(" AND ");
    
    // 查询总数
    let count_sql = format!("SELECT COUNT(*) FROM notices n WHERE {}", where_clause);
    let mut count_query = sqlx::query_scalar(&count_sql);
    
    if let Some(ref target_type) = query.target_type {
        count_query = count_query.bind(target_type);
    }
    if let Some(ref target_id) = query.target_id {
        count_query = count_query.bind(target_id);
    }
    
    let total: i64 = count_query
        .fetch_one(&pool)
        .await
        .map_err(|e| AppError::Database(e))?;
    
    // 查询数据
    let sql = format!(
        "SELECT n.id, n.title, n.content, n.author_id, p.name as author_name, 
         n.target_type, n.target_id, n.is_important, n.created_at 
         FROM notices n 
         JOIN persons p ON n.author_id = p.id 
         WHERE {} 
         ORDER BY n.is_important DESC, n.created_at DESC 
         LIMIT ${} OFFSET ${}",
        where_clause,
        param_index,
        param_index + 1
    );
    
    let mut data_query = sqlx::query_as::<_, NoticeRow>(&sql);
    
    if let Some(ref target_type) = query.target_type {
        data_query = data_query.bind(target_type);
    }
    if let Some(ref target_id) = query.target_id {
        data_query = data_query.bind(target_id);
    }
    
    let notices = data_query
        .bind(limit)
        .bind(offset)
        .fetch_all(&pool)
        .await
        .map_err(|e| AppError::Database(e))?;
    
    let items: Vec<NoticeResponse> = notices.into_iter().map(|row| row.into()).collect();
    
    Ok(Json(ListResponse {
        items,
        total,
        page,
        limit,
    }))
}

#[derive(sqlx::FromRow)]
struct NoticeRow {
    id: Uuid,
    title: String,
    content: String,
    author_id: Uuid,
    author_name: String,
    target_type: String,
    target_id: Option<Uuid>,
    is_important: bool,
    created_at: chrono::DateTime<chrono::Utc>,
}

impl From<NoticeRow> for NoticeResponse {
    fn from(row: NoticeRow) -> Self {
        Self {
            id: row.id,
            title: row.title,
            content: row.content,
            author_id: row.author_id,
            author_name: row.author_name,
            target_type: row.target_type,
            target_id: row.target_id,
            is_important: row.is_important,
            created_at: row.created_at.to_rfc3339(),
        }
    }
}

pub async fn create(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Json(req): Json<CreateNoticeRequest>,
) -> Result<Json<NoticeResponse>, AppError> {
    let pool = state.pool.ok_or_else(|| AppError::Internal)?;
    
    // 检查权限
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| AppError::Auth("无效的用户ID".to_string()))?;
    let manager = PermissionManager::new(pool.clone());
    manager.require_permission(user_id, "notice.create").await?;
    
    // 插入数据
    let row = sqlx::query_as::<_, NoticeRow>(
        "INSERT INTO notices (title, content, author_id, target_type, target_id, is_important) 
         VALUES ($1, $2, $3, $4, $5, $6) 
         RETURNING id, title, content, author_id, (SELECT name FROM persons WHERE id = $3) as author_name, 
         target_type, target_id, is_important, created_at"
    )
    .bind(&req.title)
    .bind(&req.content)
    .bind(user_id)
    .bind(&req.target_type)
    .bind(req.target_id)
    .bind(req.is_important.unwrap_or(false))
    .fetch_one(&pool)
    .await
    .map_err(|e| AppError::Database(e))?;
    
    Ok(Json(row.into()))
}

pub async fn get(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<NoticeResponse>, AppError> {
    let pool = state.pool.ok_or_else(|| AppError::Internal)?;
    
    let row = sqlx::query_as::<_, NoticeRow>(
        "SELECT n.id, n.title, n.content, n.author_id, p.name as author_name, 
         n.target_type, n.target_id, n.is_important, n.created_at 
         FROM notices n 
         JOIN persons p ON n.author_id = p.id 
         WHERE n.id = $1"
    )
    .bind(id)
    .fetch_optional(&pool)
    .await
    .map_err(|e| AppError::Database(e))?
    .ok_or(AppError::NotFound)?;
    
    Ok(Json(row.into()))
}

pub async fn update(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateNoticeRequest>,
) -> Result<Json<NoticeResponse>, AppError> {
    let pool = state.pool.ok_or_else(|| AppError::Internal)?;
    
    // 检查权限
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| AppError::Auth("无效的用户ID".to_string()))?;
    let manager = PermissionManager::new(pool.clone());
    manager.require_permission(user_id, "notice.update").await?;
    
    // 构建更新字段
    let mut updates = vec![];
    let mut param_index = 1;
    
    if req.title.is_some() {
        updates.push(format!("title = ${}", param_index));
        param_index += 1;
    }
    
    if req.content.is_some() {
        updates.push(format!("content = ${}", param_index));
        param_index += 1;
    }
    
    if req.is_important.is_some() {
        updates.push(format!("is_important = ${}", param_index));
        param_index += 1;
    }
    
    if updates.is_empty() {
        return Err(AppError::InvalidInput("没有要更新的字段".to_string()));
    }
    
    let sql = format!(
        "UPDATE notices SET {} WHERE id = ${} 
         RETURNING id, title, content, author_id, (SELECT name FROM persons WHERE id = notices.author_id) as author_name, 
         target_type, target_id, is_important, created_at",
        updates.join(", "),
        param_index
    );
    
    let mut query = sqlx::query_as::<_, NoticeRow>(&sql);
    
    if let Some(title) = req.title {
        query = query.bind(title);
    }
    
    if let Some(content) = req.content {
        query = query.bind(content);
    }
    
    if let Some(is_important) = req.is_important {
        query = query.bind(is_important);
    }
    
    let row = query
        .bind(id)
        .fetch_optional(&pool)
        .await
        .map_err(|e| AppError::Database(e))?
        .ok_or(AppError::NotFound)?;
    
    Ok(Json(row.into()))
}

pub async fn delete(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<axum::http::StatusCode, AppError> {
    let pool = state.pool.ok_or_else(|| AppError::Internal)?;
    
    // 检查权限
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| AppError::Auth("无效的用户ID".to_string()))?;
    let manager = PermissionManager::new(pool.clone());
    manager.require_permission(user_id, "notice.delete").await?;
    
    let result = sqlx::query("DELETE FROM notices WHERE id = $1")
        .bind(id)
        .execute(&pool)
        .await
        .map_err(|e| AppError::Database(e))?;
    
    if result.rows_affected() == 0 {
        return Err(AppError::NotFound);
    }
    
    Ok(axum::http::StatusCode::NO_CONTENT)
}
