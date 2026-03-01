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
pub struct ScoreQuery {
    pub page: Option<i64>,
    pub limit: Option<i64>,
    pub person_id: Option<Uuid>,
    pub group_id: Option<Uuid>,
    pub score_type: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreateScoreRequest {
    pub person_id: Uuid,
    pub group_id: Option<Uuid>,
    pub score_type: String,
    pub value: i32,
    pub reason: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdateScoreRequest {
    pub value: Option<i32>,
    pub reason: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ScoreResponse {
    pub id: Uuid,
    pub person_id: Uuid,
    pub person_name: String,
    pub group_id: Option<Uuid>,
    pub group_name: Option<String>,
    pub score_type: String,
    pub value: i32,
    pub reason: String,
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
    Query(query): Query<ScoreQuery>,
) -> Result<Json<ListResponse<ScoreResponse>>, AppError> {
    let pool = state.pool.ok_or_else(|| AppError::Internal)?;
    
    let page = query.page.unwrap_or(1);
    let limit = query.limit.unwrap_or(20);
    let offset = (page - 1) * limit;
    
    // 构建查询条件
    let mut conditions = vec!["1=1".to_string()];
    let mut param_index = 1;
    
    if query.person_id.is_some() {
        conditions.push(format!("s.person_id = ${}", param_index));
        param_index += 1;
    }
    
    if query.group_id.is_some() {
        conditions.push(format!("s.group_id = ${}", param_index));
        param_index += 1;
    }
    
    if query.score_type.is_some() {
        conditions.push(format!("s.score_type = ${}", param_index));
        param_index += 1;
    }
    
    let where_clause = conditions.join(" AND ");
    
    // 查询总数
    let count_sql = format!("SELECT COUNT(*) FROM scores s WHERE {}", where_clause);
    let mut count_query = sqlx::query_scalar(&count_sql);
    
    if let Some(ref person_id) = query.person_id {
        count_query = count_query.bind(person_id);
    }
    if let Some(ref group_id) = query.group_id {
        count_query = count_query.bind(group_id);
    }
    if let Some(ref score_type) = query.score_type {
        count_query = count_query.bind(score_type);
    }
    
    let total: i64 = count_query
        .fetch_one(&pool)
        .await
        .map_err(|e| AppError::Database(e))?;
    
    // 查询数据
    let sql = format!(
        "SELECT s.id, s.person_id, p.name as person_name, s.group_id, cg.name as group_name, 
         s.score_type, s.value, s.reason, s.created_at 
         FROM scores s 
         JOIN persons p ON s.person_id = p.id 
         LEFT JOIN class_groups cg ON s.group_id = cg.id 
         WHERE {} 
         ORDER BY s.created_at DESC 
         LIMIT ${} OFFSET ${}",
        where_clause,
        param_index,
        param_index + 1
    );
    
    let mut data_query = sqlx::query_as::<_, ScoreRow>(&sql);
    
    if let Some(ref person_id) = query.person_id {
        data_query = data_query.bind(person_id);
    }
    if let Some(ref group_id) = query.group_id {
        data_query = data_query.bind(group_id);
    }
    if let Some(ref score_type) = query.score_type {
        data_query = data_query.bind(score_type);
    }
    
    let scores = data_query
        .bind(limit)
        .bind(offset)
        .fetch_all(&pool)
        .await
        .map_err(|e| AppError::Database(e))?;
    
    let items: Vec<ScoreResponse> = scores.into_iter().map(|row| row.into()).collect();
    
    Ok(Json(ListResponse {
        items,
        total,
        page,
        limit,
    }))
}

#[derive(sqlx::FromRow)]
struct ScoreRow {
    id: Uuid,
    person_id: Uuid,
    person_name: String,
    group_id: Option<Uuid>,
    group_name: Option<String>,
    score_type: String,
    value: i32,
    reason: String,
    created_at: chrono::DateTime<chrono::Utc>,
}

impl From<ScoreRow> for ScoreResponse {
    fn from(row: ScoreRow) -> Self {
        Self {
            id: row.id,
            person_id: row.person_id,
            person_name: row.person_name,
            group_id: row.group_id,
            group_name: row.group_name,
            score_type: row.score_type,
            value: row.value,
            reason: row.reason,
            created_at: row.created_at.to_rfc3339(),
        }
    }
}

pub async fn create(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Json(req): Json<CreateScoreRequest>,
) -> Result<Json<ScoreResponse>, AppError> {
    let pool = state.pool.ok_or_else(|| AppError::Internal)?;
    
    // 检查权限
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| AppError::Auth("无效的用户ID".to_string()))?;
    let manager = PermissionManager::new(pool.clone());
    manager.require_permission(user_id, "score.create").await?;
    
    // 插入数据
    let row = sqlx::query_as::<_, ScoreRow>(
        "INSERT INTO scores (person_id, group_id, score_type, value, reason, created_by) 
         VALUES ($1, $2, $3, $4, $5, $6) 
         RETURNING id, person_id, (SELECT name FROM persons WHERE id = $1) as person_name, 
         group_id, (SELECT name FROM class_groups WHERE id = $2) as group_name, 
         score_type, value, reason, created_at"
    )
    .bind(req.person_id)
    .bind(req.group_id)
    .bind(&req.score_type)
    .bind(req.value)
    .bind(&req.reason)
    .bind(user_id)
    .fetch_one(&pool)
    .await
    .map_err(|e| AppError::Database(e))?;
    
    Ok(Json(row.into()))
}

pub async fn get(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<ScoreResponse>, AppError> {
    let pool = state.pool.ok_or_else(|| AppError::Internal)?;
    
    let row = sqlx::query_as::<_, ScoreRow>(
        "SELECT s.id, s.person_id, p.name as person_name, s.group_id, cg.name as group_name, 
         s.score_type, s.value, s.reason, s.created_at 
         FROM scores s 
         JOIN persons p ON s.person_id = p.id 
         LEFT JOIN class_groups cg ON s.group_id = cg.id 
         WHERE s.id = $1"
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
    Json(req): Json<UpdateScoreRequest>,
) -> Result<Json<ScoreResponse>, AppError> {
    let pool = state.pool.ok_or_else(|| AppError::Internal)?;
    
    // 检查权限
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| AppError::Auth("无效的用户ID".to_string()))?;
    let manager = PermissionManager::new(pool.clone());
    manager.require_permission(user_id, "score.update").await?;
    
    // 构建更新字段
    let mut updates = vec![];
    let mut param_index = 1;
    
    if req.value.is_some() {
        updates.push(format!("value = ${}", param_index));
        param_index += 1;
    }
    
    if req.reason.is_some() {
        updates.push(format!("reason = ${}", param_index));
        param_index += 1;
    }
    
    if updates.is_empty() {
        return Err(AppError::InvalidInput("没有要更新的字段".to_string()));
    }
    
    let sql = format!(
        "UPDATE scores SET {} WHERE id = ${} 
         RETURNING id, person_id, (SELECT name FROM persons WHERE id = scores.person_id) as person_name, 
         group_id, (SELECT name FROM class_groups WHERE id = scores.group_id) as group_name, 
         score_type, value, reason, created_at",
        updates.join(", "),
        param_index
    );
    
    let mut query = sqlx::query_as::<_, ScoreRow>(&sql);
    
    if let Some(value) = req.value {
        query = query.bind(value);
    }
    
    if let Some(reason) = req.reason {
        query = query.bind(reason);
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
    manager.require_permission(user_id, "score.delete").await?;
    
    let result = sqlx::query("DELETE FROM scores WHERE id = $1")
        .bind(id)
        .execute(&pool)
        .await
        .map_err(|e| AppError::Database(e))?;
    
    if result.rows_affected() == 0 {
        return Err(AppError::NotFound);
    }
    
    Ok(axum::http::StatusCode::NO_CONTENT)
}
